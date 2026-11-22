//! `mock-console`: prints the scenario table, or with `--json` the scenarios plus the capability
//! table the real engine computes for each (used to generate UI test fixtures).
use ps4studio_core::capability::evaluate;

fn main() {
    let all = ps4studio_mock_console::scenarios();
    if std::env::args().any(|a| a == "--json") {
        let out: Vec<_> = all
            .iter()
            .map(|s| {
                serde_json::json!({
                    "scenario": s,
                    "capabilities": evaluate(&s.facts),
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
