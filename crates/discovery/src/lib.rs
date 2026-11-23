//! PS4 discovery over UDP port 987. Wire format verified against chiaki-ng `lib/src/discovery.c`
//! (see docs/research.md). Replies come from the network and are treated as hostile input:
//! size-capped, strictly parsed, sanitised, and only accepted from the address we asked.

use ps4studio_core::capability::{DiscoveryFact, DiscoveryVia, HostState};
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Instant};

pub const PORT_PS4: u16 = 987;
/// TCP port of the PS4 Remote Play service (answers even when the UDP search is ignored).
pub const REMOTE_PLAY_PORT: u16 = 9295;
pub const PROTOCOL_VERSION_PS4: &str = "00020020";
const MAX_REPLY: usize = 2048;
const MAX_FIELD: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveryReply {
    pub state: HostState,
    pub host_name: Option<String>,
    pub host_id: Option<String>,
    pub host_type: Option<String>,
    pub running_title: Option<String>,
    /// Raw `system-version` digits, e.g. "09008001". Only a coarse hint.
    pub system_version: Option<String>,
}

impl DiscoveryReply {
    pub fn fact(&self) -> DiscoveryFact {
        DiscoveryFact {
            via: DiscoveryVia::UdpSearch,
            state: self.state,
            host_name: self.host_name.clone(),
            running_title: self.running_title.clone(),
        }
    }
    /// "9.00"-style hint from `system-version` (major = first two digits, minor = next two).
    /// Coarse and unverified; callers label it as reported.
    pub fn firmware_hint(&self) -> Option<String> {
        let v = self.system_version.as_deref()?;
        if v.len() < 4 || !v.is_ascii() {
            return None;
        }
        let major: u32 = v[..2].parse().ok()?;
        Some(format!("{major}.{}", &v[2..4]))
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("reply too large")]
    TooLarge,
    #[error("reply is not text")]
    NotText,
    #[error("reply is not a discovery response")]
    BadStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("no reply from console (timed out)")]
    Timeout,
    #[error("the service on that port is not a PlayStation Remote Play service")]
    NotRemotePlay,
    #[error("network error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn srch_packet() -> Vec<u8> {
    format!("SRCH * HTTP/1.1\ndevice-discovery-protocol-version:{PROTOCOL_VERSION_PS4}\n")
        .into_bytes()
}

fn clean(v: &str) -> Option<String> {
    let s: String = v
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FIELD)
        .collect();
    (!s.is_empty()).then_some(s)
}

pub fn parse_reply(buf: &[u8]) -> Result<DiscoveryReply, ParseError> {
    if buf.len() > MAX_REPLY {
        return Err(ParseError::TooLarge);
    }
    let text = std::str::from_utf8(buf).map_err(|_| ParseError::NotText)?;
    let mut lines = text.split('\n').map(|l| l.trim_end_matches('\r'));
    let status = lines.next().unwrap_or("");
    let mut parts = status.split_whitespace();
    if parts.next() != Some("HTTP/1.1") {
        return Err(ParseError::BadStatus);
    }
    let state = match parts.next() {
        Some("200") => HostState::Ready,
        Some("620") => HostState::Standby,
        _ => return Err(ParseError::BadStatus),
    };
    let mut r = DiscoveryReply {
        state,
        host_name: None,
        host_id: None,
        host_type: None,
        running_title: None,
        system_version: None,
    };
    for line in lines.take(32) {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        match k.trim() {
            "host-name" => r.host_name = clean(v),
            "host-id" => r.host_id = clean(v),
            "host-type" => r.host_type = clean(v),
            "running-app-name" => r.running_title = clean(v),
            "system-version" => {
                r.system_version = clean(v).filter(|s| s.chars().all(|c| c.is_ascii_digit()))
            }
            _ => {}
        }
    }
    Ok(r)
}

/// Unicast probe with retries. Replies from any other source address are ignored.
pub async fn probe(
    target: SocketAddr,
    per_try: Duration,
    tries: u32,
) -> Result<DiscoveryReply, ProbeError> {
    let bind: SocketAddr = if target.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
    .parse()
    .expect("static addr");
    let sock = UdpSocket::bind(bind).await?;
    let mut buf = [0u8; MAX_REPLY + 1];
    for _ in 0..tries.max(1) {
        sock.send_to(&srch_packet(), target).await?;
        let deadline = Instant::now() + per_try;
        while let Ok(res) = timeout(
            deadline.saturating_duration_since(Instant::now()),
            sock.recv_from(&mut buf),
        )
        .await
        {
            let (n, from) = res?;
            if from.ip() != target.ip() {
                continue; // spoofed / unrelated
            }
            if let Ok(r) = parse_reply(&buf[..n]) {
                return Ok(r);
            }
        }
    }
    Err(ProbeError::Timeout)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemotePlayInfo {
    pub rp_version: String,
}

/// Strictly recognises a PlayStation Remote Play reply: an HTTP status line plus an `RP-Version`
/// header whose value is digits and dots. Anything else is rejected.
pub fn parse_rp_response(buf: &[u8]) -> Option<RemotePlayInfo> {
    if buf.len() > MAX_REPLY {
        return None;
    }
    let text = std::str::from_utf8(buf).ok()?;
    let mut lines = text.split('\n').map(|l| l.trim_end_matches('\r'));
    if !lines.next()?.starts_with("HTTP/1.1 ") {
        return None;
    }
    for line in lines.take(32) {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("rp-version") {
                let v = v.trim();
                let ok = !v.is_empty()
                    && v.len() <= 16
                    && v.chars().all(|c| c.is_ascii_digit() || c == '.');
                return ok.then(|| RemotePlayInfo {
                    rp_version: v.to_owned(),
                });
            }
        }
    }
    None
}

