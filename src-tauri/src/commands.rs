use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::DialogExt;

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
pub fn pick_csv_file(app: AppHandle) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let file_path = dialog
        .file()
        .add_filter("CSV files", &["csv"])
        .add_filter("All files", &["*"])
        .blocking_pick_file();
    
    Ok(file_path.map(|p| p.to_string()))
}

#[tauri::command]
pub fn validate_csv(content: String) -> Result<CsvValidationResult, String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return Ok(CsvValidationResult {
            is_valid: false,
            message: "File is empty".to_string(),
            row_count: 0,
        });
    }
    
    // Check if it's a valid CSV format
    let mut row_count = 0;
    let mut has_header = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        
        row_count += 1;
        
        // Basic CSV validation - check for proper comma separation
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 2 {
            return Ok(CsvValidationResult {
                is_valid: false,
                message: format!("Row {} has insufficient columns (minimum 2 required)", i + 1),
                row_count: row_count - 1,
            });
        }
        
        // Check if first row looks like a header
        if i == 0 && fields.iter().any(|f| f.trim().chars().all(|c| c.is_alphabetic() || c == ' ' || c == '_')) {
            has_header = true;
        }
    }
    
    if row_count < 2 {
        return Ok(CsvValidationResult {
            is_valid: false,
            message: "CSV file must have at least 2 rows (header + data)".to_string(),
            row_count,
        });
    }
    
    Ok(CsvValidationResult {
        is_valid: true,
        message: if has_header {
            format!("Valid CSV file with {} data rows", row_count - 1)
        } else {
            format!("Valid CSV file with {} rows", row_count)
        },
        row_count,
    })
}

#[derive(Serialize, Deserialize)]
pub struct CsvValidationResult {
    pub is_valid: bool,
    pub message: String,
    pub row_count: usize,
}

#[tauri::command]
pub async fn import_csv_from_menu(app: AppHandle) -> Result<String, String> {
    // Open file picker dialog
    let file_path = match pick_csv_file(app.clone()) {
        Ok(Some(path)) => path,
        Ok(None) => {
            println!("File selection cancelled by user");
            return Ok("File selection cancelled".to_string());
        },
        Err(e) => return Err(format!("Failed to pick file: {}", e)),
    };
    
    // Read the selected file
    let content = match read_text(file_path.clone(), 2_000_000) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read file: {}", e)),
    };
    
    // Validate the CSV content
    let validation = match validate_csv(content) {
        Ok(result) => result,
        Err(e) => return Err(format!("Failed to validate CSV: {}", e)),
    };
    
    if validation.is_valid {
        let message = format!("SUCCESS: {}", validation.message);
        // Show success dialog
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.eval(&format!("alert('{}')", message));
        }
        Ok(message)
    } else {
        let message = format!("ERROR: Invalid CSV - {}", validation.message);
        // Show error dialog
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.eval(&format!("alert('{}')", message));
        }
        Err(message)
    }
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
