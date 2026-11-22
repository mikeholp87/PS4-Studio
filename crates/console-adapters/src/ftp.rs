//! Minimal hardened FTP client (RFC 959 subset, passive mode only).
//!
//! FTP is unencrypted: every caller must show that warning. Server replies are untrusted input:
//! lines are length-capped, replies are size-capped, listings are count/byte-capped, names are
//! validated, the data connection always goes to the control peer's IP (never to an address the
//! server advertises, which prevents FTP-bounce style redirection), and every await has a timeout.

use crate::paths::valid_name;
use ps4studio_core::capability::FtpFact;
use serde::Serialize;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

const MAX_LINE: usize = 1024;
const MAX_REPLY_LINES: usize = 64;
const MAX_LIST_BYTES: usize = 4 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
const CHUNK: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FtpError {
    #[error("the service did not respond in time")]
    Timeout,
    #[error("network error: {0}")]
    Io(#[from] std::io::Error),
    #[error("this is not an FTP service ({0})")]
    NotFtp(String),
    #[error("unexpected response from the service: {0}")]
    Protocol(String),
    #[error("the service refused the request ({code}): {text}")]
    Rejected { code: u16, text: String },
    #[error("the service rejected the login")]
    AuthFailed,
    #[error("cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    File,
    Dir,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub size: u64,
}

#[derive(Debug)]
struct Reply {
    code: u16,
    text: String,
}

pub struct FtpClient {
    ctrl: BufReader<TcpStream>,
    peer: IpAddr,
    dur: Duration,
    /// Set after a cancel/abort/IO failure mid-transfer: the control channel can no longer be trusted.
    broken: bool,
}

fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(200).collect()
}

async fn read_line_limited<R: AsyncBufReadExt + Unpin>(r: &mut R) -> Result<String, FtpError> {
    let mut buf = Vec::new();
    let n = (&mut *r).take(MAX_LINE as u64 + 1).read_until(b'\n', &mut buf).await?;
    if n == 0 {
        return Err(FtpError::Protocol("connection closed".into()));
    }
    if buf.last() != Some(&b'\n') {
        return Err(FtpError::Protocol("reply line too long".into()));
    }
    Ok(String::from_utf8_lossy(&buf).trim_end_matches(['\r', '\n']).to_owned())
}

impl FtpClient {
    pub async fn connect(addr: SocketAddr, user: &str, pass: &str, dur: Duration) -> Result<Self, FtpError> {
        let tcp = timeout(dur, TcpStream::connect(addr)).await.map_err(|_| FtpError::Timeout)??;
        let mut c = Self { ctrl: BufReader::new(tcp), peer: addr.ip(), dur, broken: false };
        let greet = c.read_reply().await.map_err(|e| match e {
            FtpError::Protocol(m) => FtpError::NotFtp(m),
            o => o,
        })?;
        if greet.code != 220 {
            return Err(FtpError::NotFtp(format!("greeting code {}", greet.code)));
        }
        let r = c.cmd(&format!("USER {user}")).await?;
        match r.code {
            230 => {}
            331 | 332 => {
                let p = c.cmd(&format!("PASS {pass}")).await?;
                match p.code {
                    230 | 202 => {}
                    530 | 430 | 421 => return Err(FtpError::AuthFailed),
                    code => return Err(FtpError::Rejected { code, text: p.text }),
                }
            }
            530 | 430 => return Err(FtpError::AuthFailed),
            code => return Err(FtpError::Rejected { code, text: r.text }),
        }
        c.expect("TYPE I", &[200]).await?;
        Ok(c)
    }

    async fn read_reply(&mut self) -> Result<Reply, FtpError> {
        timeout(self.dur, async {
            let first = read_line_limited(&mut self.ctrl).await?;
            let b = first.as_bytes();
            if b.len() < 4 || !b[..3].iter().all(u8::is_ascii_digit) || (b[3] != b' ' && b[3] != b'-') {
                return Err(FtpError::Protocol(clean(&first)));
            }
            let code: u16 = first[..3].parse().map_err(|_| FtpError::Protocol(clean(&first)))?;
            let mut text = clean(&first[4..]);
            if b[3] == b'-' {
                let end = format!("{code} ");
                for _ in 0..MAX_REPLY_LINES {
                    let l = read_line_limited(&mut self.ctrl).await?;
                    if l.starts_with(&end) {
                        return Ok(Reply { code, text });
                    }
                }
                return Err(FtpError::Protocol("reply too long".into()));
            }
            text.truncate(200);
            Ok(Reply { code, text })
        })
        .await
        .map_err(|_| FtpError::Timeout)?
    }

