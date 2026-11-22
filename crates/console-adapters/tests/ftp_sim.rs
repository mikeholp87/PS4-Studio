//! FTP client against the simulator's FTP service, one test per documented scenario.
use ps4studio_console_adapters::ftp::{probe, FtpClient, FtpError, Kind};
use ps4studio_core::capability::FtpFact;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

const T: Duration = Duration::from_secs(2);
static NO: AtomicBool = AtomicBool::new(false);

async fn start(id: &str) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let (a, h, _) = ps4studio_mock_console::serve_ftp(id, "127.0.0.1:0".parse().unwrap()).await.unwrap();
    (a, h)
}

#[tokio::test]
async fn probe_verifies_handshake_and_listing_but_never_claims_write() {
    let (a, _h) = start("homebrew").await;
    assert_eq!(probe(a, "anonymous", "", T).await, FtpFact::Connected { writable: None });
}

#[tokio::test]
async fn auth_failure_scenario() {
    let (a, _h) = start("ftp_auth_failure").await;
    assert_eq!(probe(a, "u", "bad", T).await, FtpFact::AuthFailed);
    assert!(matches!(FtpClient::connect(a, "u", "bad", T).await, Err(FtpError::AuthFailed)));
}

#[tokio::test]
async fn browse_transfer_roundtrip_rename_delete() {
    let (a, _h) = start("transfer_ok").await;
    let mut c = FtpClient::connect(a, "anonymous", "", T).await.unwrap();
    let root = c.list("/").await.unwrap();
    assert!(root.iter().any(|e| e.name == "data" && e.kind == Kind::Dir));
    let data = c.list("/data/homebrew").await.unwrap();
    assert_eq!(data[0].name, "readme.txt");

    let payload: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
    let seen = std::sync::atomic::AtomicU64::new(0);
    let prog = |n: u64| seen.store(n, std::sync::atomic::Ordering::Relaxed);
    let up = c.upload(&mut payload.as_slice(), "/data/pkg/a.bin", &prog, &NO).await.unwrap();
    assert_eq!(up, payload.len() as u64);
    assert_eq!(seen.load(std::sync::atomic::Ordering::Relaxed), payload.len() as u64);
    assert_eq!(c.size("/data/pkg/a.bin").await.unwrap(), Some(payload.len() as u64));

    let mut back = Vec::new();
    c.download("/data/pkg/a.bin", &mut back, &|_| {}, &NO).await.unwrap();
    assert_eq!(back, payload, "byte-exact roundtrip");

    c.rename("/data/pkg/a.bin", "/data/pkg/b.bin").await.unwrap();
    assert_eq!(c.size("/data/pkg/a.bin").await.unwrap(), None);
    c.mkd("/data/newdir").await.unwrap();
    c.dele("/data/pkg/b.bin").await.unwrap();
    c.rmd("/data/newdir").await.unwrap();
    assert!(matches!(c.dele("/data/pkg/b.bin").await, Err(FtpError::Rejected { code: 550, .. })));
    c.quit().await;
}

#[tokio::test]
async fn unauthorized_writes_are_refused_by_the_service() {
    let (a, _h) = start("unauthorized").await;
    let mut c = FtpClient::connect(a, "anonymous", "", T).await.unwrap();
    assert!(c.list("/data").await.is_ok(), "reads still work");
    let r = c.upload(&mut &b"x"[..], "/data/x", &|_| {}, &NO).await;
    assert!(matches!(r, Err(FtpError::Rejected { code: 550, .. })), "{r:?}");
    assert!(matches!(c.mkd("/data/d").await, Err(FtpError::Rejected { code: 550, .. })));
}

#[tokio::test]
async fn storage_limit_refuses_big_upload_but_allows_small() {
    let (a, _h) = start("storage_limit").await;
    let mut c = FtpClient::connect(a, "anonymous", "", T).await.unwrap();
    let big = vec![1u8; 2 * 1024 * 1024];
    let r = c.upload(&mut big.as_slice(), "/data/big", &|_| {}, &NO).await;
    assert!(matches!(r, Err(FtpError::Rejected { code: 552, .. })), "{r:?}");
    assert!(c.upload(&mut &b"tiny"[..], "/data/small", &|_| {}, &NO).await.is_ok());
}

#[tokio::test]
async fn interrupted_transfer_fails_and_poisons_the_connection() {
    let (a, _h) = start("transfer_interrupted").await;
    let mut c = FtpClient::connect(a, "anonymous", "", T).await.unwrap();
    let mut out = Vec::new();
    let r = c.download("/data/homebrew/readme.txt", &mut out, &|_| {}, &NO).await;
    assert!(r.is_err(), "short or dropped transfer must not look like success: {r:?}");
    assert!(c.list("/").await.is_err(), "client refuses reuse after a failed transfer");
}

#[tokio::test]
async fn cancel_stops_a_transfer_and_prevents_reuse() {
    let (a, _h) = start("transfer_ok").await;
    let mut c = FtpClient::connect(a, "anonymous", "", T).await.unwrap();
    let cancel = AtomicBool::new(true);
    let big = vec![0u8; 1_000_000];
    let r = c.upload(&mut big.as_slice(), "/data/x", &|_| {}, &cancel).await;
    assert!(matches!(r, Err(FtpError::Cancelled)));
    assert!(c.size("/data/x").await.is_err());
}
