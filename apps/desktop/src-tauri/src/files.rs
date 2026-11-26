//! File manager backend: connect (verified by real FTP handshake), browse, plan/confirm/apply
//! changes, downloads, cancellation. Writes are locked until the user explicitly unlocks them, and
//! every change goes through `ops::plan` → user confirmation → `ops::apply` with an audit record.

use crate::commands::{lock, view, AppState, ConnectionView};
use ps4studio_backups as bk;
use ps4studio_console_adapters::ftp::{self, Entry, FtpClient};
use ps4studio_console_adapters::ops::{self, ApplyCtx, Outcome, Plan, RemoteOp};
use ps4studio_console_adapters::paths::{normalize, Policy};
use ps4studio_core::capability::FtpFact;
use ps4studio_core::paths;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

const TIMEOUT: Duration = Duration::from_secs(10);
const PLAN_TTL: Duration = Duration::from_secs(300);

#[derive(Default)]
pub struct FileState {
    client: tokio::sync::Mutex<Option<FtpClient>>,
    target: Mutex<Option<(SocketAddr, String, String)>>,
    sim_servers: Mutex<Vec<tokio::task::JoinHandle<()>>>,
    plans: Mutex<HashMap<String, (Plan, Instant)>>,
    cancel: Arc<AtomicBool>,
    counter: AtomicU64,
    pub(crate) hb_dir: Mutex<Option<PathBuf>>,
    pub(crate) companion: tokio::sync::Mutex<Option<ps4studio_console_adapters::companion::Client>>,
}

impl FileState {
    fn clear_meta(&self) {
        for h in self
            .sim_servers
            .lock()
            .map(|mut g| std::mem::take(&mut *g))
            .unwrap_or_default()
        {
            h.abort();
        }
        if let Ok(mut t) = self.target.lock() {
            *t = None;
        }
        if let Ok(mut p) = self.plans.lock() {
            p.clear();
        }
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Sync commands (blocking pool): wait for and drop FTP/companion clients.
    pub fn reset(&self) {
        self.clear_meta();
        *self.client.blocking_lock() = None;
        *self.companion.blocking_lock() = None;
    }

    /// Async commands: never use blocking_lock on the runtime — await the mutex instead.
    pub async fn reset_async(&self) {
        self.clear_meta();
        *self.client.lock().await = None;
        *self.companion.lock().await = None;
    }
}

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn files_connect(
    state: State<'_, AppState>,
    user: Option<String>,
    password: Option<String>,
) -> Result<ConnectionView, String> {
    state.files.reset_async().await;
    let (simulated, scenario, address) = {
        let g = lock(&state.session)?;
        let s = g.as_ref().ok_or("Select a console first.")?;
        (s.facts.simulated, s.scenario_id, s.address)
    };
    let (user, pass) = (
        user.filter(|u| !u.is_empty())
            .unwrap_or_else(|| "anonymous".into()),
        password.unwrap_or_default(),
    );
    let addr: SocketAddr = if simulated {
        let id = scenario.ok_or("No simulator scenario.")?;
        let scripted = ps4studio_mock_console::scenario(id)
            .map(|sc| sc.facts.ftp)
            .unwrap_or(FtpFact::NotProbed);
        if scripted == FtpFact::NotProbed {
            // This simulated console has no file service: point at a port nobody listens on.
            let l = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(err)?;
            let a = l.local_addr().map_err(err)?;
            drop(l);
            a
        } else {
            let (a, h, _fs) =
                ps4studio_mock_console::serve_ftp(id, "127.0.0.1:0".parse().map_err(err)?)
                    .await
                    .map_err(err)?;
            lock(&state.files.sim_servers)?.push(h);
            a
        }
    } else {
        let port: u16 = lock(&state.db)?
            .get_setting("ftp_port")
            .map_err(err)?
            .and_then(|p| p.parse().ok())
            .unwrap_or(2121);
        SocketAddr::new(address.ok_or("Console has no address.")?, port)
    };

    // Probe = real handshake + login + listing. An open port alone never enables anything.
    let mut fact = ftp::probe(addr, &user, &pass, TIMEOUT).await;
    if matches!(fact, FtpFact::Unreachable { .. }) {
        fact = ftp::probe(addr, &user, &pass, TIMEOUT).await; // one retry
    }
    if matches!(fact, FtpFact::Connected { .. }) {
        let c = FtpClient::connect(addr, &user, &pass, TIMEOUT)
            .await
            .map_err(err)?;
        *state.files.client.lock().await = Some(c);
        *lock(&state.files.target)? = Some((addr, user, pass));
        state.files.cancel.store(false, Ordering::Relaxed);
    }
    let outcome = match &fact {
        FtpFact::Connected { .. } => "connected",
        FtpFact::AuthFailed => "auth_failed",
        FtpFact::PortOpenOnly => "not_ftp",
        _ => "unreachable",
    };
    let mut g = lock(&state.session)?;
    let s = g.as_mut().ok_or("Console was disconnected.")?;
    s.facts.ftp = fact;
    s.facts.write_unlocked = false; // always start read-only
    let v = view(s);
    drop(g);
    lock(&state.db)?
        .audit(
            "files_connect",
            if simulated { "simulator" } else { "console" },
            "unencrypted FTP; read-only",
            outcome,
        )
        .map_err(err)?;
    Ok(v)
}

#[tauri::command]
pub async fn files_disconnect(state: State<'_, AppState>) -> Result<ConnectionView, String> {
    if let Some(c) = state.files.client.lock().await.take() {
        c.quit().await;
    }
    state.files.reset_async().await;
    let mut g = lock(&state.session)?;
    let s = g.as_mut().ok_or("Select a console first.")?;
    s.facts.ftp = FtpFact::NotProbed;
    s.facts.write_unlocked = false;
    Ok(view(s))
}

/// Opens a fresh control connection when the previous one was poisoned (cancel/failure).
async fn client<'a>(
    state: &'a AppState,
    slot: &'a mut Option<FtpClient>,
) -> Result<&'a mut FtpClient, String> {
    if slot.is_none() {
        let t = lock(&state.files.target)?
            .clone()
            .ok_or("Not connected to a file service. Connect first.")?;
        *slot = Some(
            FtpClient::connect(t.0, &t.1, &t.2, TIMEOUT)
                .await
                .map_err(err)?,
        );
    }
    slot.as_mut().ok_or_else(|| "Not connected.".to_string())
}

