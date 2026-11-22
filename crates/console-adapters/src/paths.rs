//! Remote path policy. Reads may go anywhere the service lets us; any change (write, delete,
//! rename, mkdir) is confined to an allowlist of managed roots and never touches protected trees.

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PathError {
    #[error("path must be absolute")]
    NotAbsolute,
    #[error("path contains '..' or other forbidden parts")]
    Traversal,
    #[error("path contains control characters or backslashes")]
    BadChars,
    #[error("path is too long")]
    TooLong,
    #[error("changes are only allowed inside: {0}")]
    OutsideAllowlist(String),
    #[error("{0} is a protected system location and cannot be changed")]
    Protected(String),
}

/// Lexically normalises an absolute POSIX path: collapses `//` and `.`; rejects `..`, NUL/control
/// characters and backslashes outright (never "resolves" `..`, so there is nothing to escape).
pub fn normalize(p: &str) -> Result<String, PathError> {
    if p.len() > 1024 {
        return Err(PathError::TooLong);
    }
    if p.chars().any(|c| c.is_control() || c == '\\') {
        return Err(PathError::BadChars);
    }
    if !p.starts_with('/') {
        return Err(PathError::NotAbsolute);
    }
    let mut parts = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => return Err(PathError::Traversal),
            s => parts.push(s),
        }
    }
    Ok(format!("/{}", parts.join("/")))
}

/// A single file/dir name (no separators).
pub fn valid_name(n: &str) -> bool {
    !n.is_empty() && n.len() <= 255 && n != "." && n != ".." && !n.contains('/') && !n.chars().any(|c| c.is_control() || c == '\\')
}

pub fn join(dir: &str, name: &str) -> Result<String, PathError> {
    if !valid_name(name) {
        return Err(PathError::Traversal);
    }
    normalize(&format!("{}/{}", dir.trim_end_matches('/'), name))
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub write_roots: Vec<String>,
    pub protected: Vec<String>,
}

impl Default for Policy {
    /// `/data` is the documented homebrew data area (GoldHEN README: `/data/pkg`). Everything else is read-only.
    fn default() -> Self {
        Self {
            write_roots: vec!["/data".into()],
            protected: ["/system", "/system_ex", "/preinst", "/update", "/dev", "/boot"].map(String::from).to_vec(),
        }
    }
}

fn under(path: &str, root: &str) -> bool {
    path == root || path.strip_prefix(root).is_some_and(|r| r.starts_with('/'))
}

impl Policy {
    /// Returns the normalised path if a *change* to it is allowed. The root itself cannot be changed.
    pub fn check_write(&self, p: &str) -> Result<String, PathError> {
        let n = normalize(p)?;
        if let Some(prot) = self.protected.iter().find(|r| under(&n, r)) {
            return Err(PathError::Protected(prot.clone()));
        }
        match self.write_roots.iter().find(|r| under(&n, r)) {
            Some(r) if &n != r => Ok(n),
            _ => Err(PathError::OutsideAllowlist(self.write_roots.join(", "))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_and_rejects() {
        assert_eq!(normalize("/data//a/./b/").unwrap(), "/data/a/b");
        assert_eq!(normalize("/").unwrap(), "/");
        for (bad, e) in [
            ("data/x", PathError::NotAbsolute),
            ("/data/../system", PathError::Traversal),
            ("/data/a/..", PathError::Traversal),
            ("/data/a\0b", PathError::BadChars),
            ("/data\\x", PathError::BadChars),
            ("/data/a\nb", PathError::BadChars),
        ] {
            assert_eq!(normalize(bad), Err(e), "{bad:?}");
        }
        assert_eq!(normalize(&format!("/{}", "a".repeat(2000))), Err(PathError::TooLong));
    }

    #[test]
    fn write_policy() {
        let p = Policy::default();
        assert_eq!(p.check_write("/data/pkg/app.pkg").unwrap(), "/data/pkg/app.pkg");
        assert!(matches!(p.check_write("/data"), Err(PathError::OutsideAllowlist(_))), "root itself is not changeable");
        assert!(matches!(p.check_write("/datax/evil"), Err(PathError::OutsideAllowlist(_))), "prefix trick");
        assert!(matches!(p.check_write("/system/sys.dat"), Err(PathError::Protected(_))));
        assert!(matches!(p.check_write("/user/home"), Err(PathError::OutsideAllowlist(_))));
        assert_eq!(p.check_write("/data/../system/x"), Err(PathError::Traversal));
        let mut q = Policy::default();
        q.write_roots.push("/system".into());
        assert!(matches!(q.check_write("/system/x"), Err(PathError::Protected(_))), "protected beats allowlist");
    }

    #[test]
    fn names_and_join() {
        assert!(valid_name("game.pkg") && !valid_name("..") && !valid_name("a/b") && !valid_name(""));
        assert_eq!(join("/data/", "x.pkg").unwrap(), "/data/x.pkg");
        assert!(join("/data", "../x").is_err());
    }
}
