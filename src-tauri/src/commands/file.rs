use serde::Serialize;
use std::fs;
use std::path::Path;
use sysinfo::Disks;

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    has_permission: bool,
}

#[derive(Serialize)]
pub struct Partition {
    name: String,
    mount_point: String,
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let target_path = if path.is_empty() {
        Path::new("/")
    } else {
        Path::new(&path)
    };

    let entries = fs::read_dir(target_path).map_err(|e| e.to_string())?;

    let mut file_list = Vec::new();

    for entry in entries.flatten() {
        let path_buf = entry.path();
        let metadata = entry.metadata().ok();

        let is_dir = metadata.as_ref().is_some_and(std::fs::Metadata::is_dir);

        let has_permission = if is_dir {
            fs::read_dir(&path_buf).is_ok()
        } else {
            fs::File::open(&path_buf).is_ok()
        };

        file_list.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: path_buf.to_string_lossy().into_owned(),
            is_dir,
            has_permission,
        });
    }

    file_list.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(file_list)
}

#[tauri::command]
pub async fn list_partitions() -> Vec<Partition> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter(|disk| {
            fs::read_dir(disk.mount_point()).is_ok()
        })
        .map(|disk| Partition {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
        }
    ).collect()
}