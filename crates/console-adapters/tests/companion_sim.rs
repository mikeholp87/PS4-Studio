use ps4studio_console_adapters::companion::{Client, CompanionError};
use ps4studio_core::companion_proto::*;
use ps4studio_mock_console::SIM_CODE;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

const T: Duration = Duration::from_secs(2);
const THEME: &str = include_str!("../../../packages/theme-engine/fixtures/valid/minimal.json");

async fn start(id: &str) -> std::net::SocketAddr {
    let (a, h, _) = ps4studio_mock_console::serve_companion(id, "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    std::mem::forget(h);
    a
}

#[tokio::test]
async fn pair_status_theme_lifecycle() {
    let a = start("companion").await;
    let mut c = Client::connect(a, SIM_CODE, T).await.unwrap();
    assert!(c.info.theme_management && c.info.protocol == PROTOCOL);
    assert_eq!(
        c.call("status", serde_json::json!({})).await.unwrap()["simulated"],
        true
    );
    c.put_theme("minimal", THEME).await.unwrap();
    assert_eq!(
        c.call("list_themes", serde_json::json!({})).await.unwrap()[0]["name"],
        "Minimal"
    );
    // id/body mismatch and invalid themes are refused by the companion
    assert!(c.put_theme("other", THEME).await.is_err());
    assert!(c.put_theme("minimal", "{}").await.is_err());
    assert_eq!(
        c.call("delete_theme", serde_json::json!({"id":"minimal"}))
            .await
            .unwrap()["deleted"],
        true
    );
    // diagnostics never invents hardware numbers
    let d = c.call("diagnostics", serde_json::json!({})).await.unwrap();
    assert!(d["memory"].is_null() && d["storage"].is_null());
}

#[tokio::test]
async fn commands_are_allowlisted_and_config_keys_restricted() {
    let a = start("companion").await;
    let mut c = Client::connect(a, SIM_CODE, T).await.unwrap();
    for bad in ["exec", "shell", "read_file", "write_file", "../status"] {
        assert!(
            matches!(c.call(bad, serde_json::json!({})).await, Err(CompanionError::Remote(m)) if m == "unknown_command"),
            "{bad}"
        );
    }
    assert!(c
        .call("get_config", serde_json::json!({"key":"/etc/passwd"}))
        .await
        .is_err());
    assert!(c
        .call(
            "set_config",
            serde_json::json!({"key":"show_clock","value":"1"})
        )
        .await
        .is_ok());
    assert!(c
        .call(
            "set_config",
            serde_json::json!({"key":"show_clock","value":"x".repeat(100)})
        )
        .await
        .is_err());
}

#[tokio::test]
async fn wrong_code_is_rejected_and_locks_after_three_tries() {
    let a = start("companion").await;
    for left in [2, 1, 0] {
        match Client::connect(a, "00000000", T).await {
            Err(CompanionError::PairingRejected { attempts_left }) => {
                assert_eq!(attempts_left, left)
            }
            other => panic!("{:?}", other.err()),
        }
    }
    // even the right code is refused once locked
    assert!(matches!(
        Client::connect(a, SIM_CODE, T).await,
        Err(CompanionError::PairingRejected { attempts_left: 0 })
    ));
    assert!(matches!(
        Client::connect(a, "123", T).await,
        Err(CompanionError::BadCode)
    ));
}

#[tokio::test]
async fn version_mismatch_is_reported_clearly() {
    let a = start("companion_mismatch").await;
    match Client::connect(a, SIM_CODE, T).await {
        Err(CompanionError::VersionMismatch { theirs, version }) => {
            assert_eq!((theirs, version.as_str()), (2, "0.1.0-sim"))
        }
        other => panic!("{:?}", other.err()),
    }
}

/// A rogue "companion" that doesn't know the code must not be trusted, and tampering is detected.
#[tokio::test]
async fn rogue_servers_are_detected() {
    // 1) wrong server proof
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a = l.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut s, _) = l.accept().await.unwrap();
        let h: serde_json::Value =
            serde_json::from_str(&read_frame(&mut s).await.unwrap().body).unwrap();
        let sn = random_hex(16);
        let ack = serde_json::json!({"t":"hello_ack","protocol":1,"version":"x","nonce":sn,"features":["themes"]});
        write_frame(
            &mut s,
            &Frame {
                body: ack.to_string(),
                mac: String::new(),
            },
        )
        .await
        .unwrap();
        let _ = read_frame(&mut s).await.unwrap();
        let bad = serde_json::json!({"t":"pair_ack","ok":true,"proof":server_proof("99999999", h["nonce"].as_str().unwrap(), &sn)});
        write_frame(
            &mut s,
            &Frame {
                body: bad.to_string(),
                mac: String::new(),
            },
        )
        .await
        .unwrap();
    });
    assert!(matches!(
        Client::connect(a, SIM_CODE, T).await,
        Err(CompanionError::ServerAuth)
    ));

    // 2) valid pairing, then a response with a forged MAC
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a = l.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut s, _) = l.accept().await.unwrap();
        let h: serde_json::Value =
            serde_json::from_str(&read_frame(&mut s).await.unwrap().body).unwrap();
        let (cn, sn) = (h["nonce"].as_str().unwrap().to_owned(), random_hex(16));
        let ack = serde_json::json!({"t":"hello_ack","protocol":1,"version":"x","nonce":sn,"features":[]});
        write_frame(
            &mut s,
            &Frame {
                body: ack.to_string(),
                mac: String::new(),
            },
        )
        .await
        .unwrap();
        let _ = read_frame(&mut s).await.unwrap();
        let ok =
            serde_json::json!({"t":"pair_ack","ok":true,"proof":server_proof(SIM_CODE, &cn, &sn)});
        write_frame(
            &mut s,
            &Frame {
                body: ok.to_string(),
                mac: String::new(),
            },
        )
        .await
        .unwrap();
        let _ = read_frame(&mut s).await.unwrap();
        write_frame(
            &mut s,
            &Frame {
                body: r#"{"seq":1,"ok":true,"data":{}}"#.into(),
                mac: "00".repeat(32),
            },
        )
        .await
        .unwrap();
    });
    let mut c = Client::connect(a, SIM_CODE, T).await.unwrap();
    assert!(matches!(
        c.call("status", serde_json::json!({})).await,
        Err(CompanionError::BadMac)
    ));

    // 3) oversized length prefix and silence
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a = l.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut s, _) = l.accept().await.unwrap();
        let _ = read_frame(&mut s).await;
        let _ = s.write_all(&u32::MAX.to_be_bytes()).await;
    });
    assert!(Client::connect(a, SIM_CODE, T).await.is_err());
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a = l.local_addr().unwrap();
    tokio::spawn(async move {
        let _s = l.accept().await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    });
    assert!(matches!(
        Client::connect(a, SIM_CODE, Duration::from_millis(200)).await,
        Err(CompanionError::Timeout)
    ));
}

