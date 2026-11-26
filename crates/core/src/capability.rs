//! Typed capability engine. A capability is `Available` only when a probe produced evidence;
//! an open port alone is never evidence (see `FtpFact::PortOpenOnly`).

use serde::{Deserialize, Serialize};

/// Companion protocol version this build speaks.
pub const COMPANION_PROTOCOL: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "console.discovery")]
    ConsoleDiscovery,
    #[serde(rename = "console.basic_status")]
    ConsoleBasicStatus,
    #[serde(rename = "console.remote_play")]
    ConsoleRemotePlay,
    #[serde(rename = "console.remote_wake")]
    ConsoleRemoteWake,
    #[serde(rename = "homebrew.ftp_read")]
    HomebrewFtpRead,
    #[serde(rename = "homebrew.ftp_write")]
    HomebrewFtpWrite,
    #[serde(rename = "homebrew.package_install")]
    HomebrewPackageInstall,
    #[serde(rename = "homebrew.plugin_management")]
    HomebrewPluginManagement,
    #[serde(rename = "homebrew.app_launch")]
    HomebrewAppLaunch,
    #[serde(rename = "companion.connected")]
    CompanionConnected,
    #[serde(rename = "companion.dashboard")]
    CompanionDashboard,
    #[serde(rename = "companion.theme_management")]
    CompanionThemeManagement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Stock,
    RemotePlay,
    Homebrew,
    Companion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Available,
    Unavailable,
    /// Not verified either way; UI must not enable the action.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEntry {
    pub capability: Capability,
    pub tier: Tier,
    pub status: Status,
    /// Human-readable explanation; always set when status != Available.
    pub reason: Option<String>,
    /// What was observed to justify `Available`.
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostState {
    Ready,
    Standby,
    /// Console is present but Ready vs Standby was not observed (e.g. Remote Play TCP only).
    Unknown,
}

