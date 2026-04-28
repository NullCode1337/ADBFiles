mod commands;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .manage(crate::commands::adb::AdbState(
            std::sync::Arc::new(
                std::sync::Mutex::new(
                    adb_client::server::ADBServer::default(),
                )
            )))
        .invoke_handler(tauri::generate_handler![
            commands::adb::adb_pull,
            commands::adb::adb_push,
            commands::adb::delete_adb_file,
            commands::adb::launch_scrcpy,
            commands::adb::list_adb_devices,
            commands::adb::list_adb_directory,
            commands::file::delete_desktop_file,
            commands::file::list_directory,
            commands::file::list_partitions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
