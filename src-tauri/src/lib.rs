#![allow(unexpected_cfgs)]

mod clipboard;
mod commands;
mod config;
mod database;
mod models;
mod state;
mod tray;
mod updater;
mod utils;

#[cfg(test)]
mod state_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod app_paths_tests;

#[cfg(test)]
mod capture_runtime_tests;

#[cfg(test)]
mod capture_policy_tests;

use commands::*;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Manager, Window, WindowEvent,
};

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

async fn handle_menu_event(app_handle: &AppHandle, event_id: &str) {
    log::info!("Menu event: {}", event_id);

    let state = app_handle.state::<AppState>();

    match event_id {
        "Copy" => {
            // Get selected text from frontend and copy to clipboard
            if let Err(e) = app_handle.emit("menu_copy", ()) {
                log::error!("Failed to emit copy event: {}", e);
            }
        }
        "Paste" => {
            // Paste from clipboard to current context
            if let Err(e) = app_handle.emit("menu_paste", ()) {
                log::error!("Failed to emit paste event: {}", e);
            }
        }
        "Cut" => {
            // Cut selected text
            if let Err(e) = app_handle.emit("menu_cut", ()) {
                log::error!("Failed to emit cut event: {}", e);
            }
        }
        "SelectAll" => {
            // Select all text in current context
            if let Err(e) = app_handle.emit("menu_select_all", ()) {
                log::error!("Failed to emit select all event: {}", e);
            }
        }
        "clear_history" => {
            if let Err(e) = state.clear_history().await {
                log::error!("Failed to clear history: {}", e);
            } else {
                // Emit event to refresh frontend
                let _ = app_handle.emit("history_cleared", ());
            }
        }
        "show_statistics" => match state.get_statistics().await {
            Ok(stats) => {
                if let Err(e) = app_handle.emit("show_statistics", &stats) {
                    log::error!("Failed to emit statistics event: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to get statistics: {}", e);
            }
        },
        "show_preferences" => {
            if let Err(e) = app_handle.emit("show_preferences", ()) {
                log::error!("Failed to emit preferences event: {}", e);
            }
        }
        "toggle_monitoring" => {
            let is_monitoring = state.is_monitoring().await;
            let result = if is_monitoring {
                state.stop_monitoring().await
            } else {
                state.start_monitoring().await
            };

            if let Err(e) = result {
                log::error!("Failed to toggle monitoring: {}", e);
            } else {
                // Emit event to update menu label
                let new_is_monitoring = state.is_monitoring().await;
                if let Err(e) = app_handle.emit("monitoring_toggled", new_is_monitoring) {
                    log::error!("Failed to emit monitoring toggle event: {}", e);
                }
            }
        }
        _ => {
            log::warn!("Unknown menu event: {}", event_id);
        }
    }
}

fn handle_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        log::info!("Window close requested, hiding instead of closing");
        // Prevent the default close behavior
        api.prevent_close();

        // On macOS, use AppHandle::hide() for better system integration
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = window.app_handle().hide() {
                log::error!("Failed to hide app: {}", e);
            }
        }

        // On other platforms, use window.hide()
        #[cfg(not(target_os = "macos"))]
        {
            if let Err(e) = window.hide() {
                log::error!("Failed to hide window: {}", e);
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("clipboard-app".to_string()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, _event| {
                    log::info!("Global shortcut triggered: {:?}", shortcut);

                    // Show/focus the main window when global shortcut is pressed
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }

                    // Also emit event to frontend
                    let _ = app.emit("global-shortcut", shortcut);
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // Set macOS app to accessory mode (hide dock icon)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);
            // Create macOS menu
            #[cfg(target_os = "macos")]
            {
                let app_name_submenu = Submenu::with_items(
                    app,
                    "剪切板管理器",
                    true,
                    &[
                        &PredefinedMenuItem::about(app, Some("关于剪切板管理器"), None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "show_preferences",
                            "偏好设置...",
                            true,
                            Some("CmdOrCtrl+,"),
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, Some("隐藏剪切板管理器"))?,
                        &PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?,
                        &PredefinedMenuItem::show_all(app, Some("显示全部"))?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::quit(app, Some("退出剪切板管理器"))?,
                    ],
                )?;

                let edit_submenu = Submenu::with_items(
                    app,
                    "编辑",
                    true,
                    &[
                        &PredefinedMenuItem::copy(app, Some("拷贝"))?,
                        &PredefinedMenuItem::paste(app, Some("粘贴"))?,
                        &PredefinedMenuItem::cut(app, Some("剪切"))?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::select_all(app, Some("全选"))?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "clear_history",
                            "清空历史",
                            true,
                            Some("CmdOrCtrl+Shift+Delete"),
                        )?,
                    ],
                )?;

                let view_submenu = Submenu::with_items(
                    app,
                    "查看",
                    true,
                    &[&MenuItem::with_id(
                        app,
                        "show_statistics",
                        "查看统计",
                        true,
                        Some("CmdOrCtrl+I"),
                    )?],
                )?;

                let control_submenu = Submenu::with_items(
                    app,
                    "控制",
                    true,
                    &[&MenuItem::with_id(
                        app,
                        "toggle_monitoring",
                        "开始监听",
                        true,
                        Some("CmdOrCtrl+Space"),
                    )?],
                )?;

                let menu = Menu::with_items(
                    app,
                    &[
                        &app_name_submenu,
                        &edit_submenu,
                        &view_submenu,
                        &control_submenu,
                    ],
                )?;

                app.set_menu(menu)?;
            }

            tauri::async_runtime::block_on(async {
                // Load .env file manually in development
                if cfg!(debug_assertions) {
                    let _ = dotenvy::dotenv();
                }

                // Initialize Aptabase plugin
                let aptabase_key = std::env::var("APTABASE_APP_KEY")
                    .unwrap_or_else(|_| "A-DEV-0000000000".to_string());

                // Log Aptabase configuration (with masked key for security)
                let key_info = if aptabase_key == "A-DEV-0000000000" {
                    "development key".to_string()
                } else if aptabase_key.len() > 8 {
                    format!(
                        "{}...{}",
                        &aptabase_key[..5],
                        &aptabase_key[aptabase_key.len() - 4..]
                    )
                } else {
                    "invalid key".to_string()
                };

                log::info!("Aptabase initialized with: {}", key_info);
                log::info!(
                    "Build mode: {}",
                    if cfg!(debug_assertions) {
                        "Debug"
                    } else {
                        "Release"
                    }
                );

                let _ = app
                    .handle()
                    .plugin(tauri_plugin_aptabase::Builder::new(&aptabase_key).build());

                let state = AppState::new().await?;

                let app_handle = app.handle().clone();
                state.set_app_handle(app_handle.clone());

                // Load config and register global shortcut on startup
                if let Ok(config) = state.get_config().await {
                    if !config.global_shortcut.is_empty() {
                        if let Err(e) = state
                            .register_global_shortcut(
                                app_handle.clone(),
                                config.global_shortcut.clone(),
                            )
                            .await
                        {
                            log::error!("Failed to register global shortcut on startup: {}", e);
                        } else {
                            log::info!(
                                "Global shortcut registered on startup: {}",
                                config.global_shortcut
                            );
                        }
                    }
                }

                app.manage(state);

                // Create system tray
                tray::create_tray_icon(app.handle())?;

                Ok::<(), Box<dyn std::error::Error>>(())
            })?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            let app_handle = app.clone();
            let event_id = event.id.0.clone();
            tauri::async_runtime::spawn(async move {
                handle_menu_event(&app_handle, &event_id).await;
            });
        })
        .on_window_event(|window, event| {
            handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            start_monitoring,
            stop_monitoring,
            get_clipboard_history,
            toggle_favorite,
            delete_entry,
            clear_history,
            get_statistics,
            copy_to_clipboard,
            paste_text,
            paste_image,
            get_image_url,
            open_file_with_system,
            get_app_icon,
            convert_and_scale_image,
            copy_converted_image,
            fetch_url_content,
            check_ffprobe_available,
            extract_media_metadata,
            resolve_url_preview,
            decode_base64_preview,
            inspect_media_source,
            get_config,
            update_config,
            get_cache_statistics,
            register_global_shortcut,
            unregister_global_shortcut,
            set_auto_startup,
            get_auto_startup_status,
            cleanup_expired_entries,
            get_installed_applications,
            get_common_excluded_apps,
            validate_shortcut,
            check_for_update,
            install_update,
            should_check_for_updates,
            set_window_title,
            get_log_content,
            clear_logs,
            set_log_level,
            get_current_log_level
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            use tauri_plugin_aptabase::EventTracker;

            match event {
                tauri::RunEvent::Ready => {
                    // Use async runtime to send events in proper context
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        // Wait a moment to ensure plugin is fully ready
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = handle.track_event("app_started", None);
                    });
                }
                tauri::RunEvent::Exit => {
                    let _ = app_handle.track_event("app_exited", None);
                }
                tauri::RunEvent::Reopen {
                    has_visible_windows,
                    ..
                } => {
                    log::info!(
                        "App reopened from dock, showing main window (has_visible_windows: {})",
                        has_visible_windows
                    );
                    // Show the main window when dock icon is clicked
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if let Err(e) = window.show() {
                            log::error!("Failed to show window: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            log::error!("Failed to set window focus: {}", e);
                        }
                        if let Err(e) = window.unminimize() {
                            log::error!("Failed to unminimize window: {}", e);
                        }
                    } else {
                        log::error!("Main window not found");
                    }
                }
                _ => {}
            }
        });
}