/// Sends ONE unauthenticated request (no credentials, no registration key) to the Remote Play port
/// and checks for the Remote Play signature. Used as a fallback when the UDP search is ignored.
pub async fn probe_remote_play(
    addr: SocketAddr,
    dur: Duration,
) -> Result<RemotePlayInfo, ProbeError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut s = timeout(dur, tokio::net::TcpStream::connect(addr))
        .await
        .map_err(|_| ProbeError::Timeout)??;
    let req = format!(
        "GET /sce/rp/session HTTP/1.1\r\nHost: {addr}\r\nUser-Agent: PS4Studio\r\nConnection: close\r\nContent-Length: 0\r\nRP-Version: 10.0\r\n\r\n"
    );
    timeout(dur, s.write_all(req.as_bytes()))
        .await
        .map_err(|_| ProbeError::Timeout)??;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 512];
    // Read until the header block ends, EOF, or the size cap; every read is time-limited.
    while buf.len() <= MAX_REPLY && !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = timeout(dur, s.read(&mut chunk))
            .await
            .map_err(|_| ProbeError::Timeout)??;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    parse_rp_response(&buf).ok_or(ProbeError::NotRemotePlay)
}

#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub address: IpAddr,
    pub reply: DiscoveryReply,
}

/// User-initiated LAN search: limited broadcast (never routed beyond the local segment).
pub async fn discover_lan(wait: Duration) -> Result<Vec<Found>, ProbeError> {
    discover_via(SocketAddr::from((Ipv4Addr::BROADCAST, PORT_PS4)), wait).await
}