    async fn cmd(&mut self, line: &str) -> Result<Reply, FtpError> {
        if self.broken {
            return Err(FtpError::Protocol("connection must be reopened".into()));
        }
        if line.contains(['\r', '\n', '\0']) {
            return Err(FtpError::Protocol("illegal characters in command".into()));
        }
        let r = timeout(self.dur, self.ctrl.get_mut().write_all(format!("{line}\r\n").as_bytes())).await;
        match r {
            Err(_) => {
                self.broken = true;
                return Err(FtpError::Timeout);
            }
            Ok(Err(e)) => {
                self.broken = true;
                return Err(e.into());
            }
            Ok(Ok(())) => {}
        }
        self.read_reply().await.inspect_err(|_| self.broken = true)
    }

    async fn expect(&mut self, line: &str, ok: &[u16]) -> Result<Reply, FtpError> {
        let r = self.cmd(line).await?;
        if ok.contains(&r.code) {
            Ok(r)
        } else {
            Err(FtpError::Rejected { code: r.code, text: r.text })
        }
    }

    async fn pasv(&mut self) -> Result<TcpStream, FtpError> {
        let r = self.expect("PASV", &[227]).await?;
        let inner = r.text.split_once('(').and_then(|(_, t)| t.split_once(')')).map(|(i, _)| i).ok_or_else(|| FtpError::Protocol("bad PASV reply".into()))?;
        let n: Vec<u16> = inner.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if n.len() != 6 || n.iter().any(|v| *v > 255) {
            return Err(FtpError::Protocol("bad PASV reply".into()));
        }
        let port = n[4] * 256 + n[5];
        if port == 0 {
            return Err(FtpError::Protocol("bad PASV port".into()));
        }
        // Deliberately ignore the advertised IP: always the control peer.
        timeout(self.dur, TcpStream::connect(SocketAddr::new(self.peer, port))).await.map_err(|_| FtpError::Timeout)?.map_err(Into::into)
    }

    pub async fn pwd(&mut self) -> Result<String, FtpError> {
        Ok(self.expect("PWD", &[257]).await?.text)
    }

