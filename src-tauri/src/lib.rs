mod commands;

use commands::*;
use tauri::{menu::*, tray::TrayIconBuilder, Manager};
use tauri_plugin_updater::UpdaterExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main")
                       .expect("no main window")
                       .set_focus();
        }));
    }

    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                // Setup tray icon
                let handle = app.handle();
                let menu = MenuBuilder::new(handle)
                    .item(&MenuItem::new(
                        handle,
                        &format!("NoBullFit v{}", env!("CARGO_PKG_VERSION")),
                        false,
                        None::<&str>,
                    )?)
                    .separator()
                    .text("quit", "Quit")
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            println!("quit menu item was clicked");
                            app.exit(0);
                        }
                        "version" => {
                            // Version item is disabled, no action needed
                            println!("version menu item was clicked (disabled)");
                        }
                        _ => {
                            println!("menu item {:?} not handled", event.id);
                        }
                    })
                    .build(app)?;
            }

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
                    println!(
                        "Update available: {} -> {}",
                        update.current_version, update.version
                    );

                    // Download and install the update
                    let mut downloaded = 0;
                    match update
                        .download_and_install(
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
                        )
                        .await
                    {
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
