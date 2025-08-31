use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub date: String,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
}

#[tauri::command]
pub fn read_text(path: String, max_bytes: u64) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err("Not a file".into());
    }
    let md = fs::metadata(p).map_err(|e| e.to_string())?;
    if md.len() > max_bytes {
        return Err("File too large".into());
    }
    let content = fs::read_to_string(p).map_err(|e| e.to_string())?;
    Ok(content)
}

#[tauri::command]
pub fn pick_file() -> Result<Option<String>, String> {
    // TODO: Implement file dialog using tauri_plugin_dialog
    // For now, return None to indicate no file selected
    Ok(None)
}
