use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    has_permission: bool,
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let target_path = Path::new(&path);
    let entries = fs::read_dir(target_path).map_err(|e| e.to_string())?;

    let mut file_list = Vec::new();

    for entry in entries.flatten() {
        let metadata = entry.metadata();
        let is_dir = metadata
            .as_ref()
            .map(std::fs::Metadata::is_dir)
            .unwrap_or(false);

        // Check permission by attempting to read the directory if it's a folder
        let has_permission = if is_dir {
            fs::read_dir(entry.path()).is_ok()
        } else {
            true
        };

        file_list.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path().to_string_lossy().into_owned(),
            is_dir,
            has_permission,
        });
    }

    // Sort: Folders first, then alphabetical
    file_list.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(file_list)
}
