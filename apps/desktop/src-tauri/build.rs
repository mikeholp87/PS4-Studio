// Every command exposed to the webview must be listed here AND granted in capabilities/default.json.
const COMMANDS: &[&str] = &[
    "app_info",
    "list_scenarios",
    "connect_simulator",
    "disconnect",
    "get_connection",
    "list_profiles",
    "add_profile",
    "delete_profile",
    "get_setting",
    "set_setting",
    "list_audit",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("tauri build failed");
}
