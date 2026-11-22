//! Desktop-side client for the PS4 Studio companion (protocol in `core::companion_proto`).

use ps4studio_core::companion_proto::*;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, thiserror::Error)]
pub enum CompanionError {
    #[error("the companion did not respond in time")]
    Timeout,
    #[error("network error: {0}")]
    Io(#[from] std::io::Error),
    #[error("companion {version} speaks protocol {theirs}; this app needs {PROTOCOL}. Update one of them.")]
    VersionMismatch { theirs: u32, version: String },
    #[error("the pairing code was rejected ({attempts_left} attempts left)")]
    PairingRejected { attempts_left: u32 },
    #[error("the companion could not prove it knows the pairing code")]
    ServerAuth,
    #[error("the companion sent a message that failed authentication")]
    BadMac,
    #[error("unexpected companion response: {0}")]
    Protocol(String),
    #[error("the companion refused the request: {0}")]
    Remote(String),
    #[error("the pairing code must be 8 digits")]
    BadCode,
}

impl From<ProtoError> for CompanionError {
    fn from(e: ProtoError) -> Self {
        match e {
            ProtoError::Io(e) => CompanionError::Io(e),
            ProtoError::BadMac => CompanionError::BadMac,
            other => CompanionError::Protocol(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub version: String,
    pub protocol: u32,
    pub theme_management: bool,
}

pub struct Client {
    s: TcpStream,
    key: Vec<u8>,
    seq: u64,
    dur: Duration,
    pub info: Info,
}

async fn rd(s: &mut TcpStream, dur: Duration) -> Result<Frame, CompanionError> {
    timeout(dur, read_frame(s))
        .await
        .map_err(|_| CompanionError::Timeout)?
        .map_err(Into::into)
}

impl Client {
    pub async fn connect(
        addr: SocketAddr,
        code: &str,
        dur: Duration,
    ) -> Result<Self, CompanionError> {
        if !valid_code(code) {
            return Err(CompanionError::BadCode);
        }
        let mut s = timeout(dur, TcpStream::connect(addr))
            .await
            .map_err(|_| CompanionError::Timeout)??;
        let cn = random_hex(16);
        let hello = json!({"t":"hello","protocol":PROTOCOL,"nonce":cn});
        write_frame(
            &mut s,
            &Frame {
                body: hello.to_string(),
                mac: String::new(),
            },
        )
        .await?;
        let ack: Value = serde_json::from_str(&rd(&mut s, dur).await?.body)
            .map_err(|_| CompanionError::Protocol("bad hello_ack".into()))?;
        if ack["t"] != "hello_ack" {
            return Err(CompanionError::Protocol("bad hello_ack".into()));
        }
        let theirs = ack["protocol"].as_u64().unwrap_or(0) as u32;
        let version: String = ack["version"]
            .as_str()
            .unwrap_or("?")
            .chars()
            .filter(|c| !c.is_control())
            .take(32)
            .collect();
        if theirs != PROTOCOL {
            return Err(CompanionError::VersionMismatch { theirs, version });
        }
        let sn = ack["nonce"]
            .as_str()
            .filter(|n| n.len() == 32 && from_hex(n).is_some())
            .ok_or_else(|| CompanionError::Protocol("bad nonce".into()))?
            .to_owned();
        let pair = json!({"t":"pair","proof":client_proof(code, &cn, &sn)});
        write_frame(
            &mut s,
            &Frame {
                body: pair.to_string(),
                mac: String::new(),
            },
        )
        .await?;
        let pa: Value = serde_json::from_str(&rd(&mut s, dur).await?.body)
            .map_err(|_| CompanionError::Protocol("bad pair_ack".into()))?;
        if pa["ok"] != true {
            return Err(CompanionError::PairingRejected {
                attempts_left: pa["attempts_left"].as_u64().unwrap_or(0) as u32,
            });
        }
        if !check_server_proof(code, &cn, &sn, pa["proof"].as_str().unwrap_or_default()) {
            return Err(CompanionError::ServerAuth);
        }
        let features = ack["features"]
            .as_array()
            .map(|a| a.iter().any(|f| f == "themes"))
            .unwrap_or(false);
        Ok(Self {
            s,
            key: session_key(code, &cn, &sn),
            seq: 0,
            dur,
            info: Info {
                version,
                protocol: theirs,
                theme_management: features,
            },
        })
    }

    pub async fn call(&mut self, cmd: &str, args: Value) -> Result<Value, CompanionError> {
        self.seq += 1;
        let body = json!({"seq":self.seq,"cmd":cmd,"args":args}).to_string();
        write_frame(&mut self.s, &seal(&self.key, Dir::ClientToServer, body)).await?;
        let f = rd(&mut self.s, self.dur).await?;
        open(&self.key, Dir::ServerToClient, &f)?;
        let v: Value = serde_json::from_str(&f.body)
            .map_err(|_| CompanionError::Protocol("bad response".into()))?;
        if v["seq"].as_u64() != Some(self.seq) {
            return Err(CompanionError::Protocol("sequence mismatch".into()));
        }
        if v["ok"] == true {
            Ok(v["data"].clone())
        } else {
            Err(CompanionError::Remote(
                v["error"]
                    .as_str()
                    .unwrap_or("error")
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(120)
                    .collect(),
            ))
        }
    }

    pub async fn put_theme(&mut self, id: &str, json_text: &str) -> Result<(), CompanionError> {
        self.call("put_theme", json!({"id":id,"json":json_text}))
            .await
            .map(drop)
    }
}
