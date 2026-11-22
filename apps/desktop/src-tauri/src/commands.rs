//! Thin Tauri command layer. Logic lives in `ps4studio-core`; this file only validates input,
//! holds session state, and shapes JSON for the UI.

use ps4studio_core::capability::{evaluate, CapabilityEntry, Facts};
use ps4studio_core::db::{AuditEntry, Db};
use ps4studio_core::model::{
    validate_profile_input, ConsoleModel, ConsoleProfile, Firmware, FirmwareSource,
};
use ps4studio_core::paths;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;

pub struct AppState {
    db: Mutex<Db>,
    session: Mutex<Option<Session>>,
    remote_play: Mutex<Option<ps4studio_remote_play::Session>>,
}

struct Session {
    address: Option<IpAddr>,
    last_probe_error: Option<String>,
    profile: ConsoleProfile,
    facts: Facts,
    scenario_id: Option<&'static str>,
}

impl AppState {
    pub fn open() -> Result<Self, String> {
        let dir = paths::data_dir().ok_or("no XDG data directory available")?;
        let db = Db::open(&dir.join("ps4-studio.db")).map_err(|e| e.to_string())?;
        Ok(Self {
            db: Mutex::new(db),
            session: Mutex::new(None),
            remote_play: Mutex::new(None),
        })
    }
}

#[derive(Serialize)]
pub struct AppInfo {
    version: &'static str,
    data_dir: Option<String>,
    config_dir: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectionView {
    profile: ConsoleProfile,
    scenario_id: Option<&'static str>,
    simulated: bool,
    capabilities: Vec<CapabilityEntry>,
}

fn view(s: &Session) -> ConnectionView {
    ConnectionView {
        profile: s.profile.clone(),
        scenario_id: s.scenario_id,
        simulated: s.facts.simulated,
        capabilities: evaluate(&s.facts),
    }
}

fn lock<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|_| "internal state poisoned".to_string())
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        data_dir: paths::data_dir().map(|p| p.display().to_string()),
        config_dir: paths::config_dir().map(|p| p.display().to_string()),
    }
}

#[tauri::command]
pub fn list_scenarios() -> Vec<ps4studio_mock_console::Scenario> {
    ps4studio_mock_console::scenarios()
}

#[tauri::command]
pub fn connect_simulator(
    state: State<AppState>,
    scenario_id: String,
) -> Result<ConnectionView, String> {
    let sc = ps4studio_mock_console::scenario(&scenario_id).ok_or("Unknown simulator scenario.")?;
    let fw = Firmware {
        value: sc.reported_firmware.map(str::to_owned),
        source: if sc.reported_firmware.is_some() {
            FirmwareSource::Reported
        } else {
            FirmwareSource::Unknown
        },
    };
    let session = Session {
        address: None,
        last_probe_error: None,
        profile: ConsoleProfile {
            id: 0,
            name: sc.console_name.clone(),
            model: ConsoleModel::Slim,
            address: None,
            firmware: fw,
            simulated: true,
            last_connected: None,
        },
        facts: sc.facts.clone(),
        scenario_id: Some(sc.id),
    };
    let out = view(&session);
    lock(&state.db)?
        .audit(
            "connect_simulator",
            sc.id,
            "simulated console selected",
            "ok",
        )
        .map_err(|e| e.to_string())?;
    *lock(&state.session)? = Some(session);
    Ok(out)
}