#[tauri::command]
pub async fn files_list_remote(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<Entry>, String> {
    let path = normalize(&path).map_err(err)?;
    let mut g = state.files.client.lock().await;
    let r = client(&state, &mut g).await?.list(&path).await;
    if r.is_err() {
        *g = None; // reconnect next time
    }
    r.map_err(err)
}

#[derive(Serialize)]
pub struct LocalListing {
    path: String,
    parent: Option<String>,
    entries: Vec<Entry>,
}

/// Read-only listing of a folder on this computer (names and sizes only).
#[tauri::command]
pub fn files_list_local(path: Option<String>) -> Result<LocalListing, String> {
    let dir = match path.filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => dirs_home().ok_or("No home directory.")?,
    };
    let dir = std::fs::canonicalize(&dir).map_err(|e| format!("Cannot open folder: {e}"))?;
    let mut entries = Vec::new();
    for e in std::fs::read_dir(&dir).map_err(err)?.flatten().take(5000) {
        let Ok(name) = e.file_name().into_string() else {
            continue;
        };
        let Ok(md) = e.metadata() else { continue };
        let kind = if md.is_dir() {
            ftp::Kind::Dir
        } else {
            ftp::Kind::File
        };
        entries.push(Entry {
            name,
            kind,
            size: if md.is_file() { md.len() } else { 0 },
        });
    }
    entries.sort_by_key(|a| (a.kind != ftp::Kind::Dir, a.name.to_lowercase()));
    Ok(LocalListing {
        path: dir.display().to_string(),
        parent: dir.parent().map(|p| p.display().to_string()),
        entries,
    })
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[tauri::command]
pub fn files_set_write_lock(
    state: State<AppState>,
    unlocked: bool,
    acknowledge_unencrypted: bool,
) -> Result<ConnectionView, String> {
    let mut g = lock(&state.session)?;
    let s = g.as_mut().ok_or("Select a console first.")?;
    if unlocked {
        if !matches!(s.facts.ftp, FtpFact::Connected { .. }) {
            return Err("Connect to the file service first.".into());
        }
        if !acknowledge_unencrypted {
            return Err(
                "You must acknowledge that FTP is unencrypted before enabling writes.".into(),
            );
        }
    }
    s.facts.write_unlocked = unlocked;
    let v = view(s);
    drop(g);
    lock(&state.db)?
        .audit(
            if unlocked {
                "unlock_writes"
            } else {
                "lock_writes"
            },
            "file service",
            "",
            "ok",
        )
        .map_err(err)?;
    Ok(v)
}

#[derive(Serialize)]
pub struct PlanView {
    plan_id: String,
    plan: Plan,
}

#[tauri::command]
pub async fn files_plan(state: State<'_, AppState>, op: RemoteOp) -> Result<PlanView, String> {
    {
        let g = lock(&state.session)?;
        let s = g.as_ref().ok_or("Select a console first.")?;
        if !matches!(s.facts.ftp, FtpFact::Connected { .. }) {
            return Err(
                "File management is unavailable because no supported file service was detected."
                    .into(),
            );
        }
        if !s.facts.write_unlocked {
            return Err(
                "Writes are locked. Unlock writes for this session to make changes.".into(),
            );
        }
    }
    let mut g = state.files.client.lock().await;
    let p = ops::plan(client(&state, &mut g).await?, &Policy::default(), op)
        .await
        .map_err(err)?;
    drop(g);
    let id = format!(
        "{:x}{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        state.files.counter.fetch_add(1, Ordering::Relaxed)
    );
    let mut plans = lock(&state.files.plans)?;
    plans.retain(|_, (_, t)| t.elapsed() < PLAN_TTL);
    plans.insert(id.clone(), (p.clone(), Instant::now()));
    Ok(PlanView {
        plan_id: id,
        plan: p,
    })
}

#[derive(Serialize, Clone)]
struct Progress {
    bytes: u64,
}

#[derive(Serialize)]
pub struct ApplyResult {
    outcome: Outcome,
    backup_id: Option<String>,
}

/// CONFIRMED apply of a stored plan. The plan is consumed (single use) and re-validated by `ops::apply`.
#[tauri::command]
pub async fn files_apply<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    plan_id: String,
) -> Result<ApplyResult, String> {
    let (plan, created) = lock(&state.files.plans)?
        .remove(&plan_id)
        .ok_or("That plan has expired or was already used. Review the change again.")?;
    if created.elapsed() > PLAN_TTL {
        return Err("That plan has expired. Review the change again.".into());
    }
    {
        let g = lock(&state.session)?;
        if !g.as_ref().is_some_and(|s| s.facts.write_unlocked) {
            return Err("Writes are locked.".into());
        }
    }
    state.files.cancel.store(false, Ordering::Relaxed);
    let tmp = std::env::temp_dir().join(format!("ps4studio-bk-{plan_id}"));
    let emit = app.clone();
    let progress = move |bytes: u64| {
        let _ = emit.emit("files-progress", Progress { bytes });
    };
    let cancel = state.files.cancel.clone();
    let mut g = state.files.client.lock().await;
    let res = {
        let c = client(&state, &mut g).await?;
        ops::apply(
            c,
            &Policy::default(),
            &plan,
            &ApplyCtx {
                backup_dir: &tmp,
                progress: &progress,
                cancel: &cancel,
            },
        )
        .await
    };
    let desc = format!("{:?}", plan.op);
    match res {
        Ok(outcome) => {
            drop(g);
            // Promote any pre-change copy into a real, verifiable backup (manifest + checksum).
            let mut backup_id = None;
            let mut promote_note = String::new();
            if let (
                Some(path),
                RemoteOp::Upload { remote, .. } | RemoteOp::Delete { path: remote, .. },
            ) = (&outcome.backup, &plan.op)
            {
                match std::fs::read(path) {
                    Ok(data) => match paths::data_dir() {
                        Some(data_dir) => match bk::create(
                            &data_dir.join("backups"),
                            &format!("Before change: {remote}"),
                            vec![bk::NewItem {
                                category: bk::Category::RemoteFile,
                                original: remote.clone(),
                                data,
                            }],
                            ps4studio_core::db::now(),
                        ) {
                            Ok(m) => {
                                backup_id = Some(m.id);
                                let _ = std::fs::remove_dir_all(&tmp);
                            }
                            Err(e) => {
                                // Console write already succeeded — keep tmp and do not imply rollback.
                                promote_note = format!(
                                    " Local recovery copy left at {} (could not promote: {e}).",
                                    tmp.display()
                                );
                            }
                        },
                        None => {
                            promote_note = format!(
                                " Local recovery copy left at {} (no data directory).",
                                tmp.display()
                            );
                        }
                    },
                    Err(e) => {
                        promote_note = format!(
                            " Local recovery copy left at {} (could not read: {e}).",
                            tmp.display()
                        );
                    }
                }
            } else {
                let _ = std::fs::remove_dir_all(&tmp);
            }
            if let Some(s) = lock(&state.session)?.as_mut() {
                if let FtpFact::Connected { writable } = &mut s.facts.ftp {
                    *writable = Some(true);
                }
            }
            let detail = format!(
                "{} | recovery: {}{promote_note}",
                outcome.lines.join("; "),
                outcome.recovery
            );
            lock(&state.db)?
                .audit(
                    "files_apply",
                    &desc,
                    &detail,
                    if outcome.verified {
                        "verified"
                    } else {
                        "unverified"
                    },
                )
                .map_err(err)?;
            Ok(ApplyResult { outcome, backup_id })
        }
        Err(e) => {
            *g = None; // connection may be poisoned; reconnect on next use
            drop(g);
            let msg = e.to_string();
            // Keep any pre-change local copy; a mid-apply failure may already have changed the console.
            let has_local = tmp
                .exists()
                .then(|| std::fs::read_dir(&tmp).ok())
                .flatten()
                .is_some_and(|mut d| d.next().is_some());
            let recovery = if has_local {
                format!(
                    "A local recovery copy may still be at {}. Check the console file list — the change may be partial. Plan recovery: {}",
                    tmp.display(),
                    plan.recovery
                )
            } else {
                format!(
                    "Check the console file list — the change may be partial. Recovery: {}",
                    plan.recovery
                )
            };
            if msg.contains("550") {
                if let Some(s) = lock(&state.session)?.as_mut() {
                    if let FtpFact::Connected { writable } = &mut s.facts.ftp {
                        *writable = Some(false);
                    }
                }
            }
            lock(&state.db)?
                .audit("files_apply", &desc, &msg, "failed")
                .map_err(err)?;
            Err(format!("{msg}. {recovery}"))
        }
    }
}

