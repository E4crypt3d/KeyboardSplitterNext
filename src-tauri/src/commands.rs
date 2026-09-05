//! Tauri IPC commands - thin wrappers over the engine thread.
//! Every mutating command answers with the fresh [`Snapshot`].

use std::sync::mpsc;
use std::time::Duration;

use tauri::State;

use crate::core::{Binding, DriverStatus, Snapshot};
use crate::engine::{EngineMsg, Reply};
use crate::presets::PresetMeta;
use crate::state::EngineHandle;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

/// Send a request to the engine thread and wait for its answer. Runs as a
/// Tauri async task, so the waiting never blocks the main thread (Tauri runs
/// sync commands there; blocking it freezes the whole app including input).
async fn ask<T: Send + 'static>(
    engine: &EngineHandle,
    make: impl FnOnce(Reply<T>) -> EngineMsg,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel::<Result<T, String>>();
    engine
        .tx
        .send(make(tx))
        .map_err(|_| "Engine is not running".to_string())?;
    match tokio::time::timeout(COMMAND_TIMEOUT, blocking_recv(rx)).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(err),
        Err(_) => Err("Engine did not answer in time".to_string()),
    }
}

/// Async wrapper around the engine's std mpsc receiver: parks the rx thread
/// (spawn_blocking pool) instead of blocking a tauri runtime worker.
async fn blocking_recv<T: Send + 'static>(
    rx: mpsc::Receiver<Result<T, String>>,
) -> Result<T, String> {
    tokio::task::spawn_blocking(move || {
        rx.recv().unwrap_or_else(|_| Err("Engine is not running".to_string()))
    })
    .await
    .map_err(|e| format!("Engine reply task failed: {e}"))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn snapshot(engine: State<'_, EngineHandle>) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::Snapshot).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_running(
    engine: State<'_, EngineHandle>,
    running: bool,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetRunning { running, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn probe_driver(
    engine: State<'_, EngineHandle>,
) -> Result<DriverStatus, String> {
    ask(&engine, EngineMsg::ProbeDriver).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn assign_keyboard(
    engine: State<'_, EngineHandle>,
    player: usize,
    keyboard: Option<String>,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::AssignKeyboard { player, keyboard, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_tap_assign(
    engine: State<'_, EngineHandle>,
    player: Option<usize>,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetTapAssign { player, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_test_mode(
    engine: State<'_, EngineHandle>,
    enabled: bool,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetTestMode { enabled, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_binding(
    engine: State<'_, EngineHandle>,
    player: usize,
    binding: Binding,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetBinding { player, binding, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn remove_binding(
    engine: State<'_, EngineHandle>,
    player: usize,
    key: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RemoveBinding { player, key, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn clear_mapping(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::ClearMapping { player, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn reset_default(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::ResetDefault { player, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn rename_player(
    engine: State<'_, EngineHandle>,
    player: usize,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RenamePlayer { player, name, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_player(engine: State<'_, EngineHandle>) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::AddPlayer).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn remove_player(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RemovePlayer { player, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn reconnect_controllers(
    engine: State<'_, EngineHandle>,
) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::ReconnectControllers).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_presets(
    engine: State<'_, EngineHandle>,
) -> Result<Vec<PresetMeta>, String> {
    ask(&engine, EngineMsg::ListPresets).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_preset_keys(
    engine: State<'_, EngineHandle>,
    preset_id: String,
) -> Result<Vec<String>, String> {
    ask(&engine, |reply| EngineMsg::ListPresetKeys { preset_id, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn apply_preset(
    engine: State<'_, EngineHandle>,
    player: usize,
    preset_id: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::ApplyPreset { player, preset_id, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SaveProfile { name, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn load_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::LoadProfile { name, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<(), String> {
    ask(&engine, |reply| EngineMsg::DeleteProfile { name, reply }).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_profiles(
    engine: State<'_, EngineHandle>,
) -> Result<Vec<String>, String> {
    ask(&engine, EngineMsg::ListProfiles).await
}