    pub async fn list(&mut self, path: &str) -> Result<Vec<Entry>, FtpError> {
        self.expect("TYPE A", &[200]).await?;
        let mut data = self.pasv().await?;
        let r = self.cmd(&format!("LIST {path}")).await?;
        if r.code != 125 && r.code != 150 {
            return Err(FtpError::Rejected { code: r.code, text: r.text });
        }
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let n = timeout(self.dur, data.read(&mut chunk)).await.map_err(|_| FtpError::Timeout)??;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.len() > MAX_LIST_BYTES {
                self.broken = true;
                return Err(FtpError::Protocol("directory listing too large".into()));
            }
        }
        drop(data);
        let done = self.read_reply().await?;
        if done.code != 226 && done.code != 250 {
            return Err(FtpError::Rejected { code: done.code, text: done.text });
        }
        self.expect("TYPE I", &[200]).await?;
        let text = String::from_utf8_lossy(&buf);
        Ok(text.lines().filter_map(parse_list_line).take(MAX_ENTRIES).collect())
    }

    pub async fn size(&mut self, path: &str) -> Result<Option<u64>, FtpError> {
        let r = self.cmd(&format!("SIZE {path}")).await?;
        Ok(if r.code == 213 { r.text.trim().parse().ok() } else { None })
    }

    pub async fn mkd(&mut self, path: &str) -> Result<(), FtpError> {
        self.expect(&format!("MKD {path}"), &[257]).await.map(drop)
    }
    pub async fn rmd(&mut self, path: &str) -> Result<(), FtpError> {
        self.expect(&format!("RMD {path}"), &[250]).await.map(drop)
    }
    pub async fn dele(&mut self, path: &str) -> Result<(), FtpError> {
        self.expect(&format!("DELE {path}"), &[250]).await.map(drop)
    }
    pub async fn rename(&mut self, from: &str, to: &str) -> Result<(), FtpError> {
        self.expect(&format!("RNFR {from}"), &[350]).await?;
        self.expect(&format!("RNTO {to}"), &[250]).await.map(drop)
    }

    /// Streams `remote` into `out`. `progress(bytes_so_far)` is called per chunk. On cancel the
    /// connection is dropped and this client must not be reused.
    pub async fn download<W: AsyncWrite + Unpin>(&mut self, remote: &str, out: &mut W, progress: &(dyn Fn(u64) + Sync), cancel: &AtomicBool) -> Result<u64, FtpError> {
        let mut data = self.pasv().await?;
        let r = self.cmd(&format!("RETR {remote}")).await?;
        if r.code != 125 && r.code != 150 {
            return Err(FtpError::Rejected { code: r.code, text: r.text });
        }
        let mut total = 0u64;
        let mut buf = vec![0u8; CHUNK];
        loop {
            if cancel.load(Ordering::Relaxed) {
                self.broken = true;
                return Err(FtpError::Cancelled);
            }
            let n = match timeout(self.dur, data.read(&mut buf)).await {
                Err(_) => {
                    self.broken = true;
                    return Err(FtpError::Timeout);
                }
                Ok(Err(e)) => {
                    self.broken = true;
                    return Err(e.into());
                }
                Ok(Ok(n)) => n,
            };
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n]).await?;
            total += n as u64;
            progress(total);
        }
        drop(data);
        let done = self.read_reply().await.inspect_err(|_| self.broken = true)?;
        if done.code != 226 && done.code != 250 {
            return Err(FtpError::Rejected { code: done.code, text: done.text });
        }
        Ok(total)
    }

    pub async fn upload<R: AsyncRead + Unpin>(&mut self, input: &mut R, remote: &str, progress: &(dyn Fn(u64) + Sync), cancel: &AtomicBool) -> Result<u64, FtpError> {
        let mut data = self.pasv().await?;
        let r = self.cmd(&format!("STOR {remote}")).await?;
        if r.code != 125 && r.code != 150 {
            return Err(FtpError::Rejected { code: r.code, text: r.text });
        }
        let mut total = 0u64;
        let mut buf = vec![0u8; CHUNK];
        loop {
            if cancel.load(Ordering::Relaxed) {
                self.broken = true;
                return Err(FtpError::Cancelled);
            }
            let n = input.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            match timeout(self.dur, data.write_all(&buf[..n])).await {
                Err(_) => {
                    self.broken = true;
                    return Err(FtpError::Timeout);
                }
                Ok(Err(e)) => {
                    self.broken = true;
                    return Err(e.into());
                }
                Ok(Ok(())) => {}
            }
            total += n as u64;
            progress(total);
        }
        let _ = data.shutdown().await;
        drop(data);
        let done = self.read_reply().await.inspect_err(|_| self.broken = true)?;
        if done.code != 226 && done.code != 250 {
            return Err(FtpError::Rejected { code: done.code, text: done.text });
        }
        Ok(total)
    }

    pub async fn quit(mut self) {
        let _ = self.cmd("QUIT").await;
    }
}

/// Parses a Unix-style `LIST` line. Names that are not plain single path segments are dropped.
pub fn parse_list_line(line: &str) -> Option<Entry> {
    let line = line.trim_end();
    let kind = match line.chars().next()? {
        'd' => Kind::Dir,
        'l' => Kind::Link,
        '-' => Kind::File,
        _ => return None,
    };
    let mut rest = line;
    let mut tok: Vec<&str> = Vec::with_capacity(8);
    for _ in 0..8 {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace)?;
        tok.push(&rest[..end]);
        rest = &rest[end..];
    }
    let name = rest.trim_start();
    let name = if kind == Kind::Link { name.split(" -> ").next().unwrap_or(name) } else { name };
    if !valid_name(name) {
        return None;
    }
    Some(Entry { name: name.to_owned(), kind, size: tok[4].parse().unwrap_or(0) })
}

