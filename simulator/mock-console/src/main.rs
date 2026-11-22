//! `mock-console`: prints the scenario table, or with `--json` the scenarios plus the capability
//! table the real engine computes for each (used to generate UI test fixtures).
use ps4studio_core::capability::{evaluate, CompanionFact, Facts, FtpFact};

fn main() {
    let all = ps4studio_mock_console::scenarios();
    if std::env::args().any(|a| a == "--json") {
        let out: Vec<_> = all
            .iter()
            .map(|s| {
                // Capabilities as the UI sees them at each stage of the real flow:
                // `capabilities` = just selected (FTP unprobed); `after_ftp_probe` = scripted FTP result
                // verified; `after_unlock` = additionally writes unlocked by the user.
                let base = Facts {
                    ftp: FtpFact::NotProbed,
                    companion: CompanionFact::NotProbed,
                    ..s.facts.clone()
                };
                let comp = Facts {
                    ftp: FtpFact::NotProbed,
                    ..s.facts.clone()
                };
                let probed = Facts {
                    companion: CompanionFact::NotProbed,
                    ..s.facts.clone()
                };
                let unlocked = Facts {
                    write_unlocked: true,
                    companion: CompanionFact::NotProbed,
                    ..s.facts.clone()
                };
                serde_json::json!({
                    "scenario": s,
                    "ftp_script": s.facts.ftp,
                    "capabilities": evaluate(&base),
                    "after_ftp_probe": evaluate(&probed),
                    "after_companion": evaluate(&comp),
                    "after_unlock": evaluate(&unlocked),
                    "simulated": s.facts.simulated,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out).expect("serialise"));
    } else {
        for s in all {
            println!("{:<22} {}", s.id, s.label);
        }
    }
}
