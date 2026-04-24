use adb_client::{ADBDeviceExt, server::ADBServer};

pub struct AdbState(pub std::sync::Mutex<ADBServer>);

#[derive(serde::Serialize)]
pub struct DeviceObj {
    pub serial: String,
    pub state: String,
}

#[derive(serde::Serialize)]
pub struct AdbFileEntry {
    pub name: String,
    pub is_dir: bool,
}

#[tauri::command]
pub async fn list_adb_devices(state: tauri::State<'_, AdbState>) -> Result<Vec<DeviceObj>, String> {
    let mut server = state.0.lock().unwrap();
    let devices = server.devices().map_err(|e| e.to_string())?;

    Ok(devices
        .into_iter()
        .map(|d| DeviceObj {
            serial: d.identifier,
            state: d.state.to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn list_adb_directory(state: tauri::State<'_, AdbState>, serial: String, path: String) -> Result<Vec<AdbFileEntry>, String> {
    let mut server = state.0.lock().unwrap();
    let mut device = server.get_device_by_name(&serial).map_err(|e| e.to_string())?;

    let mut output = Vec::new();
    device.shell_command(
        &format!("ls -1apF '{}'", path.replace('\'', "'\\''")),
        Some(&mut output),
        None,
    ).map_err(|e| e.to_string())?;

    let output_str = String::from_utf8_lossy(&output);

    let files: Vec<AdbFileEntry> = output_str.lines()
    .map(|line| line.trim().replace('\r', ""))
    .filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.contains("Permission denied") { return None; }
        Some(AdbFileEntry {
            is_dir: line.ends_with('/'),
            name: line.trim_end_matches(['/', '*', '@']).to_string(),
        })
    })
    .collect();

    Ok(files)
}

#[tauri::command]
pub async fn launch_scrcpy(serial: String) -> Result<(), String> {
    let _child = tokio::process::Command::new("scrcpy")
        .arg("-s")
        .arg(&serial)
        .spawn()
        .map_err(|e| format!("Failed to start scrcpy: {e}"))?;
    
    Ok(())
}