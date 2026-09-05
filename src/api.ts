import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Binding, DriverStatus, KeyEventDto, PresetMeta, Snapshot } from './types'

// --- commands ---------------------------------------------------------------

export const snapshot = () => invoke<Snapshot>('snapshot')
export const setRunning = (running: boolean) => invoke<Snapshot>('set_running', { running })
export const probeDriver = () => invoke<DriverStatus>('probe_driver')

export const assignKeyboard = (player: number, keyboard: string | null) =>
  invoke<Snapshot>('assign_keyboard', { player, keyboard })

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
/** Replace one player's bindings with a built-in preset. */
export const applyPreset = (player: number, presetId: string) =>
  invoke<Snapshot>('apply_preset', { player, presetId })

export const renamePlayer = (player: number, name: string) =>
  invoke<Snapshot>('rename_player', { player, name })

export const addPlayer = () => invoke<Snapshot>('add_player')
export const removePlayer = (player: number) =>
  invoke<Snapshot>('remove_player', { player })

export const reconnectControllers = () => invoke<Snapshot>('reconnect_controllers')

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
