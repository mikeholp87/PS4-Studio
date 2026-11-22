use ps4studio_console_adapters::ftp::FtpClient;
use ps4studio_console_adapters::ops::{apply, download_to, plan, ApplyCtx, OpError, RemoteOp};
use ps4studio_console_adapters::paths::Policy;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

const T: Duration = Duration::from_secs(2);
static NO: AtomicBool = AtomicBool::new(false);

async fn client(id: &str) -> (FtpClient, std::sync::Arc<ps4studio_mock_console::SimFs>) {
    let (a, _h, fs) = ps4studio_mock_console::serve_ftp(id, "127.0.0.1:0".parse().unwrap()).await.unwrap();
    std::mem::forget(_h);
    (FtpClient::connect(a, "anonymous", "", T).await.unwrap(), fs)
}

fn ctx<'a>(dir: &'a std::path::Path) -> ApplyCtx<'a> {
    ApplyCtx { backup_dir: dir, progress: &|_| {}, cancel: &NO }
}

#[tokio::test]
async fn upload_plan_apply_verify_and_overwrite_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let local = tmp.path().join("hello.txt");
    std::fs::write(&local, b"v1").unwrap();
    let (mut c, fs) = client("transfer_ok").await;
    let policy = Policy::default();
    let op = || RemoteOp::Upload { local: local.display().to_string(), remote: "/data/homebrew/hello.txt".into() };

    let p = plan(&mut c, &policy, op()).await.unwrap();
    assert!(!p.overwrites && p.backup_remote.is_none());
    assert!(p.warnings.iter().any(|w| w.contains("not encrypted")));
    assert!(fs.files.lock().unwrap().get("/data/homebrew/hello.txt").is_none(), "plan must not change anything");
    let out = apply(&mut c, &policy, &p, &ctx(&tmp.path().join("bk"))).await.unwrap();
    assert!(out.verified && out.backup.is_none());
    assert_eq!(fs.files.lock().unwrap()["/data/homebrew/hello.txt"], b"v1");
    assert!(!fs.files.lock().unwrap().keys().any(|k| k.ends_with(".ps4studio-part")), "temp name is gone");

    std::fs::write(&local, b"version-two").unwrap();
    let p2 = plan(&mut c, &policy, op()).await.unwrap();
    assert!(p2.overwrites && p2.backup_remote.is_some());
    let out2 = apply(&mut c, &policy, &p2, &ctx(&tmp.path().join("bk"))).await.unwrap();
    assert!(out2.verified);
    assert_eq!(std::fs::read(out2.backup.unwrap()).unwrap(), b"v1", "old content saved before replacing");
    assert_eq!(fs.files.lock().unwrap()["/data/homebrew/hello.txt"], b"version-two");
}

#[tokio::test]
async fn policy_blocks_protected_and_traversal_before_any_network_write() {
    let tmp = tempfile::tempdir().unwrap();
    let local = tmp.path().join("x");
    std::fs::write(&local, b"x").unwrap();
    let (mut c, fs) = client("transfer_ok").await;
    let policy = Policy::default();
    for remote in ["/system/sys.dat", "/data/../system/sys.dat", "/user/x", "/data", "relative"] {
        let r = plan(&mut c, &policy, RemoteOp::Upload { local: local.display().to_string(), remote: remote.into() }).await;
        assert!(matches!(r, Err(OpError::Refused(_))), "{remote}: {r:?}");
    }
    assert!(plan(&mut c, &policy, RemoteOp::Delete { path: "/system/sys.dat".into(), dir: false }).await.is_err());
    assert_eq!(fs.files.lock().unwrap()["/system/sys.dat"].len(), 16);
    // a forged plan is re-checked at apply time
    let forged = ps4studio_console_adapters::ops::Plan {
        op: RemoteOp::Delete { path: "/system/sys.dat".into(), dir: false },
        changes: vec![], warnings: vec![], recovery: String::new(), backup_remote: None, overwrites: false,
    };
    assert!(apply(&mut c, &policy, &forged, &ctx(tmp.path())).await.is_err());
    assert!(fs.files.lock().unwrap().contains_key("/system/sys.dat"));
}

#[tokio::test]
async fn delete_backs_up_then_removes_and_rename_refuses_conflicts() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut c, fs) = client("transfer_ok").await;
    let policy = Policy::default();
    let p = plan(&mut c, &policy, RemoteOp::Delete { path: "/data/homebrew/readme.txt".into(), dir: false }).await.unwrap();
    assert!(p.backup_remote.is_some());
    let out = apply(&mut c, &policy, &p, &ctx(tmp.path())).await.unwrap();
    assert!(out.verified);
    assert_eq!(std::fs::read(out.backup.unwrap()).unwrap(), b"simulated file");
    assert!(!fs.files.lock().unwrap().contains_key("/data/homebrew/readme.txt"));

    fs.files.lock().unwrap().insert("/data/a".into(), vec![1]);
    fs.files.lock().unwrap().insert("/data/b".into(), vec![2]);
    let r = plan(&mut c, &policy, RemoteOp::Rename { from: "/data/a".into(), to: "/data/b".into() }).await;
    assert!(matches!(r, Err(OpError::Refused(_))));
    let p = plan(&mut c, &policy, RemoteOp::Rename { from: "/data/a".into(), to: "/data/c".into() }).await.unwrap();
    assert!(apply(&mut c, &policy, &p, &ctx(tmp.path())).await.unwrap().verified);
}

#[tokio::test]
async fn interrupted_upload_leaves_only_a_temp_name_never_the_real_file() {
    let tmp = tempfile::tempdir().unwrap();
    let local = tmp.path().join("big.bin");
    std::fs::write(&local, vec![7u8; 200_000]).unwrap();
    let (mut c, fs) = client("transfer_interrupted").await;
    let policy = Policy::default();
    let p = plan(&mut c, &policy, RemoteOp::Upload { local: local.display().to_string(), remote: "/data/big.bin".into() }).await.unwrap();
    assert!(apply(&mut c, &policy, &p, &ctx(tmp.path())).await.is_err());
    assert!(!fs.files.lock().unwrap().contains_key("/data/big.bin"));
}

#[tokio::test]
async fn download_never_overwrites_sets_no_exec_bit_and_cleans_up_on_failure() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let (mut c, _fs) = client("transfer_ok").await;
    let a = download_to(&mut c, "/data/homebrew/readme.txt", tmp.path(), &|_| {}, &NO).await.unwrap();
    let b = download_to(&mut c, "/data/homebrew/readme.txt", tmp.path(), &|_| {}, &NO).await.unwrap();
    assert_ne!(a, b);
    assert_eq!(std::fs::metadata(&a).unwrap().permissions().mode() & 0o111, 0, "not executable");
    assert!(download_to(&mut c, "/data/../etc/passwd", tmp.path(), &|_| {}, &NO).await.is_err());
    let (mut c2, _) = client("transfer_interrupted").await;
    assert!(download_to(&mut c2, "/data/homebrew/readme.txt", tmp.path(), &|_| {}, &NO).await.is_err());
    let leftovers: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert!(!leftovers.iter().any(|n| n.ends_with(".part")), "{leftovers:?}");
}
