//! Directory-based backups: `<root>/<id>/manifest.json` + `files/*`. The manifest records a SHA-256
//! per item so corruption or tampering of item files is detected. (The manifest itself is not signed.)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const MANIFEST_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Settings,
    Profiles,
    Themes,
    RemoteFile,
    UserFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub category: Category,
    /// Logical origin (theme id, remote path, ...). Informational.
    pub original: String,
    /// Path relative to the backup dir, always under `files/`.
    pub stored: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub id: String,
    pub created: i64,
    pub label: String,
    pub items: Vec<Item>,
}

pub struct NewItem {
    pub category: Category,
    pub original: String,
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("backup not found")]
    NotFound,
    #[error("item {0} fails its checksum: the backup is damaged or was modified")]
    Corrupt(usize),
}

pub fn hex_sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn slug(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let t = t.trim_matches('-');
    t.chars()
        .take(30)
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn safe_name(s: &str) -> String {
    let n: String = s
        .rsplit('/')
        .next()
        .unwrap_or("item")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    n.trim_start_matches('.')
        .chars()
        .take(60)
        .collect::<String>()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// A stored path is acceptable only if it is `files/<single segment>`.
fn valid_stored(p: &str) -> bool {
    p.strip_prefix("files/").is_some_and(|n| {
        !n.is_empty()
            && !n.contains('/')
            && n != "."
            && n != ".."
            && !n.contains('\\')
            && !n.contains('\0')
    })
}

pub fn create(
    root: &Path,
    label: &str,
    items: Vec<NewItem>,
    now: i64,
) -> Result<Manifest, BackupError> {
    let base = format!("{now}-{}", slug(label));
    let mut id = base.clone();
    let mut n = 2;
    while root.join(&id).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }
    let dir = root.join(&id);
    std::fs::create_dir_all(dir.join("files"))?;
    let mut out = Vec::new();
    for (i, it) in items.into_iter().enumerate() {
        let stored = format!("files/{i:03}-{}", safe_name(&it.original));
        std::fs::write(dir.join(&stored), &it.data)?;
        out.push(Item {
            category: it.category,
            original: it.original,
            stored,
            size: it.data.len() as u64,
            sha256: hex_sha256(&it.data),
        });
    }
    let m = Manifest {
        version: MANIFEST_VERSION,
        id,
        created: now,
        label: label.chars().take(60).collect(),
        items: out,
    };
    let tmp = dir.join(".manifest.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&m).expect("serialise"))?;
    std::fs::rename(tmp, dir.join("manifest.json"))?;
    Ok(m)
}

pub fn load(root: &Path, id: &str) -> Result<Manifest, BackupError> {
    if !valid_id(id) {
        return Err(BackupError::NotFound);
    }
    let p = root.join(id).join("manifest.json");
    let meta = std::fs::metadata(&p).map_err(|_| BackupError::NotFound)?;
    if meta.len() > MAX_MANIFEST_BYTES {
        return Err(BackupError::Manifest("manifest too large".into()));
    }
    let m: Manifest = serde_json::from_slice(&std::fs::read(&p)?)
        .map_err(|e| BackupError::Manifest(e.to_string()))?;
    if m.version != MANIFEST_VERSION {
        return Err(BackupError::Manifest(format!(
            "unsupported manifest version {}",
            m.version
        )));
    }
    if m.id != id {
        return Err(BackupError::Manifest(
            "manifest id does not match its folder".into(),
        ));
    }
    Ok(m)
}

/// Newest first. Unreadable or unsupported backups are skipped.
pub fn list(root: &Path) -> Vec<Manifest> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return vec![];
    };
    let mut v: Vec<Manifest> = rd
        .flatten()
        .filter_map(|e| load(root, e.file_name().to_str()?).ok())
        .collect();
    v.sort_by(|a, b| b.created.cmp(&a.created).then(b.id.cmp(&a.id)));
    v
}

fn item_path(root: &Path, m: &Manifest, index: usize) -> Result<PathBuf, BackupError> {
    let it = m.items.get(index).ok_or(BackupError::NotFound)?;
    if !valid_stored(&it.stored) {
        return Err(BackupError::Manifest(format!(
            "item {index} has an unsafe path"
        )));
    }
    Ok(root.join(&m.id).join(&it.stored))
}

