#[tauri::command]
pub async fn notify(_app: tauri::AppHandle, body: String) -> Result<(), String> {
    let title = "ADBFiles";

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("notify-send")
            .arg("--app-name=ADBFiles")
            .arg(title)
            .arg(&body)
            .spawn()
            .map_err(|e| format!("Notification error: {e}"))?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_notification::NotificationExt;
        _app.notification()
            .builder()
            .title(title)
            .body(&body)
            .show()
            .map_err(|e| format!("Notification error: {e}"))?;
    }

    Ok(())
}