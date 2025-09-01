use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

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

// Optional updater commands for manual testing (not used in automatic updates)
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum UpdaterError {
    #[error(transparent)]
    Updater(#[from] tauri_plugin_updater::Error),
    #[error("there is no pending update")]
    NoPendingUpdate,
}

impl Serialize for UpdaterError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

type UpdaterResult<T> = std::result::Result<T, UpdaterError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    version: String,
    current_version: String,
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> UpdaterResult<Option<UpdateMetadata>> {
    let update = app.updater()?.check().await?;

    let update_metadata = update.as_ref().map(|update| UpdateMetadata {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
    });

    Ok(update_metadata)
}
