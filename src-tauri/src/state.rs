//! Tauri-managed shared state.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;

use crate::engine::EngineMsg;

/// Handle to the engine thread. Commands clone the sender and ask the engine.
pub struct EngineHandle {
    pub tx: Sender<EngineMsg>,
}

/// Set when the user asked to quit from the tray; lets the window-close
/// handler distinguish "hide to tray" from a real exit.
pub struct QuitFlag(pub AtomicBool);
