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

    let device_list = devices
        .into_iter()
        .map(|d| DeviceObj {
            serial: d.identifier,
            state: d.state.to_string(),
        })
        .collect();

    Ok(device_list)
}

#[tauri::command]
pub async fn list_adb_directory(state: tauri::State<'_, AdbState>, serial: String, path: String) -> Result<Vec<AdbFileEntry>, String> {
    let mut server = state.0.lock().unwrap();
    let mut device = server.get_device_by_name(&serial).map_err(|e| e.to_string())?;

    let mut output_buffer = Vec::new();

    device.shell_command(
        &format!("ls -1LF '{path}'"),
        Some(&mut output_buffer), 
        None
    ).map_err(|e| e.to_string())?;

    let output_str = String::from_utf8(output_buffer).map_err(|e| e.to_string())?;

    let files: Vec<AdbFileEntry> = output_str
        .lines()
        .filter(|line| !line.contains("Permission denied") && !line.is_empty())
        .map(|line| {
            let is_dir = line.ends_with('/');
            let name = line.trim_end_matches(['/', '*']).to_string();
            AdbFileEntry { name, is_dir }
        })
        .collect();

    Ok(files)
}