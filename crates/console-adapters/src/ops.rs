//! Safe remote operations: INSPECT → PLAN → (user CONFIRM) → APPLY → VERIFY → RECOVER.
//! `plan` only reads. `apply` takes a `Plan` produced by `plan`, re-checks policy, backs up what it
//! is about to overwrite or delete, writes uploads to a temporary name first, and verifies after.

use crate::ftp::{FtpClient, FtpError};
use crate::paths::{normalize, valid_name, Policy};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

pub const MAX_BACKUP_BYTES: u64 = 64 * 1024 * 1024;
const PART_SUFFIX: &str = ".ps4studio-part";
/// Temporary name for the previous file during overwrite (rename-swap; never DELE-before-RNTO).
const BAK_SUFFIX: &str = ".ps4studio-bak";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemoteOp {
    Upload { local: String, remote: String },
    Mkdir { path: String },
    Delete { path: String, dir: bool },
    Rename { from: String, to: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub op: RemoteOp,
    /// What will change, one line each, shown before the user confirms.
    pub changes: Vec<String>,
    pub warnings: Vec<String>,
    pub recovery: String,
    pub backup_remote: Option<String>,
    pub overwrites: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum OpError {
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Ftp(#[from] FtpError),
    #[error("local file problem: {0}")]
    Local(#[from] std::io::Error),
}

impl From<crate::paths::PathError> for OpError {
    fn from(e: crate::paths::PathError) -> Self {
        OpError::Refused(e.to_string())
    }
}

fn parent_and_name(p: &str) -> (String, String) {
    match p.rsplit_once('/') {
        Some(("", n)) => ("/".into(), n.into()),
        Some((d, n)) => (d.into(), n.into()),
        None => ("/".into(), p.into()),
    }
}

async fn exists(c: &mut FtpClient, path: &str) -> Result<bool, OpError> {
    if c.size(path).await?.is_some() {
        return Ok(true);
    }
    let (dir, name) = parent_and_name(path);
    Ok(c.list(&dir).await?.iter().any(|e| e.name == name))
}

/// INSPECT + PLAN. Never modifies anything.
pub async fn plan(c: &mut FtpClient, policy: &Policy, op: RemoteOp) -> Result<Plan, OpError> {
    match op {
        RemoteOp::Upload { local, remote } => {
            let remote = policy.check_write(&remote)?;
            if remote.ends_with(PART_SUFFIX) || !valid_name(&parent_and_name(&remote).1) {
                return Err(OpError::Refused("Invalid destination file name.".into()));
            }
            let meta = std::fs::metadata(&local)?;
            if !meta.is_file() {
                return Err(OpError::Refused(
                    "Only regular files can be uploaded.".into(),
                ));
            }
            let existing = c.size(&remote).await?;
            let mut warnings = vec!["FTP is not encrypted: the file and any login travel in clear text on your network.".to_string()];
            let mut changes = vec![format!("Upload {local} ({} bytes) to {remote}", meta.len())];
            let backup_remote = match existing {
                Some(sz) if sz <= MAX_BACKUP_BYTES => {
                    changes.push(format!("Replace the existing {sz}-byte file (a copy is saved on this computer first)."));
                    Some(remote.clone())
                }
                Some(sz) => {
                    warnings.push(format!("The existing file is {sz} bytes, over the {} MB backup limit. It will be replaced WITHOUT a backup.", MAX_BACKUP_BYTES / 1024 / 1024));
                    None
                }
                None => None,
            };
            Ok(Plan {
                recovery: if backup_remote.is_some() {
                    "Restore the previous version from the saved backup copy, or delete the uploaded file.".into()
                } else {
                    format!("To undo, delete {remote} on the console.")
                },
                op: RemoteOp::Upload { local, remote },
                changes,
                warnings,
                overwrites: existing.is_some(),
                backup_remote,
            })
        }
        RemoteOp::Mkdir { path } => {
            let path = policy.check_write(&path)?;
            if exists(c, &path).await? {
                return Err(OpError::Refused("That folder already exists.".into()));
            }
            Ok(Plan {
                changes: vec![format!("Create folder {path}")],
                warnings: vec![],
                recovery: format!("To undo, remove the empty folder {path}."),
                backup_remote: None,
                overwrites: false,
                op: RemoteOp::Mkdir { path },
            })
        }
        RemoteOp::Delete { path, dir } => {
            let path = policy.check_write(&path)?;
            let size = if dir { None } else { c.size(&path).await? };
            if !dir && size.is_none() {
                return Err(OpError::Refused("That file does not exist.".into()));
            }
            let mut warnings = vec!["Deleting cannot be undone on the console.".to_string()];
            let backup_remote = match size {
                Some(sz) if sz <= MAX_BACKUP_BYTES => Some(path.clone()),
                Some(_) => {
                    warnings.push(
                        "The file is over the backup limit and will be deleted WITHOUT a backup."
                            .into(),
                    );
                    None
                }
                None => {
                    warnings.push("Only empty folders can be removed.".into());
                    None
                }
            };
            Ok(Plan {
                recovery: if backup_remote.is_some() {
                    "Re-upload the file from the saved backup copy.".into()
                } else {
                    "No automatic recovery is available.".into()
                },
                changes: vec![format!(
                    "Delete {} {path}",
                    if dir { "folder" } else { "file" }
                )],
                warnings,
                backup_remote,
                overwrites: false,
                op: RemoteOp::Delete { path, dir },
            })
        }
        RemoteOp::Rename { from, to } => {
            let (from, to) = (policy.check_write(&from)?, policy.check_write(&to)?);
            if !exists(c, &from).await? {
                return Err(OpError::Refused(
                    "The item to rename does not exist.".into(),
                ));
            }
            if exists(c, &to).await? {
                return Err(OpError::Refused(
                    "An item with the new name already exists.".into(),
                ));
            }
            Ok(Plan {
                changes: vec![format!("Rename {from} to {to}")],
                warnings: vec![],
                recovery: format!("To undo, rename {to} back to {from}."),
                backup_remote: None,
                overwrites: false,
                op: RemoteOp::Rename { from, to },
            })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub lines: Vec<String>,
    pub backup: Option<PathBuf>,
    pub verified: bool,
    pub recovery: String,
}

pub struct ApplyCtx<'a> {
    pub backup_dir: &'a Path,
    pub progress: &'a (dyn Fn(u64) + Sync),
    pub cancel: &'a AtomicBool,
}

async fn backup_remote(
    c: &mut FtpClient,
    remote: &str,
    dir: &Path,
    cancel: &AtomicBool,
) -> Result<PathBuf, OpError> {
    let rel = normalize(remote)?;
    let dest = dir.join(rel.trim_start_matches('/'));
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut f = tokio::fs::File::create(&dest).await?;
    c.download(remote, &mut f, &|_| {}, cancel).await?;
    Ok(dest)
}

/// APPLY + VERIFY. Re-validates the plan against policy (a plan is data, not authority).
pub async fn apply(
    c: &mut FtpClient,
    policy: &Policy,
    plan: &Plan,
    ctx: &ApplyCtx<'_>,
) -> Result<Outcome, OpError> {
    let mut lines = Vec::new();
    let mut backup = None;
    // Non-upload ops still honour plan.backup_remote (delete). Uploads re-inspect below.
    if !matches!(plan.op, RemoteOp::Upload { .. }) {
        if let Some(r) = &plan.backup_remote {
            policy.check_write(r)?;
            let b = backup_remote(c, r, ctx.backup_dir, ctx.cancel).await?;
            lines.push(format!(
                "Saved a copy of the existing file to {}",
                b.display()
            ));
            backup = Some(b);
        }
    }
    let verified = match &plan.op {
        RemoteOp::Upload { local, remote } => {
            let remote = policy.check_write(remote)?;
            // Re-inspect at APPLY: plan.overwrites can be stale (TOCTOU).
            let live = exists(c, &remote).await?;
            if live {
                match c.size(&remote).await? {
                    Some(sz) if sz <= MAX_BACKUP_BYTES => {
                        let b = backup_remote(c, &remote, ctx.backup_dir, ctx.cancel).await?;
                        lines.push(format!(
                            "Saved a copy of the existing file to {}",
                            b.display()
                        ));
                        backup = Some(b);
                    }
                    Some(sz) => lines.push(format!(
                        "Warning: existing file is {sz} bytes (over backup limit); replacing without a local backup."
                    )),
                    None => {}
                }
            } else if plan.backup_remote.is_some() || plan.overwrites {
                lines.push(
                    "Existing file was gone before apply; continuing as a new upload.".into(),
                );
            }
            let part = format!("{remote}{PART_SUFFIX}");
            let mut f = tokio::fs::File::open(local).await?;
            let sent = match c.upload(&mut f, &part, ctx.progress, ctx.cancel).await {
                Ok(n) => n,
                Err(e) => {
                    let _ = c.dele(&part).await;
                    return Err(e.into());
                }
            };
            if live {
                // Rename-swap: keep the old file until the new name is in place.
                let bak = format!("{remote}{BAK_SUFFIX}");
                if exists(c, &bak).await? {
                    c.dele(&bak).await?;
                }
                c.rename(&remote, &bak).await?;
                if let Err(e) = c.rename(&part, &remote).await {
                    let _ = c.rename(&bak, &remote).await;
                    let _ = c.dele(&part).await;
                    return Err(e.into());
                }
                match c.dele(&bak).await {
                    Ok(()) => {}
                    Err(e) => lines.push(format!(
                        "Warning: could not remove temporary backup {bak}: {e}"
                    )),
                }
            } else if let Err(e) = c.rename(&part, &remote).await {
                let _ = c.dele(&part).await;
                return Err(e.into());
            }
            lines.push(format!("Uploaded {sent} bytes to {remote}"));
            c.size(&remote).await? == Some(sent)
        }
        RemoteOp::Mkdir { path } => {
            let path = policy.check_write(path)?;
            c.mkd(&path).await?;
            lines.push(format!("Created {path}"));
            let (dir, name) = parent_and_name(&path);
            c.list(&dir).await?.iter().any(|e| e.name == name)
        }
        RemoteOp::Delete { path, dir } => {
            let path = policy.check_write(path)?;
            if *dir {
                c.rmd(&path).await?
            } else {
                c.dele(&path).await?
            }
            lines.push(format!("Deleted {path}"));
            !exists(c, &path).await?
        }
        RemoteOp::Rename { from, to } => {
            let (from, to) = (policy.check_write(from)?, policy.check_write(to)?);
            c.rename(&from, &to).await?;
            lines.push(format!("Renamed {from} to {to}"));
            exists(c, &to).await? && !exists(c, &from).await?
        }
    };
    lines.push(if verified {
        "Verified: the console shows the expected result.".into()
    } else {
        "Could not verify the result. Check the console file list.".into()
    });
    Ok(Outcome {
        lines,
        backup,
        verified,
        recovery: plan.recovery.clone(),
    })
}

/// Downloads to the first free name in `local_dir` via a temporary file; never overwrites, never
/// sets an executable bit. The remote basename must be a plain name.
pub async fn download_to(
    c: &mut FtpClient,
    remote: &str,
    local_dir: &Path,
    progress: &(dyn Fn(u64) + Sync),
    cancel: &AtomicBool,
) -> Result<PathBuf, OpError> {
    use std::os::unix::fs::OpenOptionsExt;
    let remote = normalize(remote)?;
    let (_, name) = parent_and_name(&remote);
    if !valid_name(&name) {
        return Err(OpError::Refused("Invalid file name.".into()));
    }
    if !local_dir.is_dir() {
        return Err(OpError::Refused(
            "The destination folder does not exist.".into(),
        ));
    }
    let mut dest = local_dir.join(&name);
    let mut n = 1;
    while dest.exists() {
        dest = local_dir.join(format!("{name} ({n})"));
        n += 1;
    }
    let tmp = dest.with_file_name(format!(
        ".{}.part",
        dest.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download")
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(&tmp)?;
    let mut f = tokio::fs::File::from_std(file);
    let res = c.download(&remote, &mut f, progress, cancel).await;
    drop(f);
    match res {
        Ok(_) => {
            std::fs::rename(&tmp, &dest)?;
            Ok(dest)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e.into())
        }
    }
}
