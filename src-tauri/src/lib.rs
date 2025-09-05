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

    let mut builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());
    
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                // Setup desktop menu bar
                let handle = app.handle();
                let import_menu = SubmenuBuilder::new(handle, "Import")
                    .text("import_csv", "CSV...")
                    .build()?;
                
                let file_menu = SubmenuBuilder::new(handle, "File")
                    .item(&import_menu)
                    .separator()
                    .text("quit", "Quit")
                    .build()?;

                let menu = MenuBuilder::new(handle)
                    .item(&file_menu)
                    .build()?;

                // Set the menu for the main window
                if let Some(window) = app.get_webview_window("main") {
                    let app_handle = app.handle().clone();
                    window.set_menu(menu)?;
                    window.on_menu_event(move |_window, event| match event.id.as_ref() {
                        "import_csv" => {
                            println!("Import CSV menu item was clicked");
                            // Call the import function directly from Rust
                            let app_handle_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                match import_csv_from_menu(app_handle_clone.clone()).await {
                                    Ok(result) => {
                                        if result == "File selection cancelled" {
                                            println!("CSV import cancelled by user");
                                        } else {
                                            println!("CSV import completed successfully");
                                        }
                                    },
                                    Err(e) => {
                                        println!("CSV import failed: {}", e);
                                        // Show error dialog
                                        if let Some(window) = app_handle_clone.get_webview_window("main") {
                                            let _ = window.eval(&format!("alert('Import failed: {}')", e));
                                        }
                                    }
                                }
                            });
                        }
                        "quit" => {
                            println!("quit menu item was clicked");
                            app_handle.exit(0);
                        }
                        _ => {
                            println!("menu item {:?} not handled", event.id);
                        }
                    });
                }

                // Setup tray icon
                let tray_menu = MenuBuilder::new(handle)
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
                    .menu(&tray_menu)
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

            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    check_for_updates_on_startup(handle).await;
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            read_text,
            pick_csv_file,
            validate_csv,
            import_csv_from_menu,
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
