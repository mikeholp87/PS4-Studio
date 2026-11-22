mod commands;

pub fn run() {
    let state = commands::AppState::open().expect("could not open application database");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_scenarios,
            commands::connect_simulator,
            commands::disconnect,
            commands::get_connection,
            commands::list_profiles,
            commands::add_profile,
            commands::delete_profile,
            commands::get_setting,
            commands::set_setting,
            commands::list_audit,
            commands::discover_lan,
            commands::connect_profile,
            commands::remote_play_status,
            commands::start_remote_play,
            commands::stop_remote_play,
            commands::run_diagnostics,
            commands::export_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PS4 Studio");
}
