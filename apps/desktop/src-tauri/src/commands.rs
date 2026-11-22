//! Thin Tauri command layer. Logic lives in `ps4studio-core`; this file only validates input,
//! holds session state, and shapes JSON for the UI.

use ps4studio_core::capability::{evaluate, CapabilityEntry, Facts};
use ps4studio_core::db::{AuditEntry, Db};
use ps4studio_core::model::{
    validate_profile_input, ConsoleModel, ConsoleProfile, Firmware, FirmwareSource,
};
use ps4studio_core::paths;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    db: Mutex<Db>,
    session: Mutex<Option<Session>>,
}

struct Session {
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
        _ => false,
    }
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    if !matches!(key.as_str(), "theme" | "accent" | "first_run_done") {
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
