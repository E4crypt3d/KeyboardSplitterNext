//! Tauri IPC commands - thin wrappers over the engine thread.
//! Every mutating command answers with the fresh [`Snapshot`].

use std::sync::mpsc;
use std::time::Duration;

use tauri::State;

use crate::core::{Binding, DriverStatus, Snapshot};
use crate::engine::{EngineMsg, Reply};
use crate::state::EngineHandle;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

/// Send a request to the engine thread and wait for its answer.
fn ask<T>(
    engine: &EngineHandle,
    make: impl FnOnce(Reply<T>) -> EngineMsg,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel::<Result<T, String>>();
    engine
        .tx
        .send(make(tx))
        .map_err(|_| "Engine is not running".to_string())?;
    match rx.recv_timeout(COMMAND_TIMEOUT) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(err),
        Err(_) => Err("Engine did not answer in time".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn snapshot(engine: State<'_, EngineHandle>) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::Snapshot)
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_running(
    engine: State<'_, EngineHandle>,
    running: bool,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetRunning { running, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn probe_driver(
    engine: State<'_, EngineHandle>,
) -> Result<DriverStatus, String> {
    ask(&engine, EngineMsg::ProbeDriver)
}

#[tauri::command(rename_all = "camelCase")]
pub fn assign_keyboard(
    engine: State<'_, EngineHandle>,
    player: usize,
    keyboard: Option<String>,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::AssignKeyboard { player, keyboard, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_binding(
    engine: State<'_, EngineHandle>,
    player: usize,
    binding: Binding,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SetBinding { player, binding, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_binding(
    engine: State<'_, EngineHandle>,
    player: usize,
    key: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RemoveBinding { player, key, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn clear_mapping(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::ClearMapping { player, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn reset_default(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::ResetDefault { player, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn rename_player(
    engine: State<'_, EngineHandle>,
    player: usize,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RenamePlayer { player, name, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn add_player(engine: State<'_, EngineHandle>) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::AddPlayer)
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_player(
    engine: State<'_, EngineHandle>,
    player: usize,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::RemovePlayer { player, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn reconnect_controllers(
    engine: State<'_, EngineHandle>,
) -> Result<Snapshot, String> {
    ask(&engine, EngineMsg::ReconnectControllers)
}

#[tauri::command(rename_all = "camelCase")]
pub fn save_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::SaveProfile { name, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn load_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<Snapshot, String> {
    ask(&engine, |reply| EngineMsg::LoadProfile { name, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_profile(
    engine: State<'_, EngineHandle>,
    name: String,
) -> Result<(), String> {
    ask(&engine, |reply| EngineMsg::DeleteProfile { name, reply })
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_profiles(
    engine: State<'_, EngineHandle>,
) -> Result<Vec<String>, String> {
    ask(&engine, EngineMsg::ListProfiles)
}