/// How the console was found. Some consoles ignore the UDP search but still run the Remote Play
/// service, so a console can be found by either route.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryVia {
    #[default]
    UdpSearch,
    /// TCP 9295 answered with the Remote Play signature (`RP-Version` header).
    RemotePlayPort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryFact {
    #[serde(default)]
    pub via: DiscoveryVia,
    pub state: HostState,
    pub host_name: Option<String>,
    pub running_title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FtpFact {
    NotProbed,
    Unreachable {
        detail: String,
    },
    /// TCP connect worked but the FTP greeting/handshake was not verified. Not evidence of anything.
    PortOpenOnly,
    AuthFailed,
    /// Real FTP greeting + login + directory listing succeeded.
    Connected {
        writable: Option<bool>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompanionFact {
    NotProbed,
    Absent,
    Connected {
        protocol: u32,
        version: String,
        theme_management: bool,
    },
}

/// Everything the engine knows about one console right now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facts {
    pub simulated: bool,
    pub discovery: Option<DiscoveryFact>,
    /// Absolute path of a found chiaki-ng executable (or flatpak marker).
    pub remote_play_binary: Option<String>,
    pub ftp: FtpFact,
    pub companion: CompanionFact,
    /// User explicitly enabled writes for this session.
    pub write_unlocked: bool,
}

impl Facts {
    pub fn empty() -> Self {
        Self {
            simulated: false,
            discovery: None,
            remote_play_binary: None,
            ftp: FtpFact::NotProbed,
            companion: CompanionFact::NotProbed,
            write_unlocked: false,
        }
    }
}

fn yes(capability: Capability, tier: Tier, evidence: impl Into<String>) -> CapabilityEntry {
    CapabilityEntry {
        capability,
        tier,
        status: Status::Available,
        reason: None,
        evidence: Some(evidence.into()),
    }
}
fn no(capability: Capability, tier: Tier, reason: impl Into<String>) -> CapabilityEntry {
    CapabilityEntry {
        capability,
        tier,
        status: Status::Unavailable,
        reason: Some(reason.into()),
        evidence: None,
    }
}
fn unknown(capability: Capability, tier: Tier, reason: impl Into<String>) -> CapabilityEntry {
    CapabilityEntry {
        capability,
        tier,
        status: Status::Unknown,
        reason: Some(reason.into()),
        evidence: None,
    }
}

/// Pure function: facts in, capability table out. Order is stable (UI relies on it).
pub fn evaluate(f: &Facts) -> Vec<CapabilityEntry> {
    use Capability::*;
    use Tier::*;
    let src = if f.simulated { " (simulated)" } else { "" };
    let mut out = Vec::with_capacity(12);

    match &f.discovery {
        Some(d) => {
            let ev = match d.via {
                DiscoveryVia::UdpSearch => {
                    format!("Console answered the discovery search as {:?}{src}", d.state)
                }
                DiscoveryVia::RemotePlayPort => format!(
                    "The Remote Play service answered on TCP 9295 (console present; name and state unknown){src}"
                ),
            };
            out.push(yes(ConsoleDiscovery, Stock, ev.clone()));
            out.push(yes(ConsoleBasicStatus, Stock, ev));
        }
        None => {
            let r = "Console not found. Confirm that the Debian computer and PS4 are on the same local network.";
            out.push(unknown(ConsoleDiscovery, Stock, r));
            out.push(unknown(ConsoleBasicStatus, Stock, r));
        }
    }

    out.push(match (&f.remote_play_binary, &f.discovery) {
        (None, _) => no(
            ConsoleRemotePlay,
            RemotePlay,
            "chiaki-ng was not found. Install it (Flathub: io.github.streetpea.Chiaki4deck) and register your console inside it.",
        ),
        (Some(_), None) => no(ConsoleRemotePlay, RemotePlay, "Console not found, so Remote Play cannot be started."),
        (Some(p), Some(_)) => yes(
            ConsoleRemotePlay,
            RemotePlay,
            format!("chiaki-ng found at {p}{src}. Registration is checked by chiaki-ng itself, not by PS4 Studio."),
        ),
    });
    out.push(no(
        ConsoleRemoteWake,
        RemotePlay,
        "Remote wake needs a registration credential that PS4 Studio does not handle in this version.",
    ));

    let ftp_missing =
        "File management is unavailable because no supported file service was detected.";
    out.push(match &f.ftp {
        FtpFact::NotProbed => unknown(HomebrewFtpRead, Homebrew, "File service has not been checked yet."),
        FtpFact::Unreachable { detail } => no(HomebrewFtpRead, Homebrew, format!("{ftp_missing} ({detail})")),
        FtpFact::PortOpenOnly => no(
            HomebrewFtpRead,
            Homebrew,
            "A port is open but it did not complete an FTP handshake, so it is not treated as a file service.",
        ),
        FtpFact::AuthFailed => no(HomebrewFtpRead, Homebrew, "The file service rejected the login."),
        FtpFact::Connected { .. } => yes(
            HomebrewFtpRead,
            Homebrew,
            format!("FTP greeting, login and directory listing succeeded{src}. Traffic is unencrypted."),
        ),
    });
    out.push(match (&f.ftp, f.write_unlocked) {
        (FtpFact::Connected { .. }, false) => no(
            HomebrewFtpWrite,
            Homebrew,
            "Writes are locked. Unlock writes for this session to continue.",
        ),
        (
            FtpFact::Connected {
                writable: Some(true),
            },
            true,
        ) => yes(
            HomebrewFtpWrite,
            Homebrew,
            "Write verified and unlocked by you for this session.",
        ),
        (
            FtpFact::Connected {
                writable: Some(false),
            },
            true,
        ) => no(
            HomebrewFtpWrite,
            Homebrew,
            "The file service refused a write test.",
        ),
        (FtpFact::Connected { writable: None }, true) => unknown(
            HomebrewFtpWrite,
            Homebrew,
            "Unlocked, but write access is only verified on the first approved write.",
        ),
        _ => no(HomebrewFtpWrite, Homebrew, ftp_missing),
    });
    out.push(no(
        HomebrewPackageInstall,
        Homebrew,
        "No verified installation interface is implemented. Package files can be copied to an allowed folder and installed on the console.",
    ));
    out.push(no(
        HomebrewPluginManagement,
        Homebrew,
        "Plugin folder layout has not been verified, so plugins are managed as ordinary files only.",
    ));
    out.push(no(
        HomebrewAppLaunch,
        Homebrew,
        "No verified API for launching applications exists. Requires the companion with a verified launch interface.",
    ));

    match &f.companion {
        CompanionFact::Connected {
            protocol,
            version,
            theme_management,
        } if *protocol == COMPANION_PROTOCOL => {
            let ev = format!("Companion {version} speaks protocol {protocol}{src}");
            out.push(yes(CompanionConnected, Companion, ev.clone()));
            out.push(yes(CompanionDashboard, Companion, ev.clone()));
            out.push(if *theme_management {
                yes(CompanionThemeManagement, Companion, ev)
            } else {
                no(
                    CompanionThemeManagement,
                    Companion,
                    "The companion did not advertise theme support.",
                )
            });
        }
        CompanionFact::Connected {
            protocol, version, ..
        } => {
            let r = format!(
                "Companion {version} uses protocol {protocol}; this app needs {COMPANION_PROTOCOL}. Update one of them."
            );
            out.push(no(CompanionConnected, Companion, r.clone()));
            out.push(no(CompanionDashboard, Companion, r.clone()));
            out.push(no(CompanionThemeManagement, Companion, r));
        }
        other => {
            let r = if matches!(other, CompanionFact::NotProbed) {
                "The companion has not been checked yet."
            } else {
                "Custom console dashboard requires the PS4 Studio companion running on a compatible homebrew environment."
            };
            out.push(unknown_or_no(
                matches!(other, CompanionFact::NotProbed),
                CompanionConnected,
                r,
            ));
            out.push(no(CompanionDashboard, Companion, r));
            out.push(no(CompanionThemeManagement, Companion, r));
        }
    }
    out
}

fn unknown_or_no(is_unknown: bool, c: Capability, r: &str) -> CapabilityEntry {
    if is_unknown {
        unknown(c, Tier::Companion, r)
    } else {
        no(c, Tier::Companion, r)
    }
}

pub fn status_of(entries: &[CapabilityEntry], c: Capability) -> Status {
    entries
        .iter()
        .find(|e| e.capability == c)
        .map_or(Status::Unavailable, |e| e.status)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(f: &Facts, c: Capability) -> Status {
        status_of(&evaluate(f), c)
    }

    #[test]
    fn empty_facts_enable_nothing() {
        let e = evaluate(&Facts::empty());
        assert!(e.iter().all(|x| x.status != Status::Available));
        assert!(
            e.iter().all(|x| x.reason.is_some()),
            "every disabled capability explains why"
        );
    }

    #[test]
    fn open_port_is_not_ftp() {
        let mut f = Facts::empty();
        f.ftp = FtpFact::PortOpenOnly;
        assert_eq!(st(&f, Capability::HomebrewFtpRead), Status::Unavailable);
    }

    #[test]
    fn writes_need_handshake_and_unlock() {
        let mut f = Facts::empty();
        f.ftp = FtpFact::Connected {
            writable: Some(true),
        };
        assert_eq!(st(&f, Capability::HomebrewFtpRead), Status::Available);
        assert_eq!(st(&f, Capability::HomebrewFtpWrite), Status::Unavailable);
        f.write_unlocked = true;
        assert_eq!(st(&f, Capability::HomebrewFtpWrite), Status::Available);
        f.ftp = FtpFact::Connected { writable: None };
        assert_eq!(st(&f, Capability::HomebrewFtpWrite), Status::Unknown);
        f.ftp = FtpFact::PortOpenOnly;
        assert_eq!(st(&f, Capability::HomebrewFtpWrite), Status::Unavailable);
    }

    #[test]
    fn remote_play_needs_binary_and_console() {
        let mut f = Facts::empty();
        f.remote_play_binary = Some("/usr/bin/chiaki".into());
        assert_eq!(st(&f, Capability::ConsoleRemotePlay), Status::Unavailable);
        f.discovery = Some(DiscoveryFact {
            via: DiscoveryVia::UdpSearch,
            state: HostState::Ready,
            host_name: None,
            running_title: None,
        });
        assert_eq!(st(&f, Capability::ConsoleRemotePlay), Status::Available);
        f.remote_play_binary = None;
        assert_eq!(st(&f, Capability::ConsoleRemotePlay), Status::Unavailable);
    }

    #[test]
    fn remote_play_port_discovery_does_not_invent_ready_state() {
        let mut f = Facts::empty();
        f.discovery = Some(DiscoveryFact {
            via: DiscoveryVia::RemotePlayPort,
            state: HostState::Unknown,
            host_name: None,
            running_title: None,
        });
        assert_eq!(st(&f, Capability::ConsoleDiscovery), Status::Available);
        assert_eq!(st(&f, Capability::ConsoleBasicStatus), Status::Available);
        let ev = evaluate(&f)
            .into_iter()
            .find(|c| c.capability == Capability::ConsoleBasicStatus)
            .and_then(|c| c.evidence)
            .unwrap();
        assert!(
            ev.contains("state unknown"),
            "evidence must admit Ready/Standby was not observed: {ev}"
        );
    }

    #[test]
    fn companion_version_mismatch_disables_everything() {
        let mut f = Facts::empty();
        f.companion = CompanionFact::Connected {
            protocol: 99,
            version: "9.9".into(),
            theme_management: true,
        };
        assert_eq!(st(&f, Capability::CompanionConnected), Status::Unavailable);
        assert_eq!(
            st(&f, Capability::CompanionThemeManagement),
            Status::Unavailable
        );
        f.companion = CompanionFact::Connected {
            protocol: COMPANION_PROTOCOL,
            version: "0.1".into(),
            theme_management: true,
        };
        assert_eq!(
            st(&f, Capability::CompanionThemeManagement),
            Status::Available
        );
    }

    #[test]
    fn never_available_in_v01() {
        let mut f = Facts::empty();
        f.ftp = FtpFact::Connected {
            writable: Some(true),
        };
        f.write_unlocked = true;
        for c in [
            Capability::ConsoleRemoteWake,
            Capability::HomebrewPackageInstall,
            Capability::HomebrewAppLaunch,
        ] {
            assert_eq!(st(&f, c), Status::Unavailable);
        }
    }

    #[test]
    fn serde_names_are_dotted() {
        assert_eq!(
            serde_json::to_string(&Capability::HomebrewFtpRead).unwrap(),
            "\"homebrew.ftp_read\""
        );
    }
}
