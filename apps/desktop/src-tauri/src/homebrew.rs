//! Local homebrew package inventory and integrity checks. Nothing here installs or runs anything:
//! packages are only listed, hashed and (via the file manager's confirmed flow) copied to an
//! allowed console folder.

use crate::commands::{lock, AppState};
use ps4studio_console_adapters::ops::RemoteOp;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::State;

const MAX_SIDECAR: u64 = 16 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Sidecar {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
pub struct PackageInfo {
    pub file: String,
    pub size: u64,
    pub manifest: Option<Sidecar>,
    pub trusted: bool,
    pub warnings: Vec<String>,
}

fn field_ok(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}

pub fn parse_sidecar(bytes: &[u8]) -> Result<Sidecar, String> {
    let s: Sidecar =
        serde_json::from_slice(bytes).map_err(|_| "manifest is not valid".to_string())?;
    if !field_ok(&s.name, 64) || !field_ok(&s.version, 32) || !field_ok(&s.publisher, 64) {
        return Err("manifest has an invalid field".into());
    }
    if s.sha256.len() != 64 || !s.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("manifest sha256 is not a 64-digit hex value".into());
    }
    Ok(s)
}

pub fn assess(
    file: &str,
    size: u64,
    manifest: Option<Sidecar>,
    trusted_list: &[String],
) -> PackageInfo {
    let mut warnings = vec![];
    let mut trusted = false;
    match &manifest {
        None => warnings.push(
            "No manifest (.json) next to this file, so its origin and integrity cannot be checked."
                .into(),
        ),
        Some(m) => {
            trusted = trusted_list
                .iter()
                .any(|t| t.eq_ignore_ascii_case(&m.publisher));
            if !trusted {
                warnings.push(format!("Unfamiliar publisher “{}”. Only continue if you got this file from a source you trust.", m.publisher));
            }
        }
    }
    warnings.push("Installing homebrew can crash or destabilise a console. PS4 Studio never runs packages; it can only copy them to the console.".into());
    PackageInfo {
        file: file.into(),
        size,
        manifest,
        trusted,
        warnings,
    }
}

pub fn scan_dir(dir: &Path, trusted_list: &[String]) -> Vec<PackageInfo> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut out: Vec<PackageInfo> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let md = e.metadata().ok()?;
            if !md.is_file()
                || !name.to_lowercase().ends_with(".pkg")
                || !ps4studio_console_adapters::paths::valid_name(&name)
            {
                return None;
            }
            let side = dir.join(format!("{name}.json"));
            let manifest = std::fs::metadata(&side)
                .ok()
                .filter(|m| m.len() <= MAX_SIDECAR)
                .and_then(|_| std::fs::read(&side).ok())
                .and_then(|b| parse_sidecar(&b).ok());
            Some(assess(&name, md.len(), manifest, trusted_list))
        })
        .collect();
    out.sort_by_key(|a| a.file.to_lowercase());
    out
}

fn trusted(state: &AppState) -> Result<Vec<String>, String> {
    let v = lock(&state.db)?
        .get_setting("trusted_publishers")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    Ok(v.split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Lets the user pick a folder (dialog opened here) and lists the .pkg files in it.
#[tauri::command]
pub async fn hb_pick_and_scan(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<Vec<PackageInfo>>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await
            .map_err(|e| e.to_string())?;
    let Some(dir) = picked else { return Ok(None) };
    let dir = dir.into_path().map_err(|e| e.to_string())?;
    let list = scan_dir(&dir, &trusted(&state)?);
    *lock(&state.files.hb_dir)? = Some(dir);
    Ok(Some(list))
}

fn pkg_path(state: &AppState, name: &str) -> Result<PathBuf, String> {
    if !ps4studio_console_adapters::paths::valid_name(name) {
        return Err("Invalid file name.".into());
    }
    let dir = lock(&state.files.hb_dir)?
        .clone()
        .ok_or("Choose a folder first.")?;
    Ok(dir.join(name))
}

#[derive(Serialize)]
pub struct HashResult {
    sha256: String,
    status: &'static str,
}

/// Streams the file through SHA-256 on a worker thread and compares it with the manifest, if any.
#[tauri::command]
pub async fn hb_verify(state: State<'_, AppState>, name: String) -> Result<HashResult, String> {
    let path = pkg_path(&state, &name)?;
    let side = path.with_file_name(format!("{name}.json"));
    let expected = std::fs::read(&side)
        .ok()
        .and_then(|b| parse_sidecar(&b).ok())
        .map(|m| m.sha256.to_lowercase());
    let sha = tauri::async_runtime::spawn_blocking(move || -> std::io::Result<String> {
        let mut f = std::fs::File::open(path)?;
        let mut h = Sha256::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    let status = match &expected {
        None => "no_manifest",
        Some(e) if *e == sha => "verified",
        Some(_) => "mismatch",
    };
    lock(&state.db)?
        .audit("hb_verify", &name, &sha, status)
        .map_err(|e| e.to_string())?;
    Ok(HashResult {
        sha256: sha,
        status,
    })
}

/// Builds the upload operation for the confirmed flow. Only allowed when the checksum has matched
/// the manifest OR the caller acknowledges an unverified package.
#[tauri::command]
pub fn hb_upload_op(state: State<AppState>, name: String) -> Result<RemoteOp, String> {
    let path = pkg_path(&state, &name)?;
    Ok(RemoteOp::Upload {
        local: path.display().to_string(),
        remote: format!("/data/pkg/{name}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{"name":"Demo","version":"1.0","publisher":"Acme","sha256":"0000000000000000000000000000000000000000000000000000000000000000"}"#;

    #[test]
    fn sidecar_is_strict() {
        assert!(parse_sidecar(GOOD.as_bytes()).is_ok());
        assert!(parse_sidecar(br#"{"name":"x"}"#).is_err());
        assert!(parse_sidecar(
            GOOD.replace(
                "0000000000000000000000000000000000000000000000000000000000000000",
                "zz"
            )
            .as_bytes()
        )
        .is_err());
        assert!(
            parse_sidecar(GOOD.replace("}", r#","cmd":"rm -rf /"}"#).as_bytes()).is_err(),
            "unknown fields rejected"
        );
        assert!(parse_sidecar(GOOD.replace("Acme", "Ac\\u0007me").as_bytes()).is_err());
    }

    #[test]
    fn warnings_reflect_trust() {
        let m = parse_sidecar(GOOD.as_bytes()).unwrap();
        assert!(assess("a.pkg", 1, None, &[]).warnings[0].contains("No manifest"));
        assert!(
            assess("a.pkg", 1, Some(m.clone()), &[]).warnings[0].contains("Unfamiliar publisher")
        );
        let t = assess("a.pkg", 1, Some(m), &["acme".into()]);
        assert!(t.trusted && !t.warnings[0].contains("Unfamiliar"));
    }

    #[test]
    fn scan_lists_only_plain_pkg_files() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("a.pkg"), b"x").unwrap();
        std::fs::write(d.path().join("a.pkg.json"), GOOD).unwrap();
        std::fs::write(d.path().join("b.PKG"), b"yy").unwrap();
        std::fs::write(d.path().join("c.txt"), b"z").unwrap();
        std::fs::create_dir(d.path().join("sub.pkg")).unwrap();
        let l = scan_dir(d.path(), &[]);
        assert_eq!(
            l.iter().map(|p| p.file.as_str()).collect::<Vec<_>>(),
            ["a.pkg", "b.PKG"]
        );
        assert!(l[0].manifest.is_some() && l[1].manifest.is_none());
    }
}
