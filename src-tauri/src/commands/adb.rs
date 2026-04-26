use adb_client::{ADBDeviceExt, server::ADBServer};
use std::sync::Arc;
use tauri::{Emitter, Manager};

pub struct AdbState(pub Arc<std::sync::Mutex<ADBServer>>);

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
    let lock = Arc::clone(&state.0);

    tokio::task::spawn_blocking(move || {
        let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
        let mut device = server
            .get_device_by_name(&serial)
            .map_err(|e| e.to_string())?;

        let escpath = path.replace('\'', "'\\''");
        device
            .shell_command(&format!("rm -rf '{escpath}'"), None, None)
            .map_err(|e| e.to_string())?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
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
    let lock = Arc::clone(&state.0);

    tokio::task::spawn_blocking(move || {
        let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
        let mut device = server
            .get_device_by_name(&serial)
            .map_err(|e| e.to_string())?;

        let mut output = Vec::new();
        let escpath = path.replace('\'', "'\\''");

        device
            .shell_command(
                &format!("ls -1apF '{escpath}'"),
                Some(&mut output),
                None,
            )
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output);
        let base_path = if path.ends_with('/') {
            path
        } else {
            format!("{path}/")
        };

        let files = output_str
            .lines()
            .filter_map(|line| {
                let line = line.trim().replace('\r', "");
                if line.is_empty()
                    || line.contains("Permission denied")
                    || line == "./"
                    || line == "../"
                {
                    return None;
                }

                let is_dir = line.ends_with('/');
                let name = line.trim_end_matches(['/', '*', '@', '|', '=']).to_string();

                Some(AdbFileEntry {
                    is_hidden: name.starts_with('.'),
                    path: format!("{base_path}{name}"),
                    name,
                    is_dir,
                })
            })
            .collect();

        Ok(files)
    })
    .await
    .map_err(|e| e.to_string())?
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
    let state = app.state::<AdbState>();
    let server_lock = Arc::clone(&state.0);

    tauri::async_runtime::spawn(async move {
        loop {
            let lock = Arc::clone(&server_lock);

            let devices_result = tokio::task::spawn_blocking(move || {
                let mut server = lock.lock().ok()?;
                server.devices().ok().map(|devices| {
                    devices
                        .into_iter()
                        .map(|d| DeviceObj {
                            serial: d.identifier,
                            state: d.state.to_string(),
                        })
                        .collect::<Vec<_>>()
                })
            })
            .await;

            if let Ok(Some(devices)) = devices_result {
                let _ = app.emit("adb_update", &devices);
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}
