mod commands;
use tauri::{Emitter, Manager};

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let path = argv[2].clone();
            let handle = app.clone();

            let state = handle.state::<commands::adb::AdbState>();
            let server = std::sync::Arc::clone(&state.0);

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("ctx-push", &path);
                let _ = window.unminimize();
                let _ = window.set_focus();
            } else {
                tauri::async_runtime::spawn(async move {
                    let _ = commands::utils::notify(handle, format!("Pushing to Android: {path}"))
                        .await;
                    let _ = commands::adb::ctx_push(server, path).await;
                });
            }
        }))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                commands::adb::adb_polling(app.handle().clone());
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(crate::commands::adb::AdbState(std::sync::Arc::new(
            std::sync::Mutex::new(adb_client::server::ADBServer::default()),
        )))
        .invoke_handler(tauri::generate_handler![
            commands::adb::adb_pull,
            commands::adb::adb_push,
            commands::adb::delete_adb_file,
            commands::adb::launch_scrcpy,
            commands::adb::list_adb_devices,
            commands::adb::list_adb_directory,
            commands::file::delete_desktop_file,
            commands::file::is_directory,
            commands::file::list_directory,
            commands::file::list_partitions,
            commands::file::open_file,
            commands::utils::get_path_metadata,
            commands::utils::notify,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