/// Reads one item after checking its checksum.
pub fn read_item(root: &Path, id: &str, index: usize) -> Result<Vec<u8>, BackupError> {
    let m = load(root, id)?;
    let data = std::fs::read(item_path(root, &m, index)?)?;
    if hex_sha256(&data) != m.items[index].sha256 || data.len() as u64 != m.items[index].size {
        return Err(BackupError::Corrupt(index));
    }
    Ok(data)
}

#[derive(Debug, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub problems: Vec<String>,
}

pub fn verify(root: &Path, id: &str) -> Result<VerifyReport, BackupError> {
    let m = load(root, id)?;
    let mut problems = Vec::new();
    for i in 0..m.items.len() {
        match read_item(root, id, i) {
            Ok(_) => {}
            Err(BackupError::Corrupt(_)) => {
                problems.push(format!("{}: checksum mismatch", m.items[i].original))
            }
            Err(e) => problems.push(format!("{}: {e}", m.items[i].original)),
        }
    }
    Ok(VerifyReport {
        ok: problems.is_empty(),
        problems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(root: &Path) -> Manifest {
        create(
            root,
            "Before update!",
            vec![
                NewItem {
                    category: Category::Settings,
                    original: "settings.json".into(),
                    data: b"{\"theme\":\"dark\"}".to_vec(),
                },
                NewItem {
                    category: Category::RemoteFile,
                    original: "/data/homebrew/../x.cfg".into(),
                    data: b"cfg".to_vec(),
                },
            ],
            1_700_000_000,
        )
        .unwrap()
    }

    #[test]
    fn create_list_verify_read() {
        let d = tempfile::tempdir().unwrap();
        let m = sample(d.path());
        assert_eq!(m.version, 1);
        assert!(
            m.items.iter().all(|i| valid_stored(&i.stored)),
            "{:?}",
            m.items
        );
        assert_eq!(list(d.path()), vec![m.clone()]);
        assert!(verify(d.path(), &m.id).unwrap().ok);
        assert_eq!(
            read_item(d.path(), &m.id, 0).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        assert_eq!(
            hex_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn same_second_same_label_gets_unique_ids() {
        let d = tempfile::tempdir().unwrap();
        let a = sample(d.path());
        let b = sample(d.path());
        assert_ne!(a.id, b.id);
        assert_eq!(list(d.path()).len(), 2);
    }

    #[test]
    fn detects_corruption_and_missing_files() {
        let d = tempfile::tempdir().unwrap();
        let m = sample(d.path());
        std::fs::write(d.path().join(&m.id).join(&m.items[0].stored), b"tampered").unwrap();
        std::fs::remove_file(d.path().join(&m.id).join(&m.items[1].stored)).unwrap();
        let r = verify(d.path(), &m.id).unwrap();
        assert!(!r.ok && r.problems.len() == 2, "{:?}", r.problems);
        assert!(matches!(
            read_item(d.path(), &m.id, 0),
            Err(BackupError::Corrupt(0))
        ));
    }

    #[test]
    fn hostile_manifests_are_refused() {
        let d = tempfile::tempdir().unwrap();
        let m = sample(d.path());
        let mp = d.path().join(&m.id).join("manifest.json");
        let mut bad = m.clone();
        bad.items[0].stored = "../../etc/passwd".into();
        std::fs::write(&mp, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(matches!(
            read_item(d.path(), &m.id, 0),
            Err(BackupError::Manifest(_))
        ));
        let mut bad = m.clone();
        bad.version = 99;
        std::fs::write(&mp, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(load(d.path(), &m.id).is_err());
        let mut bad = m.clone();
        bad.id = "other".into();
        std::fs::write(&mp, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(load(d.path(), &m.id).is_err());
        std::fs::write(&mp, "not json").unwrap();
        assert!(load(d.path(), &m.id).is_err());
        assert!(
            list(d.path()).is_empty(),
            "broken backups are skipped, not fatal"
        );
        assert!(matches!(load(d.path(), "../x"), Err(BackupError::NotFound)));
        assert!(matches!(load(d.path(), ""), Err(BackupError::NotFound)));
    }

    #[test]
    fn item_names_cannot_escape() {
        let d = tempfile::tempdir().unwrap();
        let m = create(
            d.path(),
            "x",
            vec![NewItem {
                category: Category::UserFile,
                original: "/etc/../../passwd".into(),
                data: vec![1],
            }],
            5,
        )
        .unwrap();
        assert!(m.items[0].stored.starts_with("files/000-"));
        assert!(!m.items[0].stored[6..].contains('/'));
    }
}
