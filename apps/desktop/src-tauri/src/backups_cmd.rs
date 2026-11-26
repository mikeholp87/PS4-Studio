//! Backup commands. Backups are local directories with a checksummed manifest (crate `backups`).
//! Restore is only offered for data PS4 Studio owns (settings, profiles, themes).

use crate::commands::{lock, setting_ok, setting_readable, AppState, SETTING_KEYS};
use ps4studio_backups as bk;
use ps4studio_core::model::{ConsoleModel, Firmware, FirmwareSource};
use ps4studio_core::{db, paths, themes::ThemeStore};
use std::path::PathBuf;
use tauri::State;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn root() -> Result<PathBuf, String> {
    Ok(paths::data_dir()
        .ok_or("no data directory")?
        .join("backups"))
}

#[tauri::command]
pub fn backup_create(state: State<AppState>, label: String) -> Result<bk::Manifest, String> {
    let label = if label.trim().is_empty() {
        "PS4 Studio backup".to_string()
    } else {
        label.trim().chars().take(60).collect()
    };
    let mut items = Vec::new();
    let d = lock(&state.db)?;
    let mut settings = serde_json::Map::new();
    for k in SETTING_KEYS {
        if let Some(v) = d.get_setting(k).map_err(err)? {
            settings.insert((*k).into(), v.into());
        }
    }
    // Per-console notes live under console_notes:* — include any that are still readable.
    for p in d.list_profiles().map_err(err)? {
        let k = format!("console_notes:profile.{}", p.id);
        if let Some(v) = d.get_setting(&k).map_err(err)? {
            settings.insert(k, v.into());
        }
    }
    items.push(bk::NewItem {
        category: bk::Category::Settings,
        original: "settings.json".into(),
        data: serde_json::to_vec_pretty(&settings).map_err(err)?,
    });
    // Connection profiles contain no secrets (credentials are never persisted).
    let profiles: Vec<_> = d
        .list_profiles()
        .map_err(err)?
        .into_iter()
        .filter(|p| !p.simulated)
        .collect();
    items.push(bk::NewItem {
        category: bk::Category::Profiles,
        original: "profiles.json".into(),
        data: serde_json::to_vec_pretty(&profiles).map_err(err)?,
    });
    for t in theme_store()?.list() {
        items.push(bk::NewItem {
            category: bk::Category::Themes,
            original: t.id,
            data: t.json.into_bytes(),
        });
    }
    let m = bk::create(&root()?, &label, items, db::now()).map_err(err)?;
    d.audit(
        "backup_create",
        &m.id,
        &format!("{} items", m.items.len()),
        "ok",
    )
    .map_err(err)?;
    Ok(m)
}

fn theme_store() -> Result<ThemeStore, String> {
    Ok(ThemeStore::new(
        paths::data_dir().ok_or("no data directory")?.join("themes"),
    ))
}

#[tauri::command]
pub fn backup_list() -> Result<Vec<bk::Manifest>, String> {
    Ok(bk::list(&root()?))
}

#[tauri::command]
pub fn backup_verify(state: State<AppState>, id: String) -> Result<bk::VerifyReport, String> {
    let r = bk::verify(&root()?, &id).map_err(err)?;
    lock(&state.db)?
        .audit(
            "backup_verify",
            &id,
            &r.problems.join("; "),
            if r.ok { "ok" } else { "damaged" },
        )
        .map_err(err)?;
    Ok(r)
}

#[tauri::command]
pub fn backup_delete(state: State<AppState>, id: String) -> Result<(), String> {
    bk::load(&root()?, &id).map_err(err)?; // validates the id and that it is a real backup
    std::fs::remove_dir_all(root()?.join(&id)).map_err(err)?;
    lock(&state.db)?
        .audit("backup_delete", &id, "", "ok")
        .map_err(err)
}

/// Restores ONE item. The item checksum is verified first; anything that would be overwritten is
/// backed up automatically before it is replaced.
#[tauri::command]
pub fn backup_restore_item(
    state: State<AppState>,
    id: String,
    index: usize,
) -> Result<String, String> {
    let root = root()?;
    let m = bk::load(&root, &id).map_err(err)?;
    let item = m.items.get(index).ok_or("No such item.")?.clone();
    let data = bk::read_item(&root, &id, index).map_err(err)?;
    let d = lock(&state.db)?;
    let msg = match item.category {
        bk::Category::Settings => {
            let map: std::collections::BTreeMap<String, String> =
                serde_json::from_slice(&data).map_err(|_| "Settings file is not valid.")?;
            let mut n = 0;
            for (k, v) in map {
                if setting_readable(&k) && setting_ok(&k, &v) {
                    d.set_setting(&k, &v).map_err(err)?;
                    n += 1;
                }
            }
            format!("Restored {n} settings. Restart the app to apply the theme.")
        }
        bk::Category::Profiles => {
            #[derive(serde::Deserialize)]
            struct P {
                name: String,
                model: ConsoleModel,
                address: Option<String>,
                firmware: Firmware,
            }
            let ps: Vec<P> =
                serde_json::from_slice(&data).map_err(|_| "Profiles file is not valid.")?;
            let existing = d.list_profiles().map_err(err)?;
            let mut added = 0;
            for p in ps {
                if ps4studio_core::model::validate_profile_input(&p.name, p.address.as_deref())
                    .is_err()
                {
                    continue;
                }
                if existing
                    .iter()
                    .any(|e| e.name == p.name && e.address == p.address)
                {
                    continue;
                }
                let fw = if p.firmware.source == FirmwareSource::Reported {
                    Firmware::default()
                } else {
                    p.firmware
                };
                d.add_profile(&p.name, p.model, p.address.as_deref(), &fw, false)
                    .map_err(err)?;
                added += 1;
            }
            format!("Added {added} saved console(s); existing ones were left unchanged.")
        }
        bk::Category::Themes => {
            let text = String::from_utf8(data).map_err(|_| "Theme is not valid text.")?;
            let store = theme_store()?;
            if let Some(cur) = store.get(&item.original).map_err(err)? {
                bk::create(
                    &root,
                    &format!("Before restore: {}", cur.id),
                    vec![bk::NewItem {
                        category: bk::Category::Themes,
                        original: cur.id,
                        data: cur.json.into_bytes(),
                    }],
                    db::now(),
                )
                .map_err(err)?;
            }
            let t = store.save(&text).map_err(err)?;
            format!(
                "Restored theme “{}”. The previous version (if any) was backed up first.",
                t.name
            )
        }
        bk::Category::RemoteFile | bk::Category::UserFile => {
            return Err("Files from a console are not restored automatically. Use “Save a copy…”, then upload it again from the File Manager (you will see exactly what changes before confirming).".into());
        }
    };
    d.audit(
        "backup_restore_item",
        &format!("{id}#{index}"),
        &item.original,
        "ok",
    )
    .map_err(err)?;
    Ok(msg)
}

/// Saves a verified copy of an item wherever the user chooses (dialog opened here, not in the webview).
#[tauri::command]
pub async fn backup_save_item_as(
    app: tauri::AppHandle,
    id: String,
    index: usize,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let data = bk::read_item(&root()?, &id, index).map_err(err)?;
    let name = bk::load(&root()?, &id).map_err(err)?.items[index]
        .stored
        .rsplit('/')
        .next()
        .unwrap_or("item")
        .to_owned();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_file_name(&name)
            .blocking_save_file()
    })
    .await
    .map_err(err)?;
    let Some(p) = picked else { return Ok(None) };
    let p = p.into_path().map_err(err)?;
    std::fs::write(&p, data).map_err(err)?;
    Ok(Some(p.display().to_string()))
}
