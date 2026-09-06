//! Tauri application wiring: engine thread + raw input capture startup,
//! system tray, hide-on-close behaviour and IPC commands.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod controller;
mod core;
mod engine;
mod input;
mod mapping;
mod presets;
mod state;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HANDLE, HMODULE};
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_POINTERS, MiniDumpWithFullMemory, MiniDumpWriteDump,
    MINIDUMP_EXCEPTION_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId,
};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, RunEvent, WindowEvent};
use tauri::image::Image;

use crate::engine::EngineMsg;
use crate::state::{EngineHandle, QuitFlag};

/// Keeps the tray icon alive for the lifetime of the app.
struct TrayHolder(Mutex<Option<TrayIcon>>);

/// Where crash logs go: mirrors Tauri's `app_config_dir`
/// (`%APPDATA%\<identifier>` on Windows), falling back to the temp dir when
/// the env var is missing (non-Windows CI, stripped-down systems).
fn crash_log_dir() -> PathBuf {
    const IDENTIFIER: &str = "com.e4crypt3d.keyboardsplitter";
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(IDENTIFIER)
}

/// Append one line to the latest crash log file. Shared by the panic hook
/// and the Windows exception handler; must never panic itself (crash paths
/// have to stay crash-proof).
fn write_crash_log(line: &str) {
    let dir = crash_log_dir();
    // The data dir may not exist yet if the crash fired before setup.
    let _ = fs::create_dir_all(&dir);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("crash-{stamp}-{}.log", std::process::id()));
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
}

/// Windows vectored exception handler: turns the bare "instruction at ...
/// referenced memory at ..." dialog into a written record. A hard crash
/// (access violation, stack overflow, ...) is not a Rust panic, so the panic
/// hook alone leaves release builds silent - this writes a one-line report
/// naming the faulting module (app.exe vs msedgewebview2.exe vs ntdll.dll)
/// plus a full minidump next to it, so the next "inconsistent crash" is
/// diagnosable from the user's machine.
///
/// Best-effort only: runs inside the exception dispatch, so every call is
/// let-errored and the whole body is wrapped in catch_unwind - a fault in
/// the handler itself must not take the process down a second time.
#[cfg(target_os = "windows")]
fn install_exception_handler() {
    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            write_exception_report(info);
        }));
        0 // EXCEPTION_CONTINUE_SEARCH: keep the default WER dialog too
    }

    unsafe {
        let _ = AddVectoredExceptionHandler(1, Some(handler));
    }
}

#[cfg(target_os = "windows")]
unsafe fn write_exception_report(info: *mut EXCEPTION_POINTERS) {
    if info.is_null() {
        return;
    }
    let record = &*(*info).ExceptionRecord;
    let code = record.ExceptionCode.0 as u32;
    let address = record.ExceptionAddress as usize;

    let kind = match code {
        // AV params: ExceptionInformation[0] = 0 read / 1 write / 8 execute,
        // ExceptionInformation[1] = the faulting address.
        0xC0000005 => {
            let access = record.ExceptionInformation.first().copied().unwrap_or(0);
            let target = record.ExceptionInformation.get(1).copied().unwrap_or(0);
            match access {
                1 => format!("access violation: write to {target:#018x}"),
                8 => format!("access violation: execute of {target:#018x}"),
                _ => format!("access violation: read of {target:#018x}"),
            }
        }
        0xC00000FD => "stack overflow".to_string(),
        0xC000001D => "illegal instruction".to_string(),
        other => format!("unhandled exception {other:#010x}"),
    };

    // Name the DLL the faulting instruction lives in; the module handle
    // doubles as its base address, so the offset is cheap to compute.
    let module_line = {
        let mut module = HMODULE::default();
        let flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
            | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        if GetModuleHandleExW(flags, PCWSTR(address as *const u16), &mut module).is_ok() {
            let mut name = [0u16; 512];
            let len = GetModuleFileNameW(Some(module), &mut name);
            let name = String::from_utf16_lossy(&name[..len as usize]);
            format!("{name} (offset +{:#x})", address - module.0 as usize)
        } else {
            "<unknown>".to_string()
        }
    };
    write_crash_log(&format!(
        "{kind} at {address:#018x} (thread {}) in {module_line}",
        GetCurrentThreadId()
    ));

    // Full minidump next to the log for offline debugging.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = crash_log_dir().join(format!("crash-{stamp}-{}.dmp", std::process::id()));
    if let Ok(file) = std::fs::File::create(&path) {
        let exception = MINIDUMP_EXCEPTION_INFORMATION {
            ThreadId: GetCurrentThreadId(),
            ExceptionPointers: info,
            ClientPointers: false.into(),
        };
        let _ = MiniDumpWriteDump(
            GetCurrentProcess(),
            GetCurrentProcessId(),
            HANDLE(file.as_raw_handle()),
            MiniDumpWithFullMemory,
            Some(&exception),
            None,
            None,
        );
    }
}

