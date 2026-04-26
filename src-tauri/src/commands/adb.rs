use adb_client::{server::ADBServer, ADBDeviceExt};
use tauri::{Emitter, Manager};

pub struct AdbState(pub std::sync::Mutex<ADBServer>);

#[derive(serde::Serialize)]
pub struct DeviceObj {
    pub serial: String,
    pub state: String,
}

#[derive(serde::Serialize)]
pub struct AdbFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_hidden: bool,
}

#[tauri::command]
pub async fn delete_adb_file(
    state: tauri::State<'_, AdbState>,
    serial: String,
    path: String,
) -> Result<(), String> {
    let mut server = state.0.lock().unwrap();
    let mut device = server.get_device_by_name(&serial).map_err(|e| e.to_string())?;
    
    device.shell_command(
        &format!("rm -rf '{}'", path.replace('\'', "'\\''")),
        None,
        None,
    ).map_err(|e| e.to_string())?;

    Ok(())
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
pub async fn list_adb_directory(
    state: tauri::State<'_, AdbState>,
    serial: String,
    path: String,
) -> Result<Vec<AdbFileEntry>, String> {
    let mut server = state.0.lock().unwrap();
    let mut device = server
        .get_device_by_name(&serial)
        .map_err(|e| e.to_string())?;

    let mut output = Vec::new();
    device
        .shell_command(
            &format!("ls -1apF '{}'", path.replace('\'', "'\\''")),
            Some(&mut output),
            None,
        )
        .map_err(|e| e.to_string())?;

    let output_str = String::from_utf8_lossy(&output);
    let base_path = if path.ends_with('/') { path.clone() } else { format!("{path}/") };

    let files: Vec<AdbFileEntry> = output_str
        .lines()
        .map(|line| line.trim().replace('\r', ""))
        .filter_map(|line| {
            let line = line.trim();
            let name = line.trim_end_matches(['/', '*', '@', '|', '=']).to_string();
            if line.is_empty()
                || line.contains("Permission denied")
                || line == "./"
                || line == "../"
            {
                return None;
            }
            Some(AdbFileEntry {
                is_dir: line.ends_with('/'),
                is_hidden: name.starts_with('.'),
                name: name.clone(),
                path: format!("{base_path}{name}"),
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

pub fn adb_polling(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AdbState>();
            
            let devices_result = {
                let mut server = state.0.lock().unwrap();
                server.devices().map(|devices| {
                    devices.into_iter()
                        .map(|d| DeviceObj {
                            serial: d.identifier,
                            state: d.state.to_string(),
                        })
                        .collect::<Vec<DeviceObj>>()
                })
            };

            if let Ok(devices) = devices_result {
                let _ = app.emit("adb_update", &devices);
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}