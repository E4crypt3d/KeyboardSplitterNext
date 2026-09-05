//! ViGEmBus backend for the virtual Xbox 360 controller.
//!
//! ViGEm is the de-facto standard virtual gamepad framework on Windows
//! (used by KeyboardSplitterXbox and most input remappers). It needs the
//! `ViGEmBus` driver installed on the machine - no custom driver code here.
//!
//! All virtual controllers share ONE driver connection (`Client`): opening
//! one bus handle per player wastes handles, slows connects and adds a
//! kernel queue each (ViGEm best practice is one client, many targets).
#![cfg(target_os = "windows")]

use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

use super::{GamepadReport, VirtualController};

/// Lazily created, process-wide ViGEm bus connection shared by every player.
/// `probe_driver` and player controllers reuse the same handle, so the
/// driver only sees one connection no matter how many players exist.
static SHARED_CLIENT: OnceLock<Mutex<Option<Arc<Client>>>> = OnceLock::new();

fn shared_client() -> Result<Arc<Client>, String> {
    let cell = SHARED_CLIENT.get_or_init(|| Mutex::new(None));
    let mut guard = cell.lock().unwrap();
    if let Some(client) = guard.as_ref() {
        // The bus handle lives as long as the process; reuse it. If the
        // driver disappears (reinstall/update), connect() sees the dead
        // handle on plugin(), invalidate_shared_client() clears the cache
        // and the next reconnect starts from a fresh handle.
        return Ok(client.clone());
    }
    let client = Arc::new(Client::connect().map_err(|e| {
        format!(
            "Could not connect to the ViGEmBus driver ({e:?}).\n\n\
             Make sure the ViGEmBus driver is installed (device manager \
             should list a \"ViGEm Bus\" device)."
        )
    })?);
    *guard = Some(client.clone());
    Ok(client)
}

/// Forget the cached bus connection; the next `shared_client()` call opens a
/// fresh handle. Used when the driver is gone and before full reconnects so
/// nobody reuses a dead ViGEmBus handle.
pub fn invalidate_shared_client() {
    if let Some(cell) = SHARED_CLIENT.get() {
        if let Ok(mut guard) = cell.lock() {
            *guard = None;
        }
    }
}

/// Cheap availability probe: can we talk to the ViGEmBus driver at all?
/// (Does not create a virtual controller, so it makes no connect sound.)
pub fn probe_driver() -> Result<(), String> {
    shared_client().map(|_| ())
}

pub struct VigemController {
    target: Option<Xbox360Wired<Arc<Client>>>,
    connected: bool,
}

impl VigemController {
    pub fn new() -> Self {
        VigemController { target: None, connected: false }
    }
}

impl Default for VigemController {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualController for VigemController {
    fn connect(&mut self) -> Result<(), String> {
        self.disconnect();

        let client = shared_client()?;
        let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        if let Err(e) = target.plugin() {
            // The cached bus handle is unusable (driver missing or replaced
            // mid-session) - drop it so the next connect tries fresh.
            invalidate_shared_client();
            return Err(format!(
                "Could not create the virtual Xbox 360 controller ({e:?}).\n\n\
                 Make sure the ViGEmBus driver is installed."
            ));
        }

        // wait_ready can take a moment while Windows enumerates the device.
        // Kept short on purpose: this runs on the engine thread, so a long
        // wait would stall every keystroke while several players reconnect.
        let mut attempts = 0;
        while target.wait_ready().is_err() && attempts < 50 {
            std::thread::sleep(Duration::from_millis(10));
            attempts += 1;
        }

        self.target = Some(target);
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        // Dropping the target unplugs the virtual controller (clean unplug
        // so games / Windows do not keep a stale XInput device around). The
        // shared bus client stays alive for the other players.
        self.target = None;
        self.connected = false;
    }

    fn submit(&mut self, report: &GamepadReport) -> Result<(), String> {
        let target = self
            .target
            .as_mut()
            .ok_or_else(|| "Virtual controller is not connected".to_string())?;
        let gamepad = XGamepad {
            buttons: XButtons { raw: report.buttons },
            left_trigger: report.left_trigger,
            right_trigger: report.right_trigger,
            thumb_lx: report.left_x,
            thumb_ly: report.left_y,
            thumb_rx: report.right_x,
            thumb_ry: report.right_y,
        };
        target.update(&gamepad).map_err(|e| format!("Virtual controller update failed ({e:?})"))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
