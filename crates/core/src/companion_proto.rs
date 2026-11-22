//! PS4 Studio companion protocol, version 1.
//!
//! Transport: TCP, frames = 4-byte big-endian length + JSON `{"body": "<json text>", "mac": "<hex>"}`.
//! Pairing: the companion shows a one-time 8-digit code on the TV. Both sides prove knowledge of it
//! with HMAC-SHA256 over fresh nonces (mutual), derive a per-connection session key, and then MAC
//! every frame (`c2s`/`s2c` direction labels + strictly increasing `seq`).
//!
//! Honest limits: this authenticates but does NOT encrypt (no TLS on the PS4 runtime), and the code
//! is only 8 digits, so a passive LAN attacker who records a pairing could brute-force it offline
//! (≈10^8 HMACs). The companion therefore shows the code for a short window, accepts it once, and
//! the UI labels the link "authenticated, not encrypted". A PAKE would fix this; see docs/security.md.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const PROTOCOL: u32 = 1;
pub const MAX_FRAME: usize = 400 * 1024;
/// The only commands a paired session may send. Everything else is refused.
pub const COMMANDS: &[&str] = &[
    "status",
    "list_themes",
    "put_theme",
    "delete_theme",
    "get_config",
    "set_config",
    "diagnostics",
];
/// Configuration keys the companion exposes. Nothing else can be read or written.
pub const CONFIG_KEYS: &[&str] = &["active_theme", "show_clock"];

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Frame {
    pub body: String,
    pub mac: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame too large")]
    TooLarge,
    #[error("malformed frame")]
    Malformed,
    #[error("message authentication failed")]
    BadMac,
}

pub fn valid_code(code: &str) -> bool {
    code.len() == 8 && code.bytes().all(|b| b.is_ascii_digit())
}

pub fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).expect("system randomness");
    to_hex(&b)
}

pub fn to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

pub fn hmac(key: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut m = HmacSha256::new_from_slice(key).expect("hmac accepts any key length");
    for p in parts {
        m.update(p);
    }
    m.finalize().into_bytes().to_vec()
}

fn verify(key: &[u8], parts: &[&[u8]], mac_hex: &str) -> bool {
    let Some(tag) = from_hex(mac_hex) else {
        return false;
    };
    let mut m = HmacSha256::new_from_slice(key).expect("key");
    for p in parts {
        m.update(p);
    }
    m.verify_slice(&tag).is_ok() // constant-time
}

/// Client's proof of knowing the code. Nonces are the hex strings exactly as exchanged.
pub fn client_proof(code: &str, cn: &str, sn: &str) -> String {
    to_hex(&hmac(
        code.as_bytes(),
        &[b"pair-client|", cn.as_bytes(), b"|", sn.as_bytes()],
    ))
}
pub fn server_proof(code: &str, cn: &str, sn: &str) -> String {
    to_hex(&hmac(
        code.as_bytes(),
        &[b"pair-server|", cn.as_bytes(), b"|", sn.as_bytes()],
    ))
}
pub fn check_client_proof(code: &str, cn: &str, sn: &str, proof: &str) -> bool {
    verify(
        code.as_bytes(),
        &[b"pair-client|", cn.as_bytes(), b"|", sn.as_bytes()],
        proof,
    )
}
pub fn check_server_proof(code: &str, cn: &str, sn: &str, proof: &str) -> bool {
    verify(
        code.as_bytes(),
        &[b"pair-server|", cn.as_bytes(), b"|", sn.as_bytes()],
        proof,
    )
}
pub fn session_key(code: &str, cn: &str, sn: &str) -> Vec<u8> {
    hmac(
        code.as_bytes(),
        &[b"session|", cn.as_bytes(), b"|", sn.as_bytes()],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    ClientToServer,
    ServerToClient,
}
impl Dir {
    fn label(self) -> &'static [u8] {
        match self {
            Dir::ClientToServer => b"c2s|",
            Dir::ServerToClient => b"s2c|",
        }
    }
}

pub fn seal(key: &[u8], dir: Dir, body: String) -> Frame {
    let mac = to_hex(&hmac(key, &[dir.label(), body.as_bytes()]));
    Frame { body, mac }
}

pub fn open(key: &[u8], dir: Dir, f: &Frame) -> Result<(), ProtoError> {
    if verify(key, &[dir.label(), f.body.as_bytes()], &f.mac) {
        Ok(())
    } else {
        Err(ProtoError::BadMac)
    }
}