/// The companion must refuse replayed or out-of-order requests and bad MACs.
#[tokio::test]
async fn server_rejects_replay_and_forgery() {
    let a = start("companion").await;
    // do the handshake by hand so we can replay frames
    let mut s = tokio::net::TcpStream::connect(a).await.unwrap();
    let cn = random_hex(16);
    write_frame(
        &mut s,
        &Frame {
            body: serde_json::json!({"t":"hello","protocol":1,"nonce":cn}).to_string(),
            mac: String::new(),
        },
    )
    .await
    .unwrap();
    let ack: serde_json::Value =
        serde_json::from_str(&read_frame(&mut s).await.unwrap().body).unwrap();
    let sn = ack["nonce"].as_str().unwrap().to_owned();
    write_frame(
        &mut s,
        &Frame {
            body: serde_json::json!({"t":"pair","proof":client_proof(SIM_CODE, &cn, &sn)})
                .to_string(),
            mac: String::new(),
        },
    )
    .await
    .unwrap();
    let _ = read_frame(&mut s).await.unwrap();
    let key = session_key(SIM_CODE, &cn, &sn);
    let req = seal(
        &key,
        Dir::ClientToServer,
        r#"{"seq":1,"cmd":"status","args":{}}"#.into(),
    );
    write_frame(&mut s, &req).await.unwrap();
    assert!(open(
        &key,
        Dir::ServerToClient,
        &read_frame(&mut s).await.unwrap()
    )
    .is_ok());
    write_frame(&mut s, &req).await.unwrap(); // replay of seq 1
    assert!(
        read_frame(&mut s).await.is_err(),
        "session is dropped on replay"
    );

    let mut s = tokio::net::TcpStream::connect(a).await;
    // forged MAC on a fresh session -> dropped (handshake with the real code succeeds first)
    let s2 = s.as_mut().unwrap();
    let cn = random_hex(16);
    write_frame(
        s2,
        &Frame {
            body: serde_json::json!({"t":"hello","protocol":1,"nonce":cn}).to_string(),
            mac: String::new(),
        },
    )
    .await
    .unwrap();
    let ack: serde_json::Value = serde_json::from_str(&read_frame(s2).await.unwrap().body).unwrap();
    let sn = ack["nonce"].as_str().unwrap().to_owned();
    write_frame(
        s2,
        &Frame {
            body: serde_json::json!({"t":"pair","proof":client_proof(SIM_CODE, &cn, &sn)})
                .to_string(),
            mac: String::new(),
        },
    )
    .await
    .unwrap();
    let _ = read_frame(s2).await.unwrap();
    write_frame(
        s2,
        &Frame {
            body: r#"{"seq":1,"cmd":"status","args":{}}"#.into(),
            mac: "00".repeat(32),
        },
    )
    .await
    .unwrap();
    assert!(read_frame(s2).await.is_err());
}
