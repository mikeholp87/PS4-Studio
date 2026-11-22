//! Simulated PS4. Everything here is fake and says so: `simulated` is always true and the
//! console name carries a `[SIMULATED]` prefix. No hardware readings are ever fabricated;
//! storage/memory figures are simply absent (`None`).

use ps4studio_core::capability::{
    CompanionFact, DiscoveryFact, Facts, FtpFact, HostState, COMPANION_PROTOCOL,
};
use serde::Serialize;

pub const SIM_LABEL: &str = "[SIMULATED]";

/// Behaviour the file/transfer layers must emulate (consumed from P3/P5 onward).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Behavior {
    None,
    PairingRequired,
    TransferInterrupts { after_bytes: u64 },
    StorageLimit { free_bytes: u64 },
    UnauthorizedWrite,
    Timeout,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scenario {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub console_name: String,
    /// Firmware string the simulated console *reports*. Never presented as verified.
    pub reported_firmware: Option<&'static str>,
    pub behavior: Behavior,
    pub simulated: bool,
    #[serde(skip)]
    pub facts: Facts,
}

fn disc(state: HostState) -> Option<DiscoveryFact> {
    Some(DiscoveryFact {
        state,
        host_name: Some(format!("{SIM_LABEL} PS4 Slim")),
        running_title: None,
    })
}

fn base() -> Facts {
    let mut f = Facts::empty();
    f.simulated = true;
    f
}

fn sc(
    id: &'static str,
    label: &'static str,
    description: &'static str,
    reported_firmware: Option<&'static str>,
    behavior: Behavior,
    facts: Facts,
) -> Scenario {
    Scenario {
        id,
        label,
        description,
        console_name: format!("{SIM_LABEL} PS4 Slim"),
        reported_firmware,
        behavior,
        simulated: true,
        facts,
    }
}

/// All 14 documented scenarios, in UI order.
pub fn scenarios() -> Vec<Scenario> {
    let rp = Some("simulated:chiaki-ng".to_string());
    let ftp_ok = FtpFact::Connected {
        writable: Some(true),
    };
    let comp_ok = CompanionFact::Connected {
        protocol: COMPANION_PROTOCOL,
        version: "0.1.0-sim".into(),
        theme_management: true,
    };
    let stock = || {
        let mut f = base();
        f.discovery = disc(HostState::Ready);
        f
    };
    vec![
        sc(
            "offline",
            "Console offline",
            "No reply to discovery.",
            None,
            Behavior::None,
            base(),
        ),
        sc(
            "stock",
            "Stock console discovered",
            "Answers discovery; nothing else.",
            Some("9.xx"),
            Behavior::None,
            stock(),
        ),
        sc(
            "remote_play",
            "Remote Play available",
            "Stock console with chiaki-ng installed.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.remote_play_binary = rp.clone();
                f
            },
        ),
        sc(
            "pairing_required",
            "Pairing required",
            "Remote Play launch fails until registered in chiaki-ng.",
            Some("9.xx"),
            Behavior::PairingRequired,
            {
                let mut f = stock();
                f.remote_play_binary = rp;
                f
            },
        ),
        sc(
            "homebrew",
            "Homebrew service available",
            "FTP service verified by handshake and listing.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.ftp = ftp_ok.clone();
                f
            },
        ),
        sc(
            "ftp_auth_failure",
            "FTP login rejected",
            "Service present but refuses login.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.ftp = FtpFact::AuthFailed;
                f
            },
        ),
        sc(
            "transfer_ok",
            "Transfer succeeds",
            "Uploads complete and verify.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.ftp = ftp_ok.clone();
                f
            },
        ),
        sc(
            "transfer_interrupted",
            "Transfer interrupted",
            "Connection drops mid-upload.",
            Some("9.xx"),
            Behavior::TransferInterrupts { after_bytes: 4096 },
            {
                let mut f = stock();
                f.ftp = ftp_ok.clone();
                f
            },
        ),
        sc(
            "unsupported_firmware",
            "Unsupported firmware",
            "Reports a firmware PS4 Studio has no compatibility data for.",
            Some("0.00"),
            Behavior::None,
            stock(),
        ),
        sc(
            "companion",
            "Companion connected",
            "PS4 Studio companion with matching protocol.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.ftp = ftp_ok.clone();
                f.companion = comp_ok;
                f
            },
        ),
        sc(
            "companion_mismatch",
            "Companion version mismatch",
            "Companion speaks a different protocol.",
            Some("9.xx"),
            Behavior::None,
            {
                let mut f = stock();
                f.companion = CompanionFact::Connected {
                    protocol: COMPANION_PROTOCOL + 1,
                    version: "9.9.9-sim".into(),
                    theme_management: true,
                };
                f
            },
        ),
        sc(
            "storage_limit",
            "Storage nearly full",
            "Uploads beyond the free-space limit are refused.",
            Some("9.xx"),
            Behavior::StorageLimit {
                free_bytes: 1024 * 1024,
            },
            {
                let mut f = stock();
                f.ftp = ftp_ok.clone();
                f
            },
        ),
        sc(
            "unauthorized",
            "Unauthorized write",
            "Service accepts login but refuses writes.",
            Some("9.xx"),
            Behavior::UnauthorizedWrite,
            {
                let mut f = stock();
                f.ftp = FtpFact::Connected {
                    writable: Some(false),
                };
                f
            },
        ),
        sc(
            "timeout",
            "Network timeout",
            "Discovery replies arrive too late.",
            None,
            Behavior::Timeout,
            base(),
        ),
    ]
}

pub fn scenario(id: &str) -> Option<Scenario> {
    scenarios().into_iter().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ps4studio_core::capability::{evaluate, status_of, Capability, Status};

    #[test]
    fn fourteen_unique_scenarios_all_labelled_simulated() {
        let all = scenarios();
        assert_eq!(all.len(), 14);
        let mut ids: Vec<_> = all.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 14);
        assert!(all
            .iter()
            .all(|s| s.simulated && s.facts.simulated && s.console_name.starts_with(SIM_LABEL)));
    }

    #[test]
    fn scenarios_drive_expected_capabilities() {
        let cap = |id: &str, c| status_of(&evaluate(&scenario(id).unwrap().facts), c);
        assert_eq!(
            cap("offline", Capability::ConsoleDiscovery),
            Status::Unknown
        );
        assert_eq!(
            cap("stock", Capability::ConsoleBasicStatus),
            Status::Available
        );
        assert_eq!(
            cap("stock", Capability::ConsoleRemotePlay),
            Status::Unavailable
        );
        assert_eq!(
            cap("remote_play", Capability::ConsoleRemotePlay),
            Status::Available
        );
        assert_eq!(
            cap("homebrew", Capability::HomebrewFtpRead),
            Status::Available
        );
        assert_eq!(
            cap("ftp_auth_failure", Capability::HomebrewFtpRead),
            Status::Unavailable
        );
        assert_eq!(
            cap("companion", Capability::CompanionThemeManagement),
            Status::Available
        );
        assert_eq!(
            cap("companion_mismatch", Capability::CompanionConnected),
            Status::Unavailable
        );
    }
}