/// Panic payload as a human readable string (Rust allows `&str` and `String`
/// payloads; anything else is reported opaquely).
fn panic_payload(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

fn handle_panic(info: &std::panic::PanicHookInfo<'_>) {
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "<unknown>".to_string());
    let message = panic_payload(info);
    log::error!("PANIC at {location}: {message}");
    write_crash_log(&format!("panic at {location}: {message}"));
}

/// Last-resort crash logging: release builds have no console (the log plugin
/// only runs in debug), so a panic on any thread used to exit silently - the
/// exact "app closes as soon as it opens" report that is impossible to
/// diagnose remotely. Every panic now lands in
/// `%APPDATA%\<identifier>\crash-<time>-<pid>.log` (best effort: failures to
/// write must never panic from inside the hook itself) and still goes to the
/// previous hook for debug builds.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        handle_panic(info);
        default_hook(info);
    }));
}

/// Windows named-mutex single-instance guard (no extra dependency needed:
/// the `windows` crate is already a dependency). Two engines fighting over
/// the same keyboards, profile file and ViGEmBus controllers is a real
/// conflict, so a second launch exits immediately - the first instance keeps
/// running in the tray. A kernel mutex is released automatically when the
/// owning process dies, so a stale lock can never block a restart.
#[cfg(target_os = "windows")]
fn acquire_single_instance() -> bool {
    use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
    use windows::Win32::System::Threading::CreateMutexW;

    let name: Vec<u16> = "Local\\KeyboardSplitterNext.SingleInstance"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: standard Win32 call with a valid UTF-16 name; the handle is
    // either closed below (duplicate) or leaked for the process lifetime.
    match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        Ok(handle) => {
            let already_exists = unsafe { GetLastError().0 == ERROR_ALREADY_EXISTS.0 };
            if already_exists {
                // Duplicate instance - release our handle and let it exit.
                unsafe { let _ = CloseHandle(handle); }
                false
            } else {
                // First instance: the handle is a raw pointer with no Drop
                // impl, so simply not closing it keeps the mutex alive for
                // the whole process lifetime (the OS cleans up on exit).
                true
            }
        }
        Err(e) => {
            // Creating the mutex failed for an unrelated reason (rare). Do
            // not block startup on it - log and continue.
            log::warn!("single-instance guard unavailable: {e:?}");
            true
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Install before anything else so even early panics are diagnosable.
    install_panic_hook();
    // Hard crashes (access violations etc.) are not Rust panics; this logs
    // them with the faulting module and writes a minidump. Windows only.
    #[cfg(target_os = "windows")]
    install_exception_handler();

    // Only one engine may ever run per session (Windows is the only OS where
    // the app works at all, but the guard is cheap and explicit).
    #[cfg(target_os = "windows")]
    if !acquire_single_instance() {
        std::process::exit(0);
    }

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
            commands::set_tap_assign,
            commands::set_test_mode,
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
            commands::apply_preset,
            commands::list_presets,
            commands::list_preset_keys,
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
        .unwrap_or_else(|| {
            // Fallback only when the bundle carries no icon: a 0x0 empty
            // image makes tray-icon build a null HICON; a single opaque
            // pixel is a valid bitmap that can never fault.
            Image::new_owned(vec![0x00, 0xFF, 0x00, 0xFF], 1, 1)
        });

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
