//! Theme storage. Themes are pure data validated against the same JSON Schema the UI uses
//! (`packages/theme-engine/theme.schema.json`); both sides run the shared fixture corpus.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const MAX_THEME_BYTES: usize = 256 * 1024;
const SCHEMA: &str = include_str!("../../../packages/theme-engine/theme.schema.json");

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StoredTheme {
    pub id: String,
    pub name: String,
    pub json: String,
}

pub fn is_valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    !b.is_empty()
        && b.len() <= 40
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// Validates untrusted theme text. Returns the parsed value on success, human-readable errors otherwise.
pub fn validate(text: &str) -> Result<Value, Vec<String>> {
    if text.len() > MAX_THEME_BYTES {
        return Err(vec!["theme file is too large".into()]);
    }
    let value: Value =
        serde_json::from_str(text).map_err(|_| vec!["not valid JSON".to_string()])?;
    let schema: Value = serde_json::from_str(SCHEMA).expect("embedded schema is valid JSON");
    let validator = jsonschema::draft7::new(&schema).expect("embedded schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| format!("{} {}", e.instance_path(), e))
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut seen = std::collections::HashSet::new();
    for t in value["tiles"].as_array().into_iter().flatten() {
        if !seen.insert(t["id"].as_str().unwrap_or_default()) {
            return Err(vec!["duplicate tile id".into()]);
        }
    }
    Ok(value)
}

pub struct ThemeStore {
    dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("invalid theme: {}", .0.join("; "))]
    Invalid(Vec<String>),
    #[error("invalid theme id")]
    BadId,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl ThemeStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, id: &str) -> Result<PathBuf, ThemeError> {
        if !is_valid_id(id) {
            return Err(ThemeError::BadId);
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    /// Validates, then writes atomically (temp file + rename). Returns the stored theme.
    pub fn save(&self, text: &str) -> Result<StoredTheme, ThemeError> {
        let v = validate(text).map_err(ThemeError::Invalid)?;
        let id = v["id"].as_str().unwrap_or_default().to_owned();
        let path = self.path(&id)?;
        std::fs::create_dir_all(&self.dir)?;
        let json = serde_json::to_string_pretty(&v).expect("serialise");
        let tmp = self.dir.join(format!(".{id}.tmp"));
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, &path)?;
        Ok(StoredTheme {
            id,
            name: v["name"].as_str().unwrap_or_default().to_owned(),
            json,
        })
    }

    /// Lists valid themes; files that fail validation are skipped, never loaded.
    pub fn list(&self) -> Vec<StoredTheme> {
        let Ok(rd) = std::fs::read_dir(&self.dir) else {
            return vec![];
        };
        let mut out: Vec<StoredTheme> = rd
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                let id = name.strip_suffix(".json")?.to_owned();
                if !is_valid_id(&id) || e.metadata().ok()?.len() as usize > MAX_THEME_BYTES {
                    return None;
                }
                let json = std::fs::read_to_string(e.path()).ok()?;
                let v = validate(&json).ok()?;
                (v["id"] == id.as_str()).then(|| StoredTheme {
                    name: v["name"].as_str().unwrap_or_default().to_owned(),
                    id,
                    json,
                })
            })
            .collect();
        out.sort_by_key(|a| a.name.to_lowercase());
        out
    }

    /// Imports untrusted text: validates, then renames the id if it collides with an existing theme.
    pub fn import(&self, text: &str) -> Result<StoredTheme, ThemeError> {
        let mut v = validate(text).map_err(ThemeError::Invalid)?;
        let taken: std::collections::HashSet<String> =
            self.list().into_iter().map(|t| t.id).collect();
        let base = v["id"].as_str().unwrap_or_default().to_owned();
        let mut id = base.clone();
        let mut n = 2;
        while taken.contains(&id) {
            let suffix = format!("-{n}");
            id = format!("{}{suffix}", &base[..base.len().min(40 - suffix.len())]);
            n += 1;
        }
        v["id"] = Value::String(id);
        self.save(&v.to_string())
    }

    pub fn delete(&self, id: &str) -> Result<bool, ThemeError> {
        match std::fs::remove_file(self.path(id)?) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get(&self, id: &str) -> Result<Option<StoredTheme>, ThemeError> {
        Ok(self.list().into_iter().find(|t| t.id == id))
    }
}

