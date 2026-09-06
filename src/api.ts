import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'
import type {
  Binding,
  DriverStatus,
  KeyEventDto,
  PresetMeta,
  Snapshot,
  TestEventDto,
} from './types'

// --- commands ---------------------------------------------------------------

export const snapshot = () => invoke<Snapshot>('snapshot')
export const setRunning = (running: boolean) => invoke<Snapshot>('set_running', { running })
export const probeDriver = () => invoke<DriverStatus>('probe_driver')

export const assignKeyboard = (player: number, keyboard: string | null) =>
  invoke<Snapshot>('assign_keyboard', { player, keyboard })

/** Start/stop "press a key to assign": wait for the next key on an
 *  unassigned keyboard and bind it to this player slot (Escape cancels). */
export const setTapAssign = (player: number | null) =>
  invoke<Snapshot>('set_tap_assign', { player })

/** Live-input feedback while the engine runs (off by default). */
export const setTestMode = (enabled: boolean) =>
  invoke<Snapshot>('set_test_mode', { enabled })

export const setBinding = (player: number, binding: Binding) =>
  invoke<Snapshot>('set_binding', { player, binding })

export const removeBinding = (player: number, key: string) =>
  invoke<Snapshot>('remove_binding', { player, key })

export const clearMapping = (player: number) =>
  invoke<Snapshot>('clear_mapping', { player })

export const resetDefault = (player: number) =>
  invoke<Snapshot>('reset_default', { player })

/** Built-in game presets (FIFA, MK, Tekken, ...). */
export const listPresets = () => invoke<PresetMeta[]>('list_presets')
/** Canonical key names bound by one built-in preset (display order). */
export const listPresetKeys = (presetId: string) =>
  invoke<string[]>('list_preset_keys', { presetId })
/** Replace one player's bindings with a built-in preset. */
export const applyPreset = (player: number, presetId: string) =>
  invoke<Snapshot>('apply_preset', { player, presetId })

export const renamePlayer = (player: number, name: string) =>
  invoke<Snapshot>('rename_player', { player, name })

export const addPlayer = () => invoke<Snapshot>('add_player')
export const removePlayer = (player: number) =>
  invoke<Snapshot>('remove_player', { player })

export const reconnectControllers = () => invoke<Snapshot>('reconnect_controllers')

/** Open an external URL in the system browser (webview navigation is
 *  blocked by default, so links go through the opener plugin). */
export const openExternal = (url: string) => openUrl(url)

export const saveProfile = (name: string) => invoke<Snapshot>('save_profile', { name })
export const loadProfile = (name: string) => invoke<Snapshot>('load_profile', { name })
export const deleteProfile = (name: string) => invoke<null>('delete_profile', { name })
export const listProfiles = () => invoke<string[]>('list_profiles')

// --- events -----------------------------------------------------------------

/** Fires whenever the backend state changed (refresh the snapshot). */
export function onEngineChanged(cb: () => void): Promise<UnlistenFn> {
  return listen('engine:changed', cb)
}

/** Stream of raw key events (used by the mapping editor's key capture). */
export function onKeyEvent(cb: (e: KeyEventDto) => void): Promise<UnlistenFn> {
  return listen<KeyEventDto>('kb:event', (ev) => cb(ev.payload))
}

/** Stream of bound key presses while test mode is enabled and engine runs. */
export function onTestEvent(cb: (e: TestEventDto) => void): Promise<UnlistenFn> {
  return listen<TestEventDto>('test:event', (ev) => cb(ev.payload))
}
