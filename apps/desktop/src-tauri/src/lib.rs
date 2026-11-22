mod backups_cmd;
mod commands;
mod files;
mod homebrew;

pub fn run() {
    let state = commands::AppState::open().expect("could not open application database");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            commands::list_themes,
            commands::save_theme,
            commands::delete_theme,
            commands::export_theme,
            commands::import_theme,
            files::files_connect,
            files::files_disconnect,
            files::files_list_remote,
            files::files_list_local,
            files::files_set_write_lock,
            files::files_plan,
            files::files_apply,
            files::files_cancel,
            files::files_download,
            backups_cmd::backup_create,
            backups_cmd::backup_list,
            backups_cmd::backup_verify,
            backups_cmd::backup_delete,
            backups_cmd::backup_restore_item,
            backups_cmd::backup_save_item_as,
            homebrew::hb_pick_and_scan,
            homebrew::hb_verify,
            homebrew::hb_upload_op,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PS4 Studio");
}