/// Capability probe: only a completed FTP greeting + login + directory listing counts.
pub async fn probe(addr: SocketAddr, user: &str, pass: &str, dur: Duration) -> FtpFact {
    let mut c = match FtpClient::connect(addr, user, pass, dur).await {
        Ok(c) => c,
        Err(FtpError::NotFtp(_) | FtpError::Timeout) => {
            // Could the TCP port at least be opened? Distinguish "closed" from "open but not FTP".
            return match timeout(dur, TcpStream::connect(addr)).await {
                Ok(Ok(_)) => FtpFact::PortOpenOnly,
                _ => FtpFact::Unreachable { detail: "no service answered on that port".into() },
            };
        }
        Err(FtpError::AuthFailed) => return FtpFact::AuthFailed,
        Err(e) => return FtpFact::Unreachable { detail: e.to_string() },
    };
    let out = match c.list("/").await {
        Ok(_) => FtpFact::Connected { writable: None },
        Err(e) => FtpFact::Unreachable { detail: format!("login worked but listing failed: {e}") },
    };
    c.quit().await;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    const T: Duration = Duration::from_millis(400);

    /// One-shot server that sends `script` then optionally stays silent.
    async fn hostile(script: Vec<u8>, linger: bool) -> SocketAddr {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
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

    #[test]
    fn parses_list_lines() {
        let e = parse_list_line("-rw-r--r--   1 ftp  ftp   1234 Jan  1 00:00 my file.pkg").unwrap();
        assert_eq!((e.name.as_str(), e.kind, e.size), ("my file.pkg", Kind::File, 1234));
        assert_eq!(parse_list_line("drwxr-xr-x 2 u g 0 Jan 1 12:00 data").unwrap().kind, Kind::Dir);
        assert_eq!(parse_list_line("lrwxrwxrwx 1 u g 4 Jan 1 12:00 a -> /etc/passwd").unwrap().name, "a");
        for bad in ["", "total 4", "-rw-r--r-- 1 u g 1 Jan 1 00:00 ../evil", "-rw-r--r-- 1 u g 1 Jan 1 00:00 a/b", "-rw-r--r-- 1 u g 1 Jan 1 00:00 ..", "garbage"] {
            assert!(parse_list_line(bad).is_none(), "{bad:?}");
        }
    }

    #[tokio::test]
    async fn non_ftp_banner_is_port_open_only() {
        let a = hostile(b"SSH-2.0-OpenSSH_9.9\r\n".to_vec(), true).await;
        assert!(matches!(FtpClient::connect(a, "a", "b", T).await, Err(FtpError::NotFtp(_))));
        let a = hostile(b"SSH-2.0-OpenSSH_9.9\r\n".to_vec(), true).await;
        assert_eq!(probe(a, "a", "b", T).await, FtpFact::PortOpenOnly);
    }

    #[tokio::test]
    async fn silent_server_times_out_as_port_open_only() {
        let a = hostile(vec![], true).await;
        assert_eq!(probe(a, "a", "b", T).await, FtpFact::PortOpenOnly);
    }

    #[tokio::test]
    async fn closed_port_is_unreachable() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let a = l.local_addr().unwrap();
        drop(l);
        assert!(matches!(probe(a, "a", "b", T).await, FtpFact::Unreachable { .. }));
    }

    #[tokio::test]
    async fn endless_line_and_endless_multiline_are_rejected() {
        let a = hostile(vec![b'A'; 200_000], false).await;
        assert!(FtpClient::connect(a, "a", "b", T).await.is_err());
        let mut s = b"220-hello\r\n".to_vec();
        for _ in 0..10_000 {
            s.extend_from_slice(b"junk\r\n");
        }
        let a = hostile(s, false).await;
        assert!(FtpClient::connect(a, "a", "b", T).await.is_err());
    }

    #[tokio::test]
    async fn rejects_command_injection() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let a = l.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut s, _) = l.accept().await.unwrap();
            s.write_all(b"220 hi\r\n").await.unwrap();
            let mut b = [0u8; 256];
            use tokio::io::AsyncReadExt;
            loop {
                let n = s.read(&mut b).await.unwrap_or(0);
                if n == 0 { return; }
                let t = String::from_utf8_lossy(&b[..n]).to_string();
                let r = if t.starts_with("USER") { "331 pw\r\n" } else if t.starts_with("PASS") { "230 ok\r\n" } else { "200 ok\r\n" };
                s.write_all(r.as_bytes()).await.unwrap();
            }
        });
        let mut c = FtpClient::connect(a, "u", "p", T).await.unwrap();
        assert!(c.mkd("/data/x\r\nDELE /system/y").await.is_err());
        // the failed injection must not have poisoned or sent anything: a normal command still works
        assert!(c.size("/data/ok").await.is_ok());
    }
}
