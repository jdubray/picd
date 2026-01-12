// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pictd_core::{get_downloads_dir, list_saved_images, ClipboardMonitor, ImageInfo};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

struct AppState {
    monitor: ClipboardMonitor,
    save_dir: Mutex<String>,
}

#[derive(Clone, Serialize)]
struct Settings {
    save_dir: String,
    is_monitoring: bool,
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    Settings {
        save_dir: state.save_dir.lock().unwrap().clone(),
        is_monitoring: state.monitor.is_running(),
    }
}

#[tauri::command]
fn set_save_directory(path: String, state: State<AppState>) -> Result<(), String> {
    let mut save_dir = state.save_dir.lock().map_err(|e| e.to_string())?;
    *save_dir = path;
    Ok(())
}

#[tauri::command]
fn start_monitoring(app_handle: AppHandle, state: State<AppState>) -> Result<(), String> {
    let save_dir = state.save_dir.lock().map_err(|e| e.to_string())?.clone();
    let handle = app_handle.clone();
    state.monitor.start(save_dir, move |info| {
        let _ = handle.emit("image-saved", &info);
    });
    Ok(())
}

#[tauri::command]
fn stop_monitoring(state: State<AppState>) -> Result<(), String> {
    state.monitor.stop();
    Ok(())
}

#[tauri::command]
fn get_saved_images(state: State<AppState>) -> Vec<ImageInfo> {
    let save_dir = state.save_dir.lock().unwrap().clone();
    list_saved_images(&save_dir)
}

#[tauri::command]
fn open_image(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            monitor: ClipboardMonitor::new(),
            save_dir: Mutex::new(get_downloads_dir().to_string_lossy().to_string()),
        })
        .setup(|app| {
            // Create tray menu
            let show_i = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let pause_i = MenuItem::with_id(app, "pause", "Pause Monitoring", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &pause_i, &quit_i])?;

            // Load tray icon
            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                .unwrap_or_else(|_| Image::from_bytes(include_bytes!("../icons/32x32.png")).unwrap());

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("pictd - Clipboard Image Saver")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "pause" => {
                        let state = app.state::<AppState>();
                        if state.monitor.is_running() {
                            state.monitor.stop();
                        } else {
                            let save_dir = state.save_dir.lock().unwrap().clone();
                            let handle = app.clone();
                            state.monitor.start(save_dir, move |info| {
                                let _ = handle.emit("image-saved", &info);
                            });
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Auto-start monitoring
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>();
            let save_dir = state.save_dir.lock().unwrap().clone();
            state.monitor.start(save_dir, move |info| {
                let _ = app_handle.emit("image-saved", &info);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_save_directory,
            start_monitoring,
            stop_monitoring,
            get_saved_images,
            open_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