#[tauri::command]
pub fn files_cancel(state: State<AppState>) {
    state.files.cancel.store(true, Ordering::Relaxed);
}

/// Downloads a remote file into a local folder (never overwrites; no executable bit).
#[tauri::command]
pub async fn files_download<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    remote: String,
    local_dir: String,
) -> Result<String, String> {
    state.files.cancel.store(false, Ordering::Relaxed);
    let emit = app.clone();
    let progress = move |bytes: u64| {
        let _ = emit.emit("files-progress", Progress { bytes });
    };
    let cancel = state.files.cancel.clone();
    let mut g = state.files.client.lock().await;
    let res = {
        let c = client(&state, &mut g).await?;
        ops::download_to(
            c,
            &remote,
            std::path::Path::new(&local_dir),
            &progress,
            &cancel,
        )
        .await
    };
    if res.is_err() {
        *g = None;
    }
    drop(g);
    let db = lock(&state.db)?;
    match res {
        Ok(p) => {
            db.audit("files_download", &remote, &p.display().to_string(), "ok")
                .map_err(err)?;
            Ok(p.display().to_string())
        }
        Err(e) => {
            db.audit("files_download", &remote, &e.to_string(), "failed")
                .map_err(err)?;
            Err(e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Companion (authenticated, NOT encrypted). Kept here because it shares the per-console
// connection lifecycle (reset on disconnect / console switch).
// ---------------------------------------------------------------------------------------------

use ps4studio_console_adapters::companion::{Client as CompanionClient, CompanionError};
use ps4studio_core::capability::{evaluate, Capability, CompanionFact, Status};

#[tauri::command]
pub async fn companion_connect(
    state: State<'_, AppState>,
    code: String,
) -> Result<ConnectionView, String> {
    *state.files.companion.lock().await = None;
    let (simulated, scenario, address) = {
        let g = lock(&state.session)?;
        let s = g.as_ref().ok_or("Select a console first.")?;
        (s.facts.simulated, s.scenario_id, s.address)
    };
    let addr: SocketAddr = if simulated {
        let id = scenario.ok_or("No simulator scenario.")?;
        let has = matches!(
            ps4studio_mock_console::scenario(id).map(|s| s.facts.companion),
            Some(CompanionFact::Connected { .. })
        );
        if has {
            let (a, h, _) =
                ps4studio_mock_console::serve_companion(id, "127.0.0.1:0".parse().map_err(err)?)
                    .await
                    .map_err(err)?;
            lock(&state.files.sim_servers)?.push(h);
            a
        } else {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(err)?;
            let a = l.local_addr().map_err(err)?;
            drop(l);
            a
        }
    } else {
        let port: u16 = lock(&state.db)?
            .get_setting("companion_port")
            .map_err(err)?
            .and_then(|p| p.parse().ok())
            .unwrap_or(9777);
        SocketAddr::new(address.ok_or("Console has no address.")?, port)
    };
    let res = CompanionClient::connect(addr, code.trim(), TIMEOUT).await;
    let (fact, outcome, reply): (CompanionFact, &str, Result<(), String>) = match res {
        Ok(c) => {
            let f = CompanionFact::Connected {
                protocol: c.info.protocol,
                version: c.info.version.clone(),
                theme_management: c.info.theme_management,
            };
            *state.files.companion.lock().await = Some(c);
            (f, "connected", Ok(()))
        }
        Err(CompanionError::VersionMismatch { theirs, version }) => (
            CompanionFact::Connected {
                protocol: theirs,
                version,
                theme_management: false,
            },
            "version_mismatch",
            Ok(()),
        ),
        Err(
            e @ (CompanionError::PairingRejected { .. }
            | CompanionError::BadCode
            | CompanionError::ServerAuth
            | CompanionError::BadMac),
        ) => (
            CompanionFact::NotProbed,
            "pairing_failed",
            Err(e.to_string()),
        ),
        Err(e) => (
            CompanionFact::Absent,
            "unreachable",
            Err(format!(
                "No companion answered: {e}. Is the PS4 Studio companion running on the console?"
            )),
        ),
    };
    let mut g = lock(&state.session)?;
    let s = g.as_mut().ok_or("Console was disconnected.")?;
    s.facts.companion = fact;
    let v = view(s);
    drop(g);
    lock(&state.db)?
        .audit(
            "companion_connect",
            if simulated { "simulator" } else { "console" },
            "authenticated, not encrypted",
            outcome,
        )
        .map_err(err)?;
    reply.map(|()| v)
}

#[tauri::command]
pub async fn companion_send_theme(state: State<'_, AppState>, id: String) -> Result<(), String> {
    {
        let g = lock(&state.session)?;
        let s = g.as_ref().ok_or("Select a console first.")?;
        let ok = evaluate(&s.facts).iter().any(|c| {
            c.capability == Capability::CompanionThemeManagement && c.status == Status::Available
        });
        if !ok {
            return Err(
                "Theme transfer needs a paired companion that advertises theme support.".into(),
            );
        }
    }
    let theme = ps4studio_core::themes::ThemeStore::new(
        paths::data_dir().ok_or("no data directory")?.join("themes"),
    )
    .get(&id)
    .map_err(err)?
    .ok_or("Save the theme first, then send it.")?;
    let mut g = state.files.companion.lock().await;
    let c = g.as_mut().ok_or("Connect to the companion first.")?;
    let r = c.put_theme(&theme.id, &theme.json).await;
    if r.is_err() {
        *g = None; // session state is unknown after an error
    }
    drop(g);
    let db = lock(&state.db)?;
    match r {
        Ok(()) => db
            .audit(
                "companion_send_theme",
                &theme.id,
                "theme stored by the companion",
                "ok",
            )
            .map_err(err),
        Err(e) => {
            db.audit("companion_send_theme", &theme.id, &e.to_string(), "failed")
                .map_err(err)?;
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn companion_disconnect(state: State<'_, AppState>) -> Result<ConnectionView, String> {
    *state.files.companion.lock().await = None;
    let mut g = lock(&state.session)?;
    let s = g.as_mut().ok_or("Select a console first.")?;
    s.facts.companion = CompanionFact::NotProbed;
    Ok(view(s))
}
