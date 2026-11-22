//! Discovery client against the simulator's real UDP responder.
use ps4studio_core::capability::HostState;
use ps4studio_discovery::{probe, ProbeError};
use std::time::Duration;

async fn run(id: &str) -> Result<ps4studio_discovery::DiscoveryReply, ProbeError> {
    let (addr, h) = ps4studio_mock_console::serve_discovery(id, "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let r = probe(addr, Duration::from_millis(150), 2).await;
    h.abort();
    r
}

#[tokio::test]
async fn stock_console_answers_and_is_labelled_simulated() {
    let r = run("stock").await.unwrap();
    assert_eq!(r.state, HostState::Ready);
    assert!(r.host_name.unwrap().starts_with("[SIMULATED]"));
    assert_eq!(r.system_version, None, "simulator never invents firmware");
}

#[tokio::test]
async fn offline_and_timeout_scenarios_time_out() {
    assert!(matches!(run("offline").await, Err(ProbeError::Timeout)));
    assert!(matches!(run("timeout").await, Err(ProbeError::Timeout)));
}
