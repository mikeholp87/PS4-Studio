//! Remote Play = launch a locally installed chiaki-ng (AGPL) as a *separate process*.
//! We never link or embed it, never pass secret-bearing flags (`--registkey`, `--morning`,
//! `--passcode` would be readable in /proc/<pid>/cmdline), and discard its output so no
//! credential can reach our logs. The executable is only ever one we detected ourselves.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

pub const FLATPAK_ID: &str = "io.github.streetpea.Chiaki4deck";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binary {
    Native(PathBuf),
    Flatpak,
}

impl Binary {
    pub fn describe(&self) -> String {
        match self {
            Binary::Native(p) => p.display().to_string(),
            Binary::Flatpak => format!("flatpak:{FLATPAK_ID}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("chiaki-ng is not installed")]
    NotInstalled,
    #[error("{0}")]
    Invalid(String),
    #[error("could not start chiaki-ng: {0}")]
    Spawn(#[from] std::io::Error),
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Searches `path_var` (a PATH-style string) for the chiaki executables, then the Flatpak install dirs.
pub fn detect_in(path_var: &str, flatpak_roots: &[PathBuf]) -> Option<Binary> {
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        for name in ["chiaki-ng", "chiaki"] {
            let p = Path::new(dir).join(name);
            if is_executable(&p) {
                return Some(Binary::Native(p));
            }
        }
    }
    flatpak_roots
        .iter()
        .any(|r| r.join("app").join(FLATPAK_ID).is_dir())
        .then_some(Binary::Flatpak)
}

pub fn detect() -> Option<Binary> {
    let mut roots = vec![PathBuf::from("/var/lib/flatpak")];
    if let Some(d) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    {
        roots.push(d.join("flatpak"));
    }
    detect_in(&std::env::var("PATH").unwrap_or_default(), &roots)
}

#[derive(Debug, Clone)]
pub struct LaunchSpec {
    /// Nickname of the console as registered inside chiaki-ng.
    pub nickname: String,
    pub host: IpAddr,
    pub fullscreen: bool,
}

impl LaunchSpec {
    pub fn validate(&self) -> Result<(), LaunchError> {
        let n = &self.nickname;
        let ok = !n.is_empty()
            && n.chars().count() <= 32
            && !n.starts_with('-')
            && n.chars()
                .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '.' | '-'));
        if ok {
            Ok(())
        } else {
            Err(LaunchError::Invalid(
                "Console nickname must be 1–32 letters, digits, spaces, '.', '_' or '-' and must not start with '-'.".into(),
            ))
        }
    }
}

/// Builds the argv. Pure and unit-tested so we can prove no secret flag is ever emitted.
pub fn build_command(
    bin: &Binary,
    spec: &LaunchSpec,
) -> Result<(String, Vec<String>), LaunchError> {
    spec.validate()?;
    let mut tail = vec![
        "stream".to_string(),
        spec.nickname.clone(),
        spec.host.to_string(),
    ];
    if spec.fullscreen {
        tail.push("--fullscreen".into());
    }
    Ok(match bin {
        Binary::Native(p) => (p.display().to_string(), tail),
        Binary::Flatpak => {
            let mut args = vec!["run".to_string(), FLATPAK_ID.to_string()];
            args.extend(tail);
            ("flatpak".into(), args)
        }
    })
}

pub struct Session {
    child: Child,
}

impl Session {
    pub fn start(bin: &Binary, spec: &LaunchSpec) -> Result<Self, LaunchError> {
        let (prog, args) = build_command(bin, spec)?;
        let child = Command::new(prog)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        Ok(Self { child })
    }

    /// `Some(code)` once the process has exited.
    pub fn try_exit(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    /// Asks the process to quit (SIGTERM), waits up to `grace`, then kills it.
    pub async fn stop(&mut self, grace: std::time::Duration) -> std::io::Result<()> {
        if let Some(pid) = self.child.id() {
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid as i32),
                nix::sys::signal::Signal::SIGTERM,
            );
        }
        if tokio::time::timeout(grace, self.child.wait())
            .await
            .is_err()
        {
            self.child.kill().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn spec(n: &str) -> LaunchSpec {
        LaunchSpec {
            nickname: n.into(),
            host: "192.168.1.50".parse().unwrap(),
            fullscreen: true,
        }
    }

    #[test]
    fn argv_never_contains_secret_flags() {
        for bin in [Binary::Native("/usr/bin/chiaki".into()), Binary::Flatpak] {
            let (_, args) = build_command(&bin, &spec("My PS4")).unwrap();
            for a in &args {
                assert!(!["--registkey", "--morning", "--passcode"].contains(&a.as_str()));
            }
            assert!(args
                .windows(3)
                .any(|w| w == ["stream", "My PS4", "192.168.1.50"]));
            assert!(args.contains(&"--fullscreen".to_string()));
        }
        assert_eq!(
            build_command(&Binary::Flatpak, &spec("a")).unwrap().0,
            "flatpak"
        );
    }

    #[test]
    fn rejects_option_injection_in_nickname() {
        for bad in [
            "",
            "--passcode",
            "-x",
            "a;b",
            "a\nb",
            &"x".repeat(33),
            "$(id)",
        ] {
            assert!(spec(bad).validate().is_err(), "{bad:?}");
        }
        assert!(spec("Living-room_PS4 2").validate().is_ok());
    }

    #[test]
    fn detects_from_path_only_executables() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("chiaki");
        std::fs::write(&p, "#!/bin/sh\n").unwrap();
        assert_eq!(
            detect_in(d.path().to_str().unwrap(), &[]),
            None,
            "non-executable ignored"
        );
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            detect_in(d.path().to_str().unwrap(), &[]),
            Some(Binary::Native(p))
        );
    }

    #[test]
    fn detects_flatpak_dir() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("app").join(FLATPAK_ID)).unwrap();
        assert_eq!(
            detect_in("", &[d.path().to_path_buf()]),
            Some(Binary::Flatpak)
        );
        assert_eq!(detect_in("", &[]), None);
    }

    #[tokio::test]
    async fn starts_and_stops_a_fake_binary_cleanly() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("chiaki");
        std::fs::write(&p, "#!/bin/sh\nexec sleep 30\n").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut s = Session::start(&Binary::Native(p), &spec("t")).unwrap();
        assert!(s.try_exit().unwrap().is_none(), "still running");
        let t = std::time::Instant::now();
        s.stop(std::time::Duration::from_secs(2)).await.unwrap();
        assert!(
            t.elapsed() < std::time::Duration::from_secs(2),
            "SIGTERM honoured, no kill needed"
        );
        assert!(s.try_exit().unwrap().is_some());
    }
}