pub async fn discover_via(dest: SocketAddr, wait: Duration) -> Result<Vec<Found>, ProbeError> {
    let std_sock = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
    std_sock.set_broadcast(true)?;
    std_sock.set_nonblocking(true)?;
    std_sock.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())?;
    let sock = UdpSocket::from_std(std_sock.into())?;
    sock.send_to(&srch_packet(), dest).await?;
    let mut found: Vec<Found> = Vec::new();
    let mut buf = [0u8; MAX_REPLY + 1];
    let deadline = Instant::now() + wait;
    while let Ok(res) = timeout(
        deadline.saturating_duration_since(Instant::now()),
        sock.recv_from(&mut buf),
    )
    .await
    {
        let (n, from) = res?;
        if found.len() >= 16 || found.iter().any(|f| f.address == from.ip()) {
            continue;
        }
        if let Ok(reply) = parse_reply(&buf[..n]) {
            found.push(Found {
                address: from.ip(),
                reply,
            });
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "HTTP/1.1 200 Ok\nhost-id:AABBCCDDEEFF\nhost-type:PS4\nhost-name:PS4-123\nhost-request-port:997\ndevice-discovery-protocol-version:00020020\nsystem-version:09008001\n\n";

    #[test]
    fn packet_matches_documented_format() {
        assert_eq!(
            srch_packet(),
            b"SRCH * HTTP/1.1\ndevice-discovery-protocol-version:00020020\n"
        );
    }

    #[test]
    fn parses_ready_and_standby() {
        let r = parse_reply(GOOD.as_bytes()).unwrap();
        assert_eq!(r.state, HostState::Ready);
        assert_eq!(r.host_name.as_deref(), Some("PS4-123"));
        assert_eq!(r.firmware_hint().as_deref(), Some("9.00"));
        let s = parse_reply(b"HTTP/1.1 620 Server Standby\nhost-name:x\n\n").unwrap();
        assert_eq!(s.state, HostState::Standby);
    }

    #[test]
    fn rejects_hostile_replies() {
        assert_eq!(parse_reply(&vec![b'A'; 5000]), Err(ParseError::TooLarge));
        assert_eq!(parse_reply(&[0xff, 0xfe, 0x00]), Err(ParseError::NotText));
        assert_eq!(parse_reply(b""), Err(ParseError::BadStatus));
        assert_eq!(
            parse_reply(b"HTTP/1.1 404 Nope\n"),
            Err(ParseError::BadStatus)
        );
        assert_eq!(parse_reply(b"GET / HTTP/1.1\n"), Err(ParseError::BadStatus));
        assert_eq!(parse_reply(b"HTTP/1.1 200\n").unwrap().host_name, None);
    }

    #[test]
    fn sanitises_fields() {
        let evil = format!(
            "HTTP/1.1 200 Ok\nhost-name:{}\u{1b}[31mred\u{7}\nsystem-version:../../etc\n",
            "A".repeat(500)
        );
        let r = parse_reply(evil.as_bytes()).unwrap();
        assert!(r.host_name.as_ref().unwrap().chars().count() <= MAX_FIELD);
        assert!(!r.host_name.as_ref().unwrap().contains('\u{7}'));
        assert_eq!(r.system_version, None, "non-digit version is dropped");
        assert_eq!(r.firmware_hint(), None);
    }

    #[test]
    fn many_headers_do_not_blow_up() {
        let mut s = String::from("HTTP/1.1 200 Ok\n");
        for i in 0..1000 {
            s.push_str(&format!("x{i}:y\n"));
        }
        assert_eq!(parse_reply(s.as_bytes()), Err(ParseError::TooLarge));
    }

    async fn responder(reply: Vec<Vec<u8>>, delay: Duration) -> SocketAddr {
        let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = sock.local_addr().unwrap();
        tokio::spawn(async move {
            let mut b = [0u8; 512];
            loop {
                let Ok((_, from)) = sock.recv_from(&mut b).await else {
                    return;
                };
                tokio::time::sleep(delay).await;
                for r in &reply {
                    let _ = sock.send_to(r, from).await;
                }
            }
        });
        addr
    }

    #[test]
    fn recognises_only_the_remote_play_signature() {
        let real = b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\nPragma: no-cache\r\nContent-Length: 0\r\nRP-Version: 10.0\r\nRP-Application-Reason: 80108bff\r\n\r\n";
        assert_eq!(
            parse_rp_response(real),
            Some(RemotePlayInfo {
                rp_version: "10.0".into()
            })
        );
        for bad in [
            &b"HTTP/1.1 200 OK\r\nServer: nginx\r\n\r\n"[..],
            b"SSH-2.0-OpenSSH_9.9\r\n",
            b"HTTP/1.1 403 Forbidden\r\nRP-Version: ../../x\r\n\r\n",
            b"HTTP/1.1 403 Forbidden\r\nRP-Version: \r\n\r\n",
            b"\xff\xfe",
            b"",
        ] {
            assert_eq!(parse_rp_response(bad), None, "{bad:?}");
        }
        assert_eq!(parse_rp_response(&vec![b'A'; 5000]), None);
    }

    async fn tcp_server(script: Vec<u8>, linger: bool) -> SocketAddr {
        use tokio::io::AsyncWriteExt;
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut s, _) = l.accept().await.unwrap();
            let _ = s.write_all(&script).await;
            if linger {
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        addr
    }

    #[tokio::test]
    async fn remote_play_probe_accepts_real_and_rejects_hostile_services() {
        let ok = tcp_server(
            b"HTTP/1.1 403 Forbidden\r\nRP-Version: 10.0\r\n\r\n".to_vec(),
            true,
        )
        .await;
        assert_eq!(
            probe_remote_play(ok, Duration::from_millis(500))
                .await
                .unwrap()
                .rp_version,
            "10.0"
        );
        let web = tcp_server(b"HTTP/1.1 200 OK\r\nServer: x\r\n\r\n".to_vec(), false).await;
        assert!(matches!(
            probe_remote_play(web, Duration::from_millis(500)).await,
            Err(ProbeError::NotRemotePlay)
        ));
        let endless = tcp_server(vec![b'A'; 200_000], false).await;
        assert!(matches!(
            probe_remote_play(endless, Duration::from_millis(500)).await,
            Err(ProbeError::NotRemotePlay)
        ));
        let silent = tcp_server(vec![], true).await;
        assert!(matches!(
            probe_remote_play(silent, Duration::from_millis(200)).await,
            Err(ProbeError::Timeout)
        ));
        let closed = {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let a = l.local_addr().unwrap();
            drop(l);
            a
        };
        assert!(probe_remote_play(closed, Duration::from_millis(300))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn probe_succeeds_and_skips_garbage_first() {
        let addr = responder(
            vec![b"junk".to_vec(), GOOD.as_bytes().to_vec()],
            Duration::ZERO,
        )
        .await;
        let r = probe(addr, Duration::from_millis(300), 1).await.unwrap();
        assert_eq!(r.host_name.as_deref(), Some("PS4-123"));
    }

    #[tokio::test]
    async fn probe_times_out_when_silent_or_late() {
        let silent = responder(vec![], Duration::ZERO).await;
        assert!(matches!(
            probe(silent, Duration::from_millis(80), 2).await,
            Err(ProbeError::Timeout)
        ));
        let late = responder(vec![GOOD.as_bytes().to_vec()], Duration::from_millis(400)).await;
        assert!(matches!(
            probe(late, Duration::from_millis(80), 1).await,
            Err(ProbeError::Timeout)
        ));
    }

    #[tokio::test]
    async fn broadcast_search_collects_unique_hosts() {
        let addr = responder(
            vec![GOOD.as_bytes().to_vec(), GOOD.as_bytes().to_vec()],
            Duration::ZERO,
        )
        .await;
        let found = discover_via(addr, Duration::from_millis(200))
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
    }
}
