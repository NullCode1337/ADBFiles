use serde::Serialize;
use std::path::{Component, Path, PathBuf};

#[derive(Serialize)]
pub struct PathSegment {
    name: String,
    path: String,
}

#[derive(Serialize)]
pub struct PathMetadata {
    segments: Vec<PathSegment>,
    parent: Option<String>,
}

#[tauri::command]
pub fn get_path_metadata(path: &str, is_adb: bool) -> PathMetadata {
    let mut segments = Vec::new();
    let path_obj = Path::new(&path);

    if is_adb {
        segments.push(PathSegment {
            name: "root".into(),
            path: "/".into(),
        });
        let mut current = PathBuf::from("/");
        for component in path_obj.components() {
            if let Component::Normal(name) = component {
                current.push(name);
                segments.push(PathSegment {
                    name: name.to_string_lossy().into_owned(),
                    path: current.to_string_lossy().replace('\\', "/"),
                });
            }
        }
    } else {
        let mut current = PathBuf::new();
        for component in path_obj.components() {
            match component {
                Component::Prefix(prefix) => {
                    let p = prefix.as_os_str().to_string_lossy().into_owned();
                    current.push(&p);
                    segments.push(PathSegment {
                        name: p,
                        path: current.to_string_lossy().into_owned(),
                    });
                }
                Component::RootDir => {
                    if cfg!(not(windows)) {
                        segments.push(PathSegment {
                            name: "root".into(),
                            path: "/".into(),
                        });
                    }
                    current.push(std::path::MAIN_SEPARATOR.to_string());
                }
                Component::Normal(name) => {
                    current.push(name);
                    segments.push(PathSegment {
                        name: name.to_string_lossy().into_owned(),
                        path: current.to_string_lossy().into_owned(),
                    });
                }
                _ => {}
            }
        }
    }

    PathMetadata {
        segments,
        parent: path_obj
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_string_lossy().into_owned()),
    }
}

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
