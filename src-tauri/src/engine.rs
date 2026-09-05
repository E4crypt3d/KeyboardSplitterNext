//! Core engine: a single background thread that owns the authoritative state
//! (devices, players, mappings, virtual controllers) and is fed by the Raw
//! Input capture thread and by Tauri commands. Keeping it on one thread means
//! no lock contention and no Send/Sync dance around the ViGEm handles.
//!
//! All mutations are funnelled through [`EngineMsg`]; queries answer over a
//! oneshot channel. Profiles are persisted as JSON into the app config dir.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{AppHandle, Emitter};

use crate::controller::{GamepadReport, VirtualController};
use crate::core::{
    Binding, ControllerState, ControllerStatus, DriverStatus, KeyEventDto, KeyboardDevice,
    PlayerInfo, SavedConfig, SavedPlayer, Snapshot,
};
use crate::mapping::{default_bindings, report_for};

const MAX_PLAYERS: usize = 4;
const DEFAULT_PROFILE: &str = "Default";

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Request/response channel used for command replies (std mpsc keeps the
/// dependency list small and works without async).
pub type Reply<T> = Sender<Result<T, String>>;

pub enum EngineMsg {
    Snapshot(Reply<Snapshot>),
    SetRunning { running: bool, reply: Reply<Snapshot> },
    ProbeDriver(Reply<DriverStatus>),
    AssignKeyboard { player: usize, keyboard: Option<String>, reply: Reply<Snapshot> },
    SetBinding { player: usize, binding: Binding, reply: Reply<Snapshot> },
    RemoveBinding { player: usize, key: String, reply: Reply<Snapshot> },
    ClearMapping { player: usize, reply: Reply<Snapshot> },
    ResetDefault { player: usize, reply: Reply<Snapshot> },
    RenamePlayer { player: usize, name: String, reply: Reply<Snapshot> },
    AddPlayer(Reply<Snapshot>),
    RemovePlayer { player: usize, reply: Reply<Snapshot> },
    ReconnectControllers(Reply<Snapshot>),
    SaveProfile { name: String, reply: Reply<Snapshot> },
    LoadProfile { name: String, reply: Reply<Snapshot> },
    DeleteProfile { name: String, reply: Reply<()> },
    ListProfiles(Reply<Vec<String>>),
    /// A key was pressed/released on one physical keyboard.
    KeyEvent { device: String, key: String, down: bool },
    /// Device list changed (startup, plug/unplug).
    DeviceList { devices: Vec<KeyboardDevice> },
    Shutdown,
}

// ---------------------------------------------------------------------------
// Runtime player
// ---------------------------------------------------------------------------

struct PlayerRuntime {
    name: String,
    keyboard: Option<String>,
    bindings: Vec<Binding>,
    held: HashSet<String>,
    controller: Option<Box<dyn VirtualController>>,
    controller_error: Option<String>,
    last_report: Option<GamepadReport>,
}

impl PlayerRuntime {
    fn new(index: usize) -> Self {
        PlayerRuntime {
            name: format!("Player {}", index + 1),
            keyboard: None,
            bindings: default_bindings(),
            held: HashSet::new(),
            controller: None,
            controller_error: None,
            last_report: None,
        }
    }

    fn controller_state(&self) -> ControllerState {
        let status = match (&self.controller, &self.controller_error) {
            (Some(_), _) => ControllerStatus::Connected,
            (None, Some(_)) => ControllerStatus::Error,
            (None, None) => ControllerStatus::NotConnected,
        };
        ControllerState {
            status,
            message: self.controller_error.clone().or_else(|| {
                if status == ControllerStatus::NotConnected {
                    Some("No virtual controller - assign a keyboard first".to_string())
                } else {
                    None
                }
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Engine core
// ---------------------------------------------------------------------------

struct Core {
    running: bool,
    driver: DriverStatus,
    devices: Vec<KeyboardDevice>,
    /// device id -> (vendor id, product id) of the previous snapshot, used to
    /// re-associate a player with a re-plugged identical keyboard.
    previous_device_ids: HashMap<String, (u32, u32)>,
    players: Vec<PlayerRuntime>,
    profiles_dir: PathBuf,
    active_profile: String,
}

impl Core {
    fn new(data_dir: PathBuf) -> Core {
        let profiles_dir = data_dir.join("profiles");
        let _ = fs::create_dir_all(&profiles_dir);
        let driver = probe_driver_status();
        let mut core = Core {
            running: false,
            driver,
            devices: Vec::new(),
            previous_device_ids: HashMap::new(),
            players: Vec::new(),
            profiles_dir,
            active_profile: DEFAULT_PROFILE.to_string(),
        };
        core.load_players_from_file(&core.profile_path(DEFAULT_PROFILE));
        core
    }

    fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", sanitize_name(name)))
    }

    fn persist(&self) {
        let config = SavedConfig {
            players: self
                .players
                .iter()
                .map(|p| SavedPlayer {
                    name: p.name.clone(),
                    keyboard: p.keyboard.clone(),
                    bindings: p.bindings.clone(),
                })
                .collect(),
        };
        let path = self.profile_path(&self.active_profile);
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Could not save profile {}: {e}", path.display());
            }
        }
    }

    fn load_players_from_file(&mut self, path: &Path) {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                // First run: two ready-to-configure players.
                self.players = vec![PlayerRuntime::new(0), PlayerRuntime::new(1)];
                return;
            }
        };
        let config: SavedConfig = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Profile {} is invalid: {e}", path.display());
                self.players = vec![PlayerRuntime::new(0), PlayerRuntime::new(1)];
                return;
            }
        };
        if config.players.is_empty() {
            self.players = vec![PlayerRuntime::new(0), PlayerRuntime::new(1)];
            return;
        }
        self.players = config
            .players
            .iter()
            .enumerate()
            .map(|(i, saved)| PlayerRuntime {
                name: if saved.name.is_empty() {
                    format!("Player {}", i + 1)
                } else {
                    saved.name.clone()
                },
                keyboard: saved.keyboard.clone(),
                bindings: if saved.bindings.is_empty() {
                    default_bindings()
                } else {
                    saved.bindings.clone()
                },
                held: HashSet::new(),
                controller: None,
                controller_error: None,
                last_report: None,
            })
            .collect();
        while self.players.len() < 2 {
            self.players.push(PlayerRuntime::new(self.players.len()));
        }
        self.players.truncate(MAX_PLAYERS);
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            engine_running: self.running,
            driver: self.driver.clone(),
            devices: self.devices.clone(),
            players: self
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| PlayerInfo {
                    index: i,
                    name: p.name.clone(),
                    keyboard: p.keyboard.clone(),
                    keyboard_name: p.keyboard.as_ref().and_then(|id| {
                        self.devices
                            .iter()
                            .find(|d| &d.id == id)
                            .map(|d| d.name.clone())
                    }),
                    controller_state: p.controller_state(),
                    bindings: p.bindings.clone(),
                })
                .collect(),
            active_profile: self.active_profile.clone(),
        }
    }

    /// Ensure the player has a live virtual controller (creates + connects).
    fn ensure_controller(&mut self, player: usize, force: bool) {
        if player >= self.players.len() {
            return;
        }
        let p = &mut self.players[player];
        if p.keyboard.is_none() {
            return;
        }
        if force {
            if let Some(mut c) = p.controller.take() {
                c.disconnect();
            }
            p.controller_error = None;
        } else if p.controller.as_ref().map(|c| c.is_connected()).unwrap_or(false) {
            return;
        }
        match make_controller() {
            Ok(mut ctrl) => match ctrl.connect() {
                Ok(()) => {
                    p.controller = Some(ctrl);
                    p.controller_error = None;
                }
                Err(e) => {
                    p.controller = None;
                    p.controller_error = Some(e);
                }
            },
            Err(e) => {
                p.controller = None;
                p.controller_error = Some(e);
            }
        }
    }

    /// Try to re-associate players whose keyboard was unplugged with a newly
    /// plugged device of the same VID/PID (USB keyboards get a new instance
    /// id on every plug, so the raw path changes).
    fn reconcile_replugged_devices(&mut self) {
        let devices = self.devices.clone();
        let previous = self.previous_device_ids.clone();
        let mut by_vidpid: HashMap<(u32, u32), String> = HashMap::new();
        for d in &devices {
            if d.vendor_id != 0 {
                by_vidpid.entry((d.vendor_id, d.product_id)).or_insert_with(|| d.id.clone());
            }
        }
        let assigned_present: HashSet<String> = self
            .players
            .iter()
            .filter_map(|o| {
                let k = o.keyboard.as_ref()?;
                if devices.iter().any(|d| &d.id == k) { Some(k.clone()) } else { None }
            })
            .collect();

        for p in self.players.iter_mut() {
            let Some(assigned) = p.keyboard.clone() else { continue };
            if devices.iter().any(|d| d.id == assigned) {
                continue; // still present
            }
            // Assigned device gone - was its VID/PID seen before?
            let Some(vidpid) = previous.get(&assigned).copied() else { continue };
            if vidpid.0 == 0 {
                continue;
            }
            if let Some(replacement) = by_vidpid.get(&vidpid) {
                if !assigned_present.contains(replacement) {
                    log::info!("Re-associated player with re-plugged keyboard {replacement}");
                    p.keyboard = Some(replacement.clone());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Platform specific bits
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn make_controller() -> Result<Box<dyn VirtualController>, String> {
    Ok(Box::new(crate::controller::vigem::VigemController::new()))
}

#[cfg(not(target_os = "windows"))]
fn make_controller() -> Result<Box<dyn VirtualController>, String> {
    Err("Virtual Xbox controllers are only available on Windows.".to_string())
}

#[cfg(target_os = "windows")]
fn probe_driver_status() -> DriverStatus {
    match crate::controller::vigem::probe_driver() {
        Ok(()) => DriverStatus {
            available: true,
            message: "ViGEmBus driver detected".to_string(),
        },
        Err(e) => DriverStatus { available: false, message: e },
    }
}

#[cfg(not(target_os = "windows"))]
fn probe_driver_status() -> DriverStatus {
    DriverStatus {
        available: false,
        message: "Virtual controllers are only available on Windows.".to_string(),
    }
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "Profile".to_string()
    } else {
        trimmed.to_string()
    }
}

// ---------------------------------------------------------------------------
// Engine thread
// ---------------------------------------------------------------------------

pub fn spawn(app: AppHandle, data_dir: PathBuf) -> Sender<EngineMsg> {
    let (tx, rx) = mpsc::channel::<EngineMsg>();
    std::thread::Builder::new()
        .name("engine".into())
        .spawn(move || run_engine(app, data_dir, rx))
        .expect("failed to spawn engine thread");
    tx
}

fn run_engine(app: AppHandle, data_dir: PathBuf, rx: Receiver<EngineMsg>) {
    log::info!("Engine started (data dir: {})", data_dir.display());
    let mut core = Core::new(data_dir);

    // Get the initial device list from the capture thread (if running).
    // Windows-only: the capture thread will push DeviceList right away.

    for msg in rx {
        let mut changed = false;
        match msg {
            EngineMsg::Shutdown => break,
            EngineMsg::Snapshot(reply) => {
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::ProbeDriver(reply) => {
                core.driver = probe_driver_status();
                let _ = reply.send(Ok(core.driver.clone()));
            }
            EngineMsg::SetRunning { running, reply } => {
                core.running = running;
                if running {
                    // Make sure every assigned player has a controller.
                    for i in 0..core.players.len() {
                        core.ensure_controller(i, false);
                    }
                }
                changed = true;
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::AssignKeyboard { player, keyboard, reply } => {
                if let Some(id) = &keyboard {
                    if !core.devices.iter().any(|d| d.id == *id) {
                        let _ = reply.send(Err(
                            "That keyboard is not connected anymore".to_string()
                        ));
                        continue;
                    }
                }
                if player < core.players.len() {
                    // Releasing one player's keyboard clears its held keys so
                    // no controller button stays stuck.
                    if core.players[player].keyboard.as_deref() != keyboard.as_deref() {
                        core.players[player].held.clear();
                        core.players[player].last_report = None;
                    }
                    // A keyboard can only feed one player: detach it from any
                    // other player first.
                    if let Some(id) = &keyboard {
                        for (i, other) in core.players.iter_mut().enumerate() {
                            if i != player && other.keyboard.as_deref() == Some(id.as_str()) {
                                other.keyboard = None;
                                other.held.clear();
                                other.last_report = None;
                            }
                        }
                    }
                    core.players[player].keyboard = keyboard;
                    core.ensure_controller(player, false);
                }
                changed = true;
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::SetBinding { player, binding, reply } => {
                if let Some(p) = core.players.get_mut(player) {
                    // One key can have only one target - replace, don't stack.
                    p.bindings.retain(|b| b.key != binding.key);
                    p.bindings.push(binding);
                    p.held.clear();
                    p.last_report = None;
                    changed = true;
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::RemoveBinding { player, key, reply } => {
                if let Some(p) = core.players.get_mut(player) {
                    p.bindings.retain(|b| b.key != key);
                    p.held.remove(&key);
                    p.last_report = None;
                    changed = true;
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::ClearMapping { player, reply } => {
                if let Some(p) = core.players.get_mut(player) {
                    p.bindings.clear();
                    p.held.clear();
                    p.last_report = None;
                    changed = true;
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::ResetDefault { player, reply } => {
                if let Some(p) = core.players.get_mut(player) {
                    p.bindings = default_bindings();
                    p.held.clear();
                    p.last_report = None;
                    changed = true;
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::RenamePlayer { player, name, reply } => {
                if let Some(p) = core.players.get_mut(player) {
                    let name = name.trim().to_string();
                    if !name.is_empty() {
                        p.name = name;
                        changed = true;
                    }
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::AddPlayer(reply) => {
                if core.players.len() >= MAX_PLAYERS {
                    let _ = reply.send(Err(format!(
                        "XInput supports at most {MAX_PLAYERS} controllers"
                    )));
                } else {
                    core.players.push(PlayerRuntime::new(core.players.len()));
                    changed = true;
                    let _ = reply.send(Ok(core.snapshot()));
                }
            }
            EngineMsg::RemovePlayer { player, reply } => {
                if player < core.players.len() && core.players.len() > 2 {
                    let mut removed = core.players.remove(player);
                    if let Some(mut c) = removed.controller.take() {
                        c.disconnect();
                    }
                    // keep names stable: they are just labels
                    for (i, p) in core.players.iter_mut().enumerate() {
                        if p.name == removed.name {
                            p.name = format!("Player {}", i + 1);
                        }
                    }
                    removed.held.clear();
                    changed = true;
                }
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::ReconnectControllers(reply) => {
                for i in 0..core.players.len() {
                    core.ensure_controller(i, true);
                }
                // Re-push last known state so nothing stays stuck.
                for p in core.players.iter_mut() {
                    if let Some(ctrl) = p.controller.as_mut() {
                        let report = p.last_report.unwrap_or_default();
                        let _ = ctrl.submit(&report);
                    }
                }
                changed = true;
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::SaveProfile { name, reply } => {
                let clean = sanitize_name(&name);
                core.active_profile = clean;
                core.persist();
                changed = true;
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::LoadProfile { name, reply } => {
                let path = core.profile_path(&name);
                if !path.exists() {
                    let _ = reply.send(Err(format!("Profile '{name}' does not exist")));
                    continue;
                }
                for p in core.players.iter_mut() {
                    if let Some(mut c) = p.controller.take() {
                        c.disconnect();
                    }
                }
                core.load_players_from_file(&path);
                core.active_profile = sanitize_name(&name);
                for i in 0..core.players.len() {
                    core.ensure_controller(i, false);
                }
                changed = true;
                let _ = reply.send(Ok(core.snapshot()));
            }
            EngineMsg::DeleteProfile { name, reply } => {
                if sanitize_name(&name) == sanitize_name(&core.active_profile) {
                    let _ = reply.send(Err(
                        "Cannot delete the active profile".to_string()
                    ));
                    continue;
                }
                let path = core.profile_path(&name);
                match fs::remove_file(&path) {
                    Ok(()) => {
                        changed = true;
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!(
                            "Could not delete profile: {e}"
                        )));
                    }
                }
            }
            EngineMsg::ListProfiles(reply) => {
                let mut names: Vec<String> = Vec::new();
                if let Ok(entries) = fs::read_dir(&core.profiles_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                names.push(stem.to_string());
                            }
                        }
                    }
                }
                names.sort();
                let _ = reply.send(Ok(names));
            }
            EngineMsg::KeyEvent { device, key, down } => {
                handle_key_event(&app, &mut core, &device, &key, down);
            }
            EngineMsg::DeviceList { devices } => {
                core.previous_device_ids = core
                    .devices
                    .iter()
                    .map(|d| (d.id.clone(), (d.vendor_id, d.product_id)))
                    .collect();
                core.devices = devices;
                // Unplugged keyboards must not leave buttons stuck.
                let present: HashSet<&String> =
                    core.devices.iter().map(|d| &d.id).collect();
                for p in core.players.iter_mut() {
                    if let Some(k) = &p.keyboard {
                        if !present.contains(k) {
                            p.held.clear();
                            p.last_report = None;
                        }
                    }
                }
                core.reconcile_replugged_devices();
                changed = true;
            }
        }

        if changed {
            core.persist();
            let _ = app.emit("engine:changed", ());
        }
    }

    // Clean shutdown: unplug every virtual controller (drop runs disconnect).
    for p in core.players.iter_mut() {
        if let Some(mut c) = p.controller.take() {
            c.disconnect();
        }
    }
    log::info!("Engine stopped");
}

fn handle_key_event(app: &AppHandle, core: &mut Core, device: &str, key: &str, down: bool) {
    // Locate the player this keyboard belongs to.
    let player_index = core
        .players
        .iter()
        .position(|p| p.keyboard.as_deref() == Some(device));
    let Some(player_index) = player_index else {
        return; // unassigned keyboard - ignore
    };

    let device_name = core
        .devices
        .iter()
        .find(|d| d.id == device)
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Keyboard".to_string());

    // Always surface key events to the UI so the mapping editor can capture
    // keys even while the engine is paused.
    let _ = app.emit(
        "kb:event",
        KeyEventDto { device: device.to_string(), device_name, key: key.to_string(), down },
    );

    if !core.running {
        return;
    }

    // Only meaningful (bound) keys can change the controller state, but we
    // still track everything to keep release events correct.
    let p = &mut core.players[player_index];
    if down {
        p.held.insert(key.to_string());
    } else {
        p.held.remove(key);
    }

    let report = report_for(&p.bindings, &p.held);
    if p.last_report == Some(report) {
        return;
    }

    let submit = match p.controller.as_mut() {
        Some(ctrl) => ctrl.submit(&report),
        None => {
            // Controller missing (driver unavailable?) - nothing to push.
            return;
        }
    };

    match submit {
        Ok(()) => {
            p.controller_error = None;
            p.last_report = Some(report);
        }
        Err(e) => {
            p.controller_error = Some(e);
            p.last_report = None;
            // Emit so the UI can offer a reconnect.
            let _ = app.emit("engine:changed", ());
        }
    }
}

