//! Keyboard input: detection (devices) and capture (raw input thread).

pub mod devices;

#[cfg(target_os = "windows")]
pub mod capture;
