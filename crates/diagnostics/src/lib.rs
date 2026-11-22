//! Diagnostic checks and a sanitised, shareable report. Nothing here invents hardware metrics.

use ps4studio_core::capability::{CapabilityEntry, Status};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Pass,
    Warn,
    Fail,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub id: &'static str,
    pub title: String,
    pub level: Level,
    pub detail: String,
    /// Human-readable next step.
    pub hint: Option<String>,
}

fn chk(
    id: &'static str,
    title: &str,
    level: Level,
    detail: impl Into<String>,
    hint: Option<&str>,
) -> Check {
    Check {
        id,
        title: title.into(),
        level,
        detail: detail.into(),
        hint: hint.map(str::to_owned),
    }
}

/// Checks that need no console.
pub fn local_checks(data_dir: Option<&std::path::Path>) -> Vec<Check> {
    let mut v = vec![chk(
        "app_version",
        "PS4 Studio version",
        Level::Info,
        env!("CARGO_PKG_VERSION"),
        None,
    )];
    v.push(match ps4studio_remote_play::detect() {
        Some(b) => chk("chiaki", "chiaki-ng (Remote Play)", Level::Pass, format!("Found: {}", b.describe()), None),
        None => chk(
            "chiaki",
            "chiaki-ng (Remote Play)",
            Level::Warn,
            "Not found on PATH or as a Flatpak.",
            Some("Install chiaki-ng from Flathub (io.github.streetpea.Chiaki4deck) to use Remote Play."),
        ),
    });
    v.push(match data_dir {
        Some(d) if d.is_dir() => chk(
            "data_dir",
            "Data directory",
            Level::Pass,
            d.display().to_string(),
            None,
        ),
        Some(d) => chk(
            "data_dir",
            "Data directory",
            Level::Fail,
            format!("{} is missing.", d.display()),
            None,
        ),
        None => chk(
            "data_dir",
            "Data directory",
            Level::Fail,
            "No XDG data directory.",
            Some("Set $HOME or $XDG_DATA_HOME."),
        ),
    });
    v
}

/// Turns the capability table into checks, with human-readable explanations.
pub fn capability_checks(caps: &[CapabilityEntry]) -> Vec<Check> {
    caps.iter()
        .map(|c| {
            let (level, detail) = match c.status {
                Status::Available => (Level::Pass, c.evidence.clone().unwrap_or_default()),
                Status::Unknown => (Level::Warn, c.reason.clone().unwrap_or_default()),
                Status::Unavailable => (Level::Fail, c.reason.clone().unwrap_or_default()),
            };
            Check {
                id: "capability",
                title: format!("{:?}", c.capability),
                level,
                detail,
                hint: None,
            }
        })
        .collect()
}

fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Removes things that should not leave the machine: IPv4 host octets, MAC addresses and long hex
/// identifiers (host-id, registration keys), `key=value` / `token:` style secrets, and the home dir.
pub fn sanitize(input: &str, home: Option<&str>) -> String {
    let mut s = input.to_owned();
    if let Some(h) = home.filter(|h| h.len() > 1) {
        s = s.replace(h, "~");
    }
    s.split('\n')
        .map(sanitize_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn sanitize_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    for k in [
        "password",
        "passcode",
        "token",
        "secret",
        "registkey",
        "regist-key",
        "morning",
        "credential",
        "psn",
    ] {
        if let Some(i) = lower.find(k) {
            // keep the label, drop everything after the separator
            if let Some(sep) = line[i..].find([':', '=']) {
                return format!("{}[redacted]", sanitize_words(&line[..i + sep + 1]));
            }
        }
    }
    sanitize_words(line)
}

fn sanitize_words(line: &str) -> String {
    line.split(' ')
        .map(sanitize_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_word(w: &str) -> String {
    let core =
        w.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '-'));
    if core.is_empty() {
        return w.to_owned();
    }
    let rep = if let Some(m) = mask_ipv4(core) {
        Some(m)
    } else if (core.len() == 17
        && core.split([':', '-']).count() == 6
        && core.split([':', '-']).all(|p| p.len() == 2 && is_hex(p)))
        || (core.len() >= 12 && is_hex(core))
    {
        Some("[id-redacted]".to_string())
    } else {
        None
    };
    match rep {
        Some(r) => w.replace(core, &r),
        None => w.to_owned(),
    }
}

fn mask_ipv4(s: &str) -> Option<String> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 3 && p.chars().all(|c| c.is_ascii_digit()))
    {
        Some(format!("{}.{}.{}.x", parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

pub fn render_report(checks: &[Check], extra: &str, home: Option<&str>) -> String {
    let mut out = String::from(
        "PS4 Studio diagnostic report (sanitised)\n=======================================\n",
    );
    for c in checks {
        out.push_str(&format!("[{:?}] {} — {}\n", c.level, c.title, c.detail));
        if let Some(h) = &c.hint {
            out.push_str(&format!("    hint: {h}\n"));
        }
    }
    if !extra.is_empty() {
        out.push('\n');
        out.push_str(extra);
        out.push('\n');
    }
    sanitize(&out, home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_ips_macs_ids_and_home() {
        let s = sanitize(
            "host 192.168.1.50 mac aa:bb:cc:dd:ee:ff id 0123456789ABCDEF0123 at /home/mikeh/x",
            Some("/home/mikeh"),
        );
        assert!(s.contains("192.168.1.x"), "{s}");
        assert!(
            !s.contains("192.168.1.50") && !s.contains("aa:bb") && !s.contains("0123456789ABCDEF"),
            "{s}"
        );
        assert!(s.contains("~/x"), "{s}");
    }

    #[test]
    fn redacts_credential_lines() {
        for line in [
            "passcode=1234",
            "Registkey: abcdef",
            "auth token = zzz",
            "user-credential:12345678",
        ] {
            let s = sanitize(line, None);
            assert!(s.contains("[redacted]"), "{line} -> {s}");
            assert!(
                !s.contains("1234")
                    && !s.contains("abcdef")
                    && !s.contains("zzz")
                    && !s.contains("12345678"),
                "{s}"
            );
        }
    }

    #[test]
    fn plain_text_survives() {
        assert_eq!(sanitize("Console not found.", None), "Console not found.");
    }

    #[test]
    fn report_is_sanitised_end_to_end() {
        let c = vec![chk(
            "x",
            "Net",
            Level::Fail,
            "no reply from 10.0.0.7 token=abc",
            Some("check Wi-Fi"),
        )];
        let r = render_report(&c, "", None);
        assert!(r.contains("10.0.0.x") || r.contains("[redacted]"));
        assert!(!r.contains("abc") && !r.contains("10.0.0.7"));
        assert!(r.contains("hint: check Wi-Fi"));
    }

    #[test]
    fn local_checks_include_version_and_chiaki() {
        let v = local_checks(None);
        assert!(v.iter().any(|c| c.id == "app_version") && v.iter().any(|c| c.id == "chiaki"));
        assert!(v
            .iter()
            .any(|c| c.id == "data_dir" && c.level == Level::Fail));
    }
}
