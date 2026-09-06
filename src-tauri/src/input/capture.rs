//! Raw Input capture thread (Windows only).
//!
//! Creates a hidden message-only window on its own thread, registers it for
//! Raw Input keyboard events with `RIDEV_INPUTSINK` (so keys are observed
//! even while the app is unfocused/minimized) and forwards every key event
//! together with its *source device* to the engine.
//!
//! Raw Input only *observes* keys - it does not swallow them. Like other
//! raw-input based tools this is fine for controller-only games: the bound
//! keys also type into whatever window has focus, which is normally the game
//! itself (XInput based, ignoring keyboard). Tools that need to fully block
//! the source keyboard (e.g. the Interception-driver based
//! KeyboardSplitterXbox) can be plugged in later behind `input::capture`
//! without touching the rest of the engine.

use std::collections::HashMap;
use std::mem::size_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::Sender;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
    RAWINPUTDEVICE_FLAGS, RAWINPUTHEADER, RAWKEYBOARD, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK,
    RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, DestroyWindow, HWND_MESSAGE, MSG, RI_KEY_BREAK,
    WM_DESTROY, WM_INPUT, WM_INPUT_DEVICE_CHANGE, WINDOW_STYLE, WNDCLASSW, WNDCLASS_STYLES,
};

use crate::core::KeyboardDevice;
use crate::core::keys::name_for_vk;
use crate::engine::EngineMsg;
use crate::input::devices;

const WINDOW_CLASS: &str = "KeyboardSplitterRawInputWindow";

struct CaptureCtx {
    /// handle (as usize) -> keyboard device id
    cache: Mutex<HashMap<usize, String>>,
    tx: Sender<EngineMsg>,
}

static CTX: OnceLock<Arc<CaptureCtx>> = OnceLock::new();

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Refresh the device list and our handle->device cache, then notify the
/// engine. Called at startup and on every WM_INPUT_DEVICE_CHANGE.
fn refresh_devices(ctx: &CaptureCtx) {
    let entries = devices::enumerate_with_handles();
    let mut cache = ctx.cache.lock().unwrap();
    cache.clear();
    let mut list: Vec<KeyboardDevice> = Vec::with_capacity(entries.len());
    for (handle, device) in entries {
        cache.insert(handle, device.id.clone());
        list.push(device);
    }
    drop(cache);
    let _ = ctx.tx.send(EngineMsg::DeviceList { devices: list });
}

