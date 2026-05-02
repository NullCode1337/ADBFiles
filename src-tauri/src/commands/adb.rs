use adb_client::{ADBDeviceExt, server::ADBServer};
use std::sync::Arc;
use tauri::Emitter;
use tokio::task::spawn_blocking;

pub struct AdbState(pub Arc<std::sync::Mutex<ADBServer>>);

#[derive(serde::Serialize, Clone)]
pub struct DeviceObj {
    pub name: String,
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

struct AdbPath;

impl AdbPath {
    fn join(base: &str, name: &str) -> String {
        let base = base.trim_end_matches('/');
        format!("{base}/{name}")
    }

    fn escape(path: &str) -> String {
        path.replace('\'', "'\\''")
    }

    fn trail(path: &str) -> String {
        if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{path}/")
        }
    }
}

pub fn adb_polling(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let app = app.clone();
            let _ = spawn_blocking(move || {
                let mut server = ADBServer::default();

                let _ = server.track_devices(|_| {
                    let devices = ADBServer::default()
                        .devices_long()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|d| DeviceObj {
                            name: d.model,
                            serial: d.identifier,
                            state: d.state.to_string(),
                        })
                        .collect::<Vec<DeviceObj>>();

                    let _ = app.emit("adb_update", devices);
                    Ok(())
                });
            })
            .await;

            // Wait a bit before re-loop if tracker exits
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}

#[tauri::command]
pub async fn delete_adb_file(
    state: tauri::State<'_, AdbState>,
    serial: String,
    path: String,
) -> Result<(), String> {
    let lock = Arc::clone(&state.0);

    spawn_blocking(move || {
        let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
        let mut device = server
            .get_device_by_name(&serial)
            .map_err(|e| e.to_string())?;

        device
            .shell_command(&format!("rm -rf '{}'", AdbPath::escape(&path)), None, None)
            .map_err(|e| e.to_string())?;

        Ok(())
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

#[tauri::command]
pub async fn list_adb_devices(state: tauri::State<'_, AdbState>) -> Result<Vec<DeviceObj>, String> {
    let lock = Arc::clone(&state.0);

    spawn_blocking(move || {
        let mut server = lock.lock().map_err(|_| "Poisoned lock".to_string())?;
        let devices = server.devices_long().map_err(|e| e.to_string())?;

        Ok(devices
            .into_iter()
            .map(|d| DeviceObj {
                name: d.model,
                serial: d.identifier,
                state: d.state.to_string(),
            })
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_adb_directory(
    state: tauri::State<'_, AdbState>,
    serial: String,
    path: String,
) -> Result<Vec<AdbFileEntry>, String> {
    let lock = Arc::clone(&state.0);

    spawn_blocking(move || {
        let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
        let mut device = server
            .get_device_by_name(&serial)
            .map_err(|e| e.to_string())?;

        let mut output = Vec::new();

        device
            .shell_command(
                &format!("ls -1ap '{}'", AdbPath::escape(&path)),
                Some(&mut output),
                None,
            )
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output);
        let base_path = AdbPath::trail(&path);

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
                let name = line.trim_end_matches('/').to_string();

                Some(AdbFileEntry {
                    is_hidden: name.starts_with('.'),
                    path: AdbPath::join(&base_path, &name),
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
pub async fn adb_push(
    state: tauri::State<'_, AdbState>,
    serial: String,
    src: String,
    dest: String,
    is_dir: bool,
) -> Result<(), String> {
    if is_dir {
        tokio::process::Command::new("adb")
            .args(["-s", &serial, "push", &src, &dest])
            .output()
            .await
            .map_err(|e| format!("Folder push failed: {e}"))?;
        Ok(())
    } else {
        let lock = Arc::clone(&state.0);
        spawn_blocking(move || {
            let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
            let mut device = server
                .get_device_by_name(&serial)
                .map_err(|e| e.to_string())?;

            let file_name = std::path::Path::new(&src)
                .file_name()
                .ok_or("Invalid path")?
                .to_string_lossy();
            let dest_path = AdbPath::join(&dest, &file_name);

            let file = std::fs::File::open(&src).map_err(|e| e.to_string())?;
            let mut reader = std::io::BufReader::new(file);
            device
                .push(&mut reader, &dest_path)
                .map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

#[tauri::command]
pub async fn adb_pull(
    state: tauri::State<'_, AdbState>,
    serial: String,
    src: String,
    dest: String,
    is_dir: bool,
) -> Result<(), String> {
    if is_dir {
        tokio::process::Command::new("adb")
            .args(["-s", &serial, "pull", &src, &dest])
            .output()
            .await
            .map_err(|e| format!("Folder pull failed: {e}"))?;
        Ok(())
    } else {
        let lock = Arc::clone(&state.0);
        spawn_blocking(move || {
            let mut server = lock.lock().map_err(|_| "Poisoned lock")?;
            let mut device = server
                .get_device_by_name(&serial)
                .map_err(|e| e.to_string())?;

            let file_name = src
                .trim_end_matches('/')
                .split('/')
                .next_back()
                .ok_or("Invalid path")?;
            let dest_path = std::path::Path::new(&dest).join(file_name);

            let file = std::fs::File::create(&dest_path).map_err(|e| e.to_string())?;
            let mut writer = std::io::BufWriter::new(file);
            if let Err(e) = device.pull(&src, &mut writer) {
                let _ = std::fs::remove_file(&dest_path);
                return Err(e.to_string());
            }

            Ok(())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
