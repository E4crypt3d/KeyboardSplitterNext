//! Tauri application wiring: engine thread + raw input capture startup,
//! system tray, hide-on-close behaviour and IPC commands.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod controller;
mod core;
mod engine;
mod input;
mod mapping;
mod state;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, RunEvent, WindowEvent};
use tauri::image::Image;

use crate::engine::EngineMsg;
use crate::state::{EngineHandle, QuitFlag};

/// Keeps the tray icon alive for the lifetime of the app.
struct TrayHolder(Mutex<Option<TrayIcon>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let data_dir = resolve_data_dir(app);
            let _ = fs::create_dir_all(&data_dir);
            log::info!("data dir: {}", data_dir.display());

            let tx = engine::spawn(app.handle().clone(), data_dir);

            app.manage(EngineHandle { tx: tx.clone() });
            app.manage(QuitFlag(std::sync::atomic::AtomicBool::new(false)));
            app.manage(TrayHolder(Mutex::new(None)));
            setup_tray(app)?;

            // Raw Input capture: forwards physical-key events (with source
            // device) to the engine. Windows only.
            #[cfg(target_os = "windows")]
            if let Err(e) = input::capture::spawn(tx) {
                log::error!("could not start raw input capture: {e}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window hides the app to the tray instead of exiting
            // (it keeps splitting while minimized). Quit happens from the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let flag = window.state::<QuitFlag>();
                if !flag.0.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::set_running,
            commands::probe_driver,
            commands::assign_keyboard,
            commands::set_binding,
            commands::remove_binding,
            commands::clear_mapping,
            commands::reset_default,
            commands::rename_player,
            commands::add_player,
            commands::remove_player,
            commands::reconnect_controllers,
            commands::save_profile,
            commands::load_profile,
            commands::delete_profile,
            commands::list_profiles,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            shutdown_engine(app_handle);
        }
    });
}

fn shutdown_engine(app: &AppHandle) {
    // Give the engine a moment to unplug all virtual controllers cleanly
    // (a hard kill would leave stuck XInput devices behind).
    if let Some(handle) = app.try_state::<EngineHandle>() {
        let tx: &Sender<EngineMsg> = &handle.tx;
        let _ = tx.send(EngineMsg::Shutdown);
    }
    std::thread::sleep(std::time::Duration::from_millis(350));
}

fn resolve_data_dir(app: &App) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("keyboard-splitter"))
}

fn setup_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Keyboard Splitter", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon: Image = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| Image::new_owned(vec![], 0, 0));

    let tray = TrayIconBuilder::with_id("keyboard-splitter-tray")
        .icon(icon)
        .tooltip("Keyboard Splitter")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                show_main_window(app);
            }
            "quit" => {
                let flag = app.state::<QuitFlag>();
                flag.0.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                show_main_window(app);
            }
        })
        .build(app)?;

    let holder = app.state::<TrayHolder>();
    *holder.0.lock().unwrap() = Some(tray);
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