fn handle_raw_input(ctx: &CaptureCtx, lparam: LPARAM) {
    let hraw = HRAWINPUT(lparam.0 as *mut core::ffi::c_void);
    let header_size = size_of::<windows::Win32::UI::Input::RAWINPUTHEADER>() as u32;

    // One WM_INPUT carries exactly one RAWINPUT packet (Microsoft docs).
    // The buffer is thread-local and reused so a keystroke never allocates
    // (capacity is reserved on the first packet, then kept).
    thread_local! {
        static BUF: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    BUF.with_borrow_mut(|buf| {
        // resize, not reserve: GetRawInputData writes the packet into the
        // buffer, and the bound check below compares against len(). reserve()
        // only grows capacity while len() stays 0, which silently dropped
        // every packet (regression in 0.3.1). resize() makes the length match
        // the capacity so the check is real.
        if buf.capacity() < RAW_BUF_SIZE {
            buf.resize(RAW_BUF_SIZE, 0);
        }
        let mut size: u32 = RAW_BUF_SIZE as u32;
        let written = unsafe {
            GetRawInputData(
                hraw,
                RID_INPUT,
                Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
                &mut size,
                header_size,
            )
        };
        if written == u32::MAX || written == 0 {
            // -1 = error; 0 should not happen for a valid WM_INPUT, ignore
            // both (e.g. a mouse packet we never asked for).
            return;
        }
        // Defensive bound check: only reinterpret the buffer as RAWINPUT when
        // it can actually hold the header plus the keyboard struct. A
        // truncated or malformed packet must never turn into an out-of-bounds
        // read (the reported `written` size is not fully trusted).
        let needed = size_of::<RAWINPUTHEADER>() + size_of::<RAWKEYBOARD>();
        if (written as usize) < needed || (written as usize) > buf.len() {
            return;
        }
        let input = unsafe { &*(buf.as_ptr() as *const RAWINPUT) };
        if input.header.dwType != RIM_TYPEKEYBOARD.0 {
            return;
        }
        let keyboard = unsafe { &input.data.keyboard };
        let vk = keyboard.VKey;
        // 0xFF means "no virtual key" (e.g. pure modifier up events).
        if vk == 0xFF {
            return;
        }
        let is_break = keyboard.Flags & RI_KEY_BREAK as u16 != 0;
        let key = name_for_vk(vk);

        let handle = input.header.hDevice.0 as usize;
        let device_id = {
            let cache = ctx.cache.lock().unwrap();
            cache.get(&handle).cloned()
        };
        let Some(device_id) = device_id else {
            // Device not in cache yet (arrival event raced) - refresh once.
            refresh_devices(ctx);
            return;
        };

        let _ = ctx.tx.send(EngineMsg::KeyEvent {
            device: device_id,
            key,
            down: !is_break,
        });
    });
}

/// A RAWINPUTKEYBOARD packet is a few dozen bytes; 1 KiB is a generous,
/// stack-free upper bound for any keyboard raw input blob.
const RAW_BUF_SIZE: usize = 1024;

/// Direct trampoline installed as the window procedure: the real handler is
/// kept in a static so every callback runs behind `catch_unwind`. A panic
/// unwinding out of an `extern "system"` callback aborts the whole process
/// (Windows shows a generic memory/heap error), so the trampoline catches
/// panics and falls back to `DefWindowProcW` - WM_INPUT keeps flowing and a
/// single bad packet can never take the splitter down.
static WND_PROC: OnceLock<
    Box<dyn Fn(HWND, u32, WPARAM, LPARAM) -> LRESULT + Send + Sync>,
> = OnceLock::new();

extern "system" fn wnd_proc_trampoline(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match WND_PROC.get() {
        Some(proc) => proc(hwnd, msg, wparam, lparam),
        None => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            if let Some(ctx) = CTX.get() {
                handle_raw_input(ctx, lparam);
            }
            LRESULT(0)
        }
        WM_INPUT_DEVICE_CHANGE => {
            if let Some(ctx) = CTX.get() {
                // wParam: GIDC_ARRIVAL (1) / GIDC_REMOVAL (2).
                log::debug!("raw input device change event (wParam={})", wparam.0);
                refresh_devices(ctx);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Spawn the capture thread. Only one capture thread should ever exist.
pub fn spawn(tx: Sender<EngineMsg>) -> Result<(), String> {
    std::thread::Builder::new()
        .name("raw-input-capture".into())
        .spawn(|| unsafe {
            if let Err(e) = message_thread(tx) {
                log::error!("Raw input thread failed: {e}");
            }
        })
        .map(|_| ())
        .map_err(|e| format!("could not spawn capture thread: {e}"))
}

unsafe fn message_thread(tx: Sender<EngineMsg>) -> Result<(), String> {
    let hinstance: HINSTANCE = GetModuleHandleW(None)
        .map(|m| HINSTANCE(m.0))
        .map_err(|e| format!("GetModuleHandleW failed: {e:?}"))?;

    let class_name = wide(WINDOW_CLASS);
    let _ = WND_PROC.set(Box::new(|hwnd, msg, wparam, lparam| {
        match catch_unwind(AssertUnwindSafe(|| unsafe {
            window_proc(hwnd, msg, wparam, lparam)
        })) {
            Ok(result) => result,
            Err(_) => {
                log::error!("panic in raw input window procedure (msg=0x{msg:04X}) - recovered");
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
    }));
    let wc = WNDCLASSW {
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(wnd_proc_trampoline),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
        hCursor: windows::Win32::UI::WindowsAndMessaging::HCURSOR::default(),
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
    };
    let class_atom = RegisterClassW(&wc);
    if class_atom == 0 {
        let err = GetLastError();
        // ERROR_CLASS_ALREADY_EXISTS (1410) is fine on re-registration.
        if err.0 != 1410 {
            return Err(format!("RegisterClassW failed: {}", err.0));
        }
    }

    let hwnd = CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
        PCWSTR(class_name.as_ptr()),
        PCWSTR(class_name.as_ptr()),
        WINDOW_STYLE(0),
        0,
        0,
        0,
        0,
        Some(HWND_MESSAGE),
        None,
        Some(hinstance),
        None,
    )
    .map_err(|e| format!("CreateWindowExW failed: {e:?}"))?;

    // Register for keyboard raw input, delivered to our window even when the
    // app has no focus (INPUTSINK) and device arrival/removal notifications.
    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01, // generic desktop
        usUsage: 0x06,     // keyboard
        dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0 | RIDEV_DEVNOTIFY.0),
        hwndTarget: hwnd,
    };
    RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32)
        .map_err(|e| format!("RegisterRawInputDevices failed: {e:?}"))?;

    let ctx = Arc::new(CaptureCtx {
        cache: Mutex::new(HashMap::new()),
        tx,
    });
    let _ = CTX.set(ctx.clone());

    log::info!("Raw input capture started");
    refresh_devices(&ctx);

    // Standard Win32 message pump - blocks until WM_QUIT.
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        let _ = DispatchMessageW(&msg);
    }

    let _ = DestroyWindow(hwnd);
    log::info!("Raw input capture stopped");
    Ok(())
}
