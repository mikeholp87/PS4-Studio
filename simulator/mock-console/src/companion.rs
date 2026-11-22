//! Simulated PS4 Studio companion speaking protocol v1. Behaves like the documented real one:
//! one-time code, 3 pairing attempts then locked, allowlisted commands, MAC + sequence checks.

use crate::scenario;
use ps4studio_core::capability::CompanionFact;
use ps4studio_core::companion_proto::*;
use ps4studio_core::themes;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

/// The simulator's "TV screen" code (a real companion shows a fresh random code).
pub const SIM_CODE: &str = "24681357";
const MAX_ATTEMPTS: u32 = 3;
const MAX_THEMES: usize = 16;

pub struct SimCompanion {
    pub themes: Mutex<BTreeMap<String, String>>,
    config: Mutex<BTreeMap<String, String>>,
    attempts: Mutex<u32>,
    protocol: u32,
}

pub async fn serve_companion(
    scenario_id: &str,
    bind: SocketAddr,
) -> std::io::Result<(SocketAddr, tokio::task::JoinHandle<()>, Arc<SimCompanion>)> {
    let sc = scenario(scenario_id).ok_or_else(|| std::io::Error::other("unknown scenario"))?;
    let protocol = match sc.facts.companion {
        CompanionFact::Connected { protocol, .. } => protocol,
        _ => return Err(std::io::Error::other("this scenario has no companion")),
    };
    let st = Arc::new(SimCompanion {
        themes: Default::default(),
        config: Default::default(),
        attempts: Mutex::new(0),
        protocol,
    });
    let l = TcpListener::bind(bind).await?;
    let addr = l.local_addr()?;
    let st2 = st.clone();
    let h = tokio::spawn(async move {
        loop {
            let Ok((s, _)) = l.accept().await else { return };
            let st = st2.clone();
            tokio::spawn(async move {
                let _ = conn(s, st).await;
            });
        }
    });
    Ok((addr, h, st))
}

fn parse(f: &Frame) -> Option<Value> {
    serde_json::from_str(&f.body).ok()
}

async fn conn(mut s: TcpStream, st: Arc<SimCompanion>) -> Result<(), ProtoError> {
    let hello = parse(&read_frame(&mut s).await?).ok_or(ProtoError::Malformed)?;
    if hello["t"] != "hello" {
        return Err(ProtoError::Malformed);
    }
    let cn = hello["nonce"]
        .as_str()
        .ok_or(ProtoError::Malformed)?
        .to_owned();
    let sn = random_hex(16);
    let ack = json!({"t":"hello_ack","protocol":st.protocol,"version":"0.1.0-sim","nonce":sn,"features":["themes"],"simulated":true});
    write_frame(
        &mut s,
        &Frame {
            body: ack.to_string(),
            mac: String::new(),
        },
    )
    .await?;
    if st.protocol != PROTOCOL {
        return Ok(()); // a companion speaking another version ends the session
    }
    let pair = parse(&read_frame(&mut s).await?).ok_or(ProtoError::Malformed)?;
    let proof = pair["proof"].as_str().unwrap_or_default();
    let ok = {
        let mut a = st.attempts.lock().unwrap();
        if *a >= MAX_ATTEMPTS {
            false
        } else if check_client_proof(SIM_CODE, &cn, &sn, proof) {
            true
        } else {
            *a += 1;
            false
        }
    };
    let left = MAX_ATTEMPTS.saturating_sub(*st.attempts.lock().unwrap());
    if !ok {
        let body = json!({"t":"pair_ack","ok":false,"attempts_left":left});
        write_frame(
            &mut s,
            &Frame {
                body: body.to_string(),
                mac: String::new(),
            },
        )
        .await?;
        return Ok(());
    }
    let body = json!({"t":"pair_ack","ok":true,"proof":server_proof(SIM_CODE, &cn, &sn)});
    write_frame(
        &mut s,
        &Frame {
            body: body.to_string(),
            mac: String::new(),
        },
    )
    .await?;
    let key = session_key(SIM_CODE, &cn, &sn);
    let mut last_seq = 0u64;
    loop {
        let f = read_frame(&mut s).await?;
        open(&key, Dir::ClientToServer, &f)?; // bad MAC ends the session
        let req = parse(&f).ok_or(ProtoError::Malformed)?;
        let seq = req["seq"].as_u64().ok_or(ProtoError::Malformed)?;
        if seq != last_seq + 1 {
            return Err(ProtoError::BadMac); // replay / reorder
        }
        last_seq = seq;
        let cmd = req["cmd"].as_str().unwrap_or_default();
        let resp = if !COMMANDS.contains(&cmd) {
            json!({"seq":seq,"ok":false,"error":"unknown_command"})
        } else {
            match handle(&st, cmd, &req["args"]) {
                Ok(d) => json!({"seq":seq,"ok":true,"data":d}),
                Err(e) => json!({"seq":seq,"ok":false,"error":e}),
            }
        };
        write_frame(&mut s, &seal(&key, Dir::ServerToClient, resp.to_string())).await?;
    }
}

fn handle(st: &SimCompanion, cmd: &str, args: &Value) -> Result<Value, String> {
    match cmd {
        "status" => Ok(json!({"version":"0.1.0-sim","protocol":st.protocol,"theme_count":st.themes.lock().unwrap().len(),"simulated":true})),
        "list_themes" => Ok(json!(st.themes.lock().unwrap().iter().map(|(id, j)| json!({"id":id,"name":serde_json::from_str::<Value>(j).ok().and_then(|v| v["name"].as_str().map(str::to_owned))})).collect::<Vec<_>>())),
        "put_theme" => {
            let id = args["id"].as_str().ok_or("missing id")?;
            let text = args["json"].as_str().ok_or("missing json")?;
            let v = themes::validate(text).map_err(|e| format!("invalid theme: {}", e.join("; ")))?;
            if v["id"] != id || !themes::is_valid_id(id) {
                return Err("theme id mismatch".into());
            }
            let mut t = st.themes.lock().unwrap();
            if !t.contains_key(id) && t.len() >= MAX_THEMES {
                return Err("storage full".into());
            }
            t.insert(id.to_owned(), text.to_owned());
            Ok(json!({"stored":id}))
        }
        "delete_theme" => {
            let id = args["id"].as_str().ok_or("missing id")?;
            Ok(json!({"deleted": st.themes.lock().unwrap().remove(id).is_some()}))
        }
        "get_config" | "set_config" => {
            let k = args["key"].as_str().ok_or("missing key")?;
            if !CONFIG_KEYS.contains(&k) {
                return Err("unknown config key".into());
            }
            if cmd == "set_config" {
                let v = args["value"].as_str().ok_or("missing value")?;
                if v.len() > 64 {
                    return Err("value too long".into());
                }
                st.config.lock().unwrap().insert(k.to_owned(), v.to_owned());
            }
            Ok(json!({"key":k,"value":st.config.lock().unwrap().get(k)}))
        }
        // Honest: the simulator reports no hardware numbers at all.
        "diagnostics" => Ok(json!({"version":"0.1.0-sim","protocol":st.protocol,"memory":null,"storage":null,"note":"simulated; no hardware readings"})),
        _ => Err("unknown_command".into()),
    }
}