/// Reads a user-chosen file for import. Size-capped *before* reading into memory.
pub fn read_import(path: &Path) -> Result<String, ThemeError> {
    let meta = std::fs::metadata(path)?;
    if meta.len() as usize > MAX_THEME_BYTES {
        return Err(ThemeError::Invalid(vec!["theme file is too large".into()]));
    }
    Ok(std::fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus(kind: &str) -> Vec<(String, String)> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/theme-engine/fixtures")
            .join(kind);
        let mut v: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (
                    e.file_name().to_string_lossy().into_owned(),
                    std::fs::read_to_string(e.path()).unwrap(),
                )
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn shared_corpus_valid() {
        let c = corpus("valid");
        assert!(c.len() >= 3);
        for (n, t) in c {
            assert!(validate(&t).is_ok(), "{n} should be valid");
        }
    }

    #[test]
    fn shared_corpus_invalid() {
        let c = corpus("invalid");
        assert!(c.len() >= 10);
        for (n, t) in c {
            assert!(validate(&t).is_err(), "{n} should be rejected");
        }
    }

    #[test]
    fn rejects_garbage_and_oversize() {
        assert!(validate("{nope").is_err());
        assert!(validate(&"x".repeat(MAX_THEME_BYTES + 1)).is_err());
        assert!(validate("null").is_err());
    }

    #[test]
    fn id_rules_block_traversal() {
        for bad in ["", "../x", "a/b", "A", "-a", "a.b", &"a".repeat(41)] {
            assert!(!is_valid_id(bad), "{bad}");
        }
        assert!(is_valid_id("my-theme-2"));
    }

    #[test]
    fn store_roundtrip_skips_tampered_files() {
        let d = tempfile::tempdir().unwrap();
        let s = ThemeStore::new(d.path().join("themes"));
        let good = corpus("valid")
            .into_iter()
            .find(|(n, _)| n == "minimal.json")
            .unwrap()
            .1;
        let t = s.save(&good).unwrap();
        assert_eq!(t.id, "minimal");
        assert_eq!(s.list().len(), 1);
        // tampered file with a mismatched id and an invalid file are never listed
        std::fs::write(d.path().join("themes/other.json"), &good).unwrap();
        std::fs::write(d.path().join("themes/evil.json"), "{\"script\":1}").unwrap();
        assert_eq!(s.list().len(), 1);
        assert!(s.save("{}").is_err());
        assert!(s.delete("../etc").is_err());
        assert!(s.delete("minimal").unwrap());
        assert!(!s.delete("minimal").unwrap());
    }

    #[test]
    fn import_renames_on_collision_and_rejects_hostile() {
        let d = tempfile::tempdir().unwrap();
        let s = ThemeStore::new(d.path().to_path_buf());
        let good = corpus("valid")
            .into_iter()
            .find(|(n, _)| n == "aurora.json")
            .unwrap()
            .1;
        assert_eq!(s.import(&good).unwrap().id, "aurora");
        assert_eq!(s.import(&good).unwrap().id, "aurora-2");
        assert_eq!(s.import(&good).unwrap().id, "aurora-3");
        assert_eq!(s.list().len(), 3);
        for (_, bad) in corpus("invalid") {
            assert!(s.import(&bad).is_err());
        }
        assert_eq!(s.list().len(), 3);
    }

    #[test]
    fn import_size_cap_applies_before_read() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("big.json");
        std::fs::write(&p, vec![b' '; MAX_THEME_BYTES + 10]).unwrap();
        assert!(read_import(&p).is_err());
    }
}
