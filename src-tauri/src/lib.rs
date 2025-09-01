mod commands;

use commands::*;
use tauri_plugin_updater::UpdaterExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_updates_on_startup(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            read_text, 
            pick_file,
            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_for_updates_on_startup(app: tauri::AppHandle) {
    // Wait a bit for the app to fully start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("Update available: {} -> {}", update.current_version, update.version);
                    
                    // Download and install the update
                    let mut downloaded = 0;
                    match update.download_and_install(
                        |chunk_length, content_length| {
                            downloaded += chunk_length;
                            if let Some(total) = content_length {
                                let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                                println!("Download progress: {}%", percent);
                            }
                        },
                        || {
                            println!("Download finished, installing update...");
                        },
                    ).await {
                        Ok(_) => {
                            println!("Update installed successfully, restarting app...");
                            app.restart();
                        }
                        Err(e) => {
                            eprintln!("Failed to install update: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("No updates available");
                }
                Err(e) => {
                    eprintln!("Failed to check for updates: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize updater: {}", e);
        }
    }
}