pub async fn write_frame<W: AsyncWriteExt + Unpin>(w: &mut W, f: &Frame) -> Result<(), ProtoError> {
    let json = serde_json::to_vec(f).map_err(|_| ProtoError::Malformed)?;
    if json.len() > MAX_FRAME {
        return Err(ProtoError::TooLarge);
    }
    w.write_all(&(json.len() as u32).to_be_bytes()).await?;
    w.write_all(&json).await?;
    Ok(())
}

/// Reads one frame; the length is checked BEFORE any allocation.
pub async fn read_frame<R: AsyncReadExt + Unpin>(r: &mut R) -> Result<Frame, ProtoError> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len).await?;
    let n = u32::from_be_bytes(len) as usize;
    if n == 0 || n > MAX_FRAME {
        return Err(ProtoError::TooLarge);
    }
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf).map_err(|_| ProtoError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_matches_rfc4231_case_2() {
        // key = "Jefe", data = "what do ya want for nothing?"
        let t = to_hex(&hmac(b"Jefe", &[b"what do ya want for nothing?"]));
        assert_eq!(
            t,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn pairing_proofs_are_mutual_and_bound_to_nonces() {
        let (c, cn, sn) = ("12345678", "aa", "bb");
        let p = client_proof(c, cn, sn);
        assert!(check_client_proof(c, cn, sn, &p));
        assert!(!check_client_proof("12345679", cn, sn, &p), "wrong code");
        assert!(
            !check_client_proof(c, cn, "cc", &p),
            "replay with other nonce"
        );
        assert!(
            !check_client_proof(c, cn, sn, &server_proof(c, cn, sn)),
            "roles are not interchangeable"
        );
        assert!(!check_client_proof(c, cn, sn, "zz") && !check_client_proof(c, cn, sn, ""));
    }

    #[test]
    fn frames_authenticate_direction_and_content() {
        let k = session_key("12345678", "aa", "bb");
        let f = seal(
            &k,
            Dir::ClientToServer,
            r#"{"seq":1,"cmd":"status","args":{}}"#.into(),
        );
        assert!(open(&k, Dir::ClientToServer, &f).is_ok());
        assert!(
            open(&k, Dir::ServerToClient, &f).is_err(),
            "reflection attack"
        );
        let mut t = f.clone();
        t.body = t.body.replace("status", "put_theme");
        assert!(open(&k, Dir::ClientToServer, &t).is_err());
        assert!(open(
            &session_key("00000000", "aa", "bb"),
            Dir::ClientToServer,
            &f
        )
        .is_err());
    }

    #[tokio::test]
    async fn frame_io_roundtrip_and_limits() {
        let f = Frame {
            body: "x".repeat(1000),
            mac: "ab".into(),
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &f).await.unwrap();
        assert_eq!(read_frame(&mut buf.as_slice()).await.unwrap(), f);
        // oversized declared length is refused without allocating it
        let huge = (MAX_FRAME as u32 + 1).to_be_bytes();
        assert!(matches!(
            read_frame(&mut huge.as_slice()).await,
            Err(ProtoError::TooLarge)
        ));
        assert!(matches!(
            read_frame(&mut [0u8, 0, 0, 0].as_slice()).await,
            Err(ProtoError::TooLarge)
        ));
        // garbage JSON / truncated body
        let mut g = 5u32.to_be_bytes().to_vec();
        g.extend_from_slice(b"nope!");
        assert!(matches!(
            read_frame(&mut g.as_slice()).await,
            Err(ProtoError::Malformed)
        ));
        let mut t = 100u32.to_be_bytes().to_vec();
        t.extend_from_slice(b"{}");
        assert!(read_frame(&mut t.as_slice()).await.is_err());
    }

    /// Same constants the C implementation (apps/ps4-companion) is tested against; they were
    /// computed independently with Python, so Rust == C == Python.
    #[test]
    fn matches_cross_language_vectors() {
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../../apps/ps4-companion/tests/vectors.json"
        ))
        .unwrap();
        let (code, cn, sn) = (
            v["code"].as_str().unwrap(),
            v["cn"].as_str().unwrap(),
            v["sn"].as_str().unwrap(),
        );
        assert_eq!(client_proof(code, cn, sn), v["client_proof"]);
        assert_eq!(server_proof(code, cn, sn), v["server_proof"]);
        let k = session_key(code, cn, sn);
        assert_eq!(to_hex(&k), v["session_key"]);
        let f = seal(
            &k,
            Dir::ClientToServer,
            v["body"].as_str().unwrap().to_owned(),
        );
        assert_eq!(f.mac, v["c2s_mac"]);
    }

    #[test]
    fn code_format() {
        assert!(
            valid_code("00000000")
                && !valid_code("1234567")
                && !valid_code("1234567a")
                && !valid_code("123456789")
        );
        assert_ne!(random_hex(16), random_hex(16));
    }
}
