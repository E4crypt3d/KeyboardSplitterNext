//! Shared, serializable data model used by the engine, the Tauri IPC layer
//! and the React frontend. Keep every type here small, serde-friendly and
//! platform independent (no Windows code allowed in this module).

pub mod keys;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Gamepad "targets" - what a physical key press is turned into.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    Back,
    Start,
    LThumb,
    RThumb,
    Guide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DpadDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StickSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StickDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TriggerSide {
    Left,
    Right,
}

/// A controller action a physical key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Target {
    Button { button: GamepadButton },
    Dpad { direction: DpadDirection },
    Trigger { side: TriggerSide },
    Stick {
        side: StickSide,
        direction: StickDirection,
    },
}

/// One "key -> controller action" entry of a player mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    /// Canonical key name, see `crate::core::keys` ("W", "Space", "Left", ...).
    pub key: String,
    pub target: Target,
}

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

/// A physical keyboard discovered through the Raw Input API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardDevice {
    /// Stable-ish per-instance identifier (Windows device path).
    pub id: String,
    /// Human readable name shown in the UI.
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    /// Windows device path (useful for debugging / future matching).
    pub path: String,
}

// ---------------------------------------------------------------------------
// Controller driver status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverStatus {
    pub available: bool,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Runtime players / snapshots sent to the UI
// ---------------------------------------------------------------------------

/// Runtime view of one player slot exposed to the frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub index: usize,
    pub name: String,
    /// Device id of the assigned physical keyboard, if any.
    pub keyboard: Option<String>,
    /// Human readable name of the assigned keyboard (resolved from devices).
    pub keyboard_name: Option<String>,
    pub controller_state: ControllerState,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerState {
    pub status: ControllerStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControllerStatus {
    Connected,
    Error,
    NotConnected,
}

/// Full state snapshot sent in reply to `snapshot` and after every mutation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub engine_running: bool,
    pub driver: DriverStatus,
    pub devices: Vec<KeyboardDevice>,
    pub players: Vec<PlayerInfo>,
    pub active_profile: String,
}

/// Event payload emitted when a key is pressed/released on an assigned keyboard.
/// The mapping editor listens to this to "capture" the next physical key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyEventDto {
    pub device: String,
    pub device_name: String,
    pub key: String,
    pub down: bool,
}

// ---------------------------------------------------------------------------
// Persisted configuration (profiles)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPlayer {
    pub name: String,
    pub keyboard: Option<String>,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedConfig {
    pub players: Vec<SavedPlayer>,
}
