use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn notify(app: tauri::AppHandle, body: &str) {
    app.notification()
        .builder()
        .title("ADBFiles")
        .body(body)
        .icon("icons/128x128.png")
        .show()
        .unwrap();
}