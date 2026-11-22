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

#[cfg(test)]
mod ipc_tests {
    //! Drives the REAL command handlers through Tauri's test runtime against the simulator.
    use super::commands::AppState;
    use serde_json::{json, Value};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
    use tauri::webview::InvokeRequest;

    fn call(
        w: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Value>().unwrap())
    }

    fn cap<'a>(v: &'a Value, id: &str) -> &'a Value {
        v["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["capability"] == id)
            .unwrap()
    }

    #[test]
    fn simulator_file_flow_end_to_end() {
        let data = tempfile::tempdir().unwrap();
        // SAFETY of env mutation: this is the only test in the crate touching XDG_DATA_HOME.
        std::env::set_var("XDG_DATA_HOME", data.path());
        let app = mock_builder()
            .manage(AppState::open().unwrap())
            .invoke_handler(tauri::generate_handler![
                super::commands::connect_simulator,
                super::files::files_connect,
                super::files::files_list_remote,
                super::files::files_set_write_lock,
                super::files::files_plan,
                super::files::files_apply,
                super::backups_cmd::backup_list,
                super::backups_cmd::backup_verify,
                super::commands::list_audit,
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let w = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        // A simulated console with NO file service: probing must not enable anything.
        let v = call(&w, "connect_simulator", json!({"scenarioId": "stock"})).unwrap();
        assert_eq!(
            cap(&v, "homebrew.ftp_read")["status"],
            "unknown",
            "unprobed until the user connects"
        );
        let v = call(&w, "files_connect", json!({"user": null, "password": null})).unwrap();
        assert_eq!(cap(&v, "homebrew.ftp_read")["status"], "unavailable");

        // Auth failure scenario.
        call(
            &w,
            "connect_simulator",
            json!({"scenarioId": "ftp_auth_failure"}),
        )
        .unwrap();
        let v = call(&w, "files_connect", json!({"user": "x", "password": "bad"})).unwrap();
        assert!(cap(&v, "homebrew.ftp_read")["reason"]
            .as_str()
            .unwrap()
            .contains("rejected the login"));

        // Homebrew scenario: read-only until explicitly unlocked.
        call(&w, "connect_simulator", json!({"scenarioId": "homebrew"})).unwrap();
        let v = call(&w, "files_connect", json!({"user": null, "password": null})).unwrap();
        assert_eq!(cap(&v, "homebrew.ftp_read")["status"], "available");
        assert_eq!(v["write_unlocked"], false);
        let root = call(&w, "files_list_remote", json!({"path": "/"})).unwrap();
        assert!(root.as_array().unwrap().iter().any(|e| e["name"] == "data"));
        assert!(
            call(&w, "files_list_remote", json!({"path": "/data/../etc"})).is_err(),
            "traversal refused"
        );

        let local = data.path().join("hello.txt");
        std::fs::write(&local, b"hello").unwrap();
        let op = json!({"kind": "upload", "local": local.display().to_string(), "remote": "/data/homebrew/hello.txt"});
        let locked = call(&w, "files_plan", json!({"op": op})).unwrap_err();
        assert!(locked.as_str().unwrap().contains("locked"), "{locked}");
        assert!(call(
            &w,
            "files_set_write_lock",
            json!({"unlocked": true, "acknowledgeUnencrypted": false})
        )
        .is_err());
        let v = call(
            &w,
            "files_set_write_lock",
            json!({"unlocked": true, "acknowledgeUnencrypted": true}),
        )
        .unwrap();
        assert_eq!(v["write_unlocked"], true);

        // Protected and outside-allowlist targets are refused at PLAN time.
        for bad in ["/system/sys.dat", "/user/x", "/data/../system/x"] {
            let e = call(
                &w,
                "files_plan",
                json!({"op": {"kind": "upload", "local": local.display().to_string(), "remote": bad}}),
            );
            assert!(e.is_err(), "{bad} should be refused");
        }
        // A bogus plan id cannot be applied.
        assert!(call(&w, "files_apply", json!({"planId": "deadbeef"})).is_err());

        // PLAN -> CONFIRM(apply) -> VERIFY; a plan is single-use.
        let pv = call(&w, "files_plan", json!({"op": op})).unwrap();
        assert!(pv["plan"]["warnings"][0]
            .as_str()
            .unwrap()
            .contains("not encrypted"));
        let id = pv["plan_id"].as_str().unwrap().to_owned();
        let res = call(&w, "files_apply", json!({"planId": id})).unwrap();
        assert_eq!(res["outcome"]["verified"], true);
        assert!(
            call(&w, "files_apply", json!({"planId": id})).is_err(),
            "plan consumed"
        );

        // Overwrite => backup manifest created, verifiable, and audit recorded.
        std::fs::write(&local, b"hello again").unwrap();
        let pv = call(&w, "files_plan", json!({"op": op})).unwrap();
        assert_eq!(pv["plan"]["overwrites"], true);
        let res = call(&w, "files_apply", json!({"planId": pv["plan_id"]})).unwrap();
        let bid = res["backup_id"].as_str().expect("backup id").to_owned();
        assert_eq!(
            call(&w, "backup_verify", json!({"id": bid})).unwrap()["ok"],
            true
        );
        assert_eq!(
            call(&w, "backup_list", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let audit = call(&w, "list_audit", json!({})).unwrap();
        let acts: Vec<&str> = audit
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["action"].as_str().unwrap())
            .collect();
        assert!(
            acts.contains(&"unlock_writes")
                && acts.iter().filter(|a| **a == "files_apply").count() >= 2,
            "{acts:?}"
        );

        // Unauthorized scenario: service refuses writes -> clear failure, write marked refused.
        call(
            &w,
            "connect_simulator",
            json!({"scenarioId": "unauthorized"}),
        )
        .unwrap();
        call(&w, "files_connect", json!({"user": null, "password": null})).unwrap();
        call(
            &w,
            "files_set_write_lock",
            json!({"unlocked": true, "acknowledgeUnencrypted": true}),
        )
        .unwrap();
        let pv = call(
            &w,
            "files_plan",
            json!({"op": {"kind": "mkdir", "path": "/data/newdir"}}),
        )
        .unwrap();
        let e = call(&w, "files_apply", json!({"planId": pv["plan_id"]})).unwrap_err();
        assert!(e.as_str().unwrap().contains("550"), "{e}");
    }
}
