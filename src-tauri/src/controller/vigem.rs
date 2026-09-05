//! ViGEmBus backend for the virtual Xbox 360 controller.
//!
//! ViGEm is the de-facto standard virtual gamepad framework on Windows
//! (used by KeyboardSplitterXbox and most input remappers). It needs the
//! `ViGEmBus` driver installed on the machine - no custom driver code here.
#![cfg(target_os = "windows")]

use std::sync::Arc;
use std::time::Duration;

use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

use super::{GamepadReport, VirtualController};

/// Cheap availability probe: can we talk to the ViGEmBus driver at all?
/// (Does not create a virtual controller, so it makes no connect sound.)
pub fn probe_driver() -> Result<(), String> {
    Client::connect().map(|_| ()).map_err(|e| {
        format!(
            "Could not connect to the ViGEmBus driver ({e:?}).\n\
             Install ViGEmBus from https://github.com/nefarius/ViGEmBus/releases \
             and restart this app."
        )
    })
}

pub struct VigemController {
    client: Option<Arc<Client>>,
    target: Option<Xbox360Wired<Arc<Client>>>,
    connected: bool,
}

impl VigemController {
    pub fn new() -> Self {
        VigemController { client: None, target: None, connected: false }
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

        let client = Arc::new(Client::connect().map_err(|e| {
            format!(
                "Could not connect to the ViGEmBus driver ({e:?}).\n\n\
                 Make sure the ViGEmBus driver is installed (device manager \
                 should list a \"ViGEm Bus\" device)."
            )
        })?);
        let mut target = Xbox360Wired::new(client.clone(), TargetId::XBOX360_WIRED);
        target.plugin().map_err(|e| {
            format!(
                "Could not create the virtual Xbox 360 controller ({e:?}).\n\n\
                 Make sure the ViGEmBus driver is installed."
            )
        })?;

        // wait_ready can take a moment while Windows enumerates the device.
        let mut attempts = 0;
        while target.wait_ready().is_err() && attempts < 100 {
            std::thread::sleep(Duration::from_millis(20));
            attempts += 1;
        }

        self.client = Some(client);
        self.target = Some(target);
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        // Dropping the target unplugs the virtual controller (clean unplug
        // so games / Windows do not keep a stale XInput device around).
        self.target = None;
        self.client = None;
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