#[tauri::command]
pub fn disconnect(state: State<AppState>) -> Result<(), String> {
    if lock(&state.session)?.take().is_some() {
        lock(&state.db)?
            .audit("disconnect", "-", "", "ok")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_connection(state: State<AppState>) -> Result<Option<ConnectionView>, String> {
    Ok(lock(&state.session)?.as_ref().map(view))
}

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> Result<Vec<ConsoleProfile>, String> {
    lock(&state.db)?.list_profiles().map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct NewProfile {
    name: String,
    model: ConsoleModel,
    address: String,
    firmware: Option<String>,
}

#[tauri::command]
pub fn add_profile(state: State<AppState>, profile: NewProfile) -> Result<ConsoleProfile, String> {
    validate_profile_input(&profile.name, Some(&profile.address))?;
    let fw_value = profile
        .firmware
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    if fw_value
        .as_deref()
        .is_some_and(|s| s.len() > 16 || !s.chars().all(|c| c.is_ascii_digit() || c == '.'))
    {
        return Err("Firmware must look like 9.00 (digits and dots).".into());
    }
    let fw = Firmware {
        source: if fw_value.is_some() {
            FirmwareSource::UserEntered
        } else {
            FirmwareSource::Unknown
        },
        value: fw_value,
    };
    let db = lock(&state.db)?;
    let p = db
        .add_profile(
            &profile.name,
            profile.model,
            Some(profile.address.trim()),
            &fw,
            false,
        )
        .map_err(|e| e.to_string())?;
    db.audit("add_profile", &p.name, "profile created", "ok")
        .map_err(|e| e.to_string())?;
    Ok(p)
}

#[tauri::command]
pub fn delete_profile(state: State<AppState>, id: i64) -> Result<bool, String> {
    let db = lock(&state.db)?;
    let removed = db.delete_profile(id).map_err(|e| e.to_string())?;
    if removed {
        db.audit("delete_profile", &id.to_string(), "profile removed", "ok")
            .map_err(|e| e.to_string())?;
    }
    Ok(removed)
}

/// Only these keys can be read/written from the webview, each with a validator.
fn setting_ok(key: &str, value: &str) -> bool {
    match key {
        "theme" => matches!(value, "dark" | "light" | "system"),
        "accent" => {
            value.len() == 7
                && value.starts_with('#')
                && value[1..].chars().all(|c| c.is_ascii_hexdigit())
        }
        "first_run_done" => matches!(value, "0" | "1"),
        "chiaki_nickname" => ps4studio_remote_play::LaunchSpec {
            nickname: value.to_owned(),
            host: IpAddr::from([0, 0, 0, 0]),
            fullscreen: false,
        }
        .validate()
        .is_ok(),
        _ => false,
    }
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    if !matches!(
        key.as_str(),
        "theme" | "accent" | "first_run_done" | "chiaki_nickname"
    ) {
        return Err("Unknown setting.".into());
    }
    lock(&state.db)?
        .get_setting(&key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    if !setting_ok(&key, &value) {
        return Err("Invalid setting.".into());
    }
    lock(&state.db)?
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_audit(state: State<AppState>) -> Result<Vec<AuditEntry>, String> {
    lock(&state.db)?.list_audit(200).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct Discovered {
    address: String,
    name: Option<String>,
    state: ps4studio_core::capability::HostState,
    firmware_hint: Option<String>,
}

/// User-initiated limited-broadcast search (stays on the local segment).
#[tauri::command]
pub async fn discover_lan(state: State<'_, AppState>) -> Result<Vec<Discovered>, String> {
    let found = ps4studio_discovery::discover_lan(Duration::from_secs(2))
        .await
        .map_err(|e| format!("Discovery failed: {e}"))?;
    lock(&state.db)?
        .audit(
            "discover_lan",
            "broadcast:987",
            &format!("{} console(s) answered", found.len()),
            "ok",
        )
        .map_err(|e| e.to_string())?;
    Ok(found
        .into_iter()
        .map(|f| Discovered {
            address: f.address.to_string(),
            name: f.reply.host_name.clone(),
            state: f.reply.state,
            firmware_hint: f.reply.firmware_hint(),
        })
        .collect())
}

/// Probes the saved console with unicast discovery (3 tries, 1 s each) and builds verified facts.
#[tauri::command]
pub async fn connect_profile(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ConnectionView, String> {
    let mut profile = lock(&state.db)?
        .get_profile(id)
        .map_err(|e| e.to_string())?
        .ok_or("Console not found.")?;
    let ip: IpAddr = profile
        .address
        .as_deref()
        .unwrap_or_default()
        .parse()
        .map_err(|_| "Saved address is invalid.")?;
    let result = ps4studio_discovery::probe(
        SocketAddr::new(ip, ps4studio_discovery::PORT_PS4),
        Duration::from_secs(1),
        3,
    )
    .await;
    let mut facts = Facts::empty();
    facts.remote_play_binary = ps4studio_remote_play::detect().map(|b| b.describe());
    let mut err = None;
    match result {
        Ok(r) => {
            facts.discovery = Some(r.fact());
            if profile.firmware.value.is_none() {
                if let Some(h) = r.firmware_hint() {
                    profile.firmware = Firmware {
                        value: Some(h),
                        source: FirmwareSource::Reported,
                    };
                }
            }
        }
        Err(e) => err = Some(e.to_string()),
    }
    {
        let db = lock(&state.db)?;
        db.audit(
            "connect_profile",
            &profile.name,
            err.as_deref().unwrap_or("console answered discovery"),
            if err.is_some() { "no_reply" } else { "ok" },
        )
        .map_err(|e| e.to_string())?;
        if err.is_none() {
            db.touch_connected(profile.id).map_err(|e| e.to_string())?;
        }
    }
    let session = Session {
        address: Some(ip),
        last_probe_error: err,
        profile,
        facts,
        scenario_id: None,
    };
    let out = view(&session);
    *lock(&state.session)? = Some(session);
    Ok(out)
}

#[derive(Serialize)]
pub struct RemotePlayStatus {
    installed: bool,
    binary: Option<String>,
    nickname: Option<String>,
    running: bool,
}

#[tauri::command]
pub fn remote_play_status(state: State<AppState>) -> Result<RemotePlayStatus, String> {
    let bin = ps4studio_remote_play::detect();
    let running = match lock(&state.remote_play)?.as_mut() {
        Some(s) => s.try_exit().map_err(|e| e.to_string())?.is_none(),
        None => false,
    };
    Ok(RemotePlayStatus {
        installed: bin.is_some(),
        binary: bin.map(|b| b.describe()),
        nickname: lock(&state.db)?
            .get_setting("chiaki_nickname")
            .map_err(|e| e.to_string())?,
        running,
    })
}

#[tauri::command]
pub fn start_remote_play(
    state: State<AppState>,
    nickname: String,
    fullscreen: bool,
) -> Result<(), String> {
    let (ip, simulated, caps) = {
        let g = lock(&state.session)?;
        let s = g.as_ref().ok_or("Select a console first.")?;
        (s.address, s.facts.simulated, evaluate(&s.facts))
    };
    if simulated {
        return Err(
            "This is a simulated console, so no real Remote Play session is started.".into(),
        );
    }
    if let Some(e) = caps
        .iter()
        .find(|c| c.capability == ps4studio_core::capability::Capability::ConsoleRemotePlay)
    {
        if e.status != ps4studio_core::capability::Status::Available {
            return Err(e
                .reason
                .clone()
                .unwrap_or_else(|| "Remote Play is not available.".into()));
        }
    }
    let bin = ps4studio_remote_play::detect().ok_or("chiaki-ng is not installed.")?;
    let spec = ps4studio_remote_play::LaunchSpec {
        nickname: nickname.clone(),
        host: ip.ok_or("Console has no address.")?,
        fullscreen,
    };
    spec.validate().map_err(|e| e.to_string())?;
    let mut slot = lock(&state.remote_play)?;
    if let Some(s) = slot.as_mut() {
        if s.try_exit().map_err(|e| e.to_string())?.is_none() {
            return Err("A Remote Play session is already running.".into());
        }
    }
    *slot = Some(ps4studio_remote_play::Session::start(&bin, &spec).map_err(|e| e.to_string())?);
    drop(slot);
    let db = lock(&state.db)?;
    db.set_setting("chiaki_nickname", &nickname)
        .map_err(|e| e.to_string())?;
    db.audit(
        "remote_play_start",
        &nickname,
        "chiaki-ng launched (no credentials passed)",
        "ok",
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_remote_play(state: State<'_, AppState>) -> Result<(), String> {
    let taken = lock(&state.remote_play)?.take();
    if let Some(mut s) = taken {
        s.stop(Duration::from_secs(3))
            .await
            .map_err(|e| e.to_string())?;
        lock(&state.db)?
            .audit("remote_play_stop", "-", "", "ok")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize)]
pub struct Diagnostics {
    checks: Vec<ps4studio_diagnostics::Check>,
}

fn gather(state: &AppState) -> Result<Vec<ps4studio_diagnostics::Check>, String> {
    use ps4studio_diagnostics::{capability_checks, local_checks, Check, Level};
    let mut checks = local_checks(paths::data_dir().as_deref());
    if let Some(s) = lock(&state.session)?.as_ref() {
        checks.push(Check {
            id: "console",
            title: "Selected console".into(),
            level: if s.facts.discovery.is_some() { Level::Pass } else { Level::Fail },
            detail: match (&s.last_probe_error, s.facts.simulated) {
                (_, true) => "Simulated console (not real hardware).".into(),
                (Some(e), _) => format!("{e} ({})", s.address.map(|a| a.to_string()).unwrap_or_default()),
                (None, _) => "Console answered the discovery search.".into(),
            },
            hint: s.facts.discovery.is_none().then(|| "Console not found. Confirm that the Debian computer and PS4 are on the same local network, and that the PS4 is on or in Rest Mode.".to_string()),
        });
        checks.extend(capability_checks(&evaluate(&s.facts)));
    }
    Ok(checks)
}

#[tauri::command]
pub fn run_diagnostics(state: State<AppState>) -> Result<Diagnostics, String> {
    Ok(Diagnostics {
        checks: gather(&state)?,
    })
}

/// Writes a sanitised report under the XDG data dir and returns its path.
#[tauri::command]
pub fn export_diagnostics(state: State<AppState>) -> Result<String, String> {
    let checks = gather(&state)?;
    let home = std::env::var("HOME").ok();
    let text = ps4studio_diagnostics::render_report(&checks, "", home.as_deref());
    let dir = paths::data_dir()
        .ok_or("no data directory")?
        .join("reports");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("diagnostics-{}.txt", ps4studio_core::db::now()));
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    lock(&state.db)?
        .audit("export_diagnostics", "-", "sanitised report written", "ok")
        .map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::setting_ok;

    #[test]
    fn settings_are_allowlisted_and_validated() {
        assert!(setting_ok("theme", "dark"));
        assert!(!setting_ok("theme", "<script>"));
        assert!(setting_ok("accent", "#7c83ff"));
        assert!(!setting_ok("accent", "red"));
        assert!(!setting_ok("accent", "#12345g"));
        assert!(!setting_ok("db_path", "/etc/passwd"));
    }
}
