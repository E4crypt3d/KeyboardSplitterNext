// TypeScript mirror of the Rust `core` module DTOs (serde rename_all =
// camelCase for structs; the small enums serialize as UPPERCASE tokens).

export type GamepadButton =
  | 'A' | 'B' | 'X' | 'Y' | 'LB' | 'RB'
  | 'BACK' | 'START' | 'LTHUMB' | 'RTHUMB' | 'GUIDE'

export type DpadDirection = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT'
export type StickSide = 'LEFT' | 'RIGHT'
export type StickDirection = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT'
export type TriggerSide = 'LEFT' | 'RIGHT'

export type Target =
  | { type: 'button'; button: GamepadButton }
  | { type: 'dpad'; direction: DpadDirection }
  | { type: 'trigger'; side: TriggerSide }
  | { type: 'stick'; side: StickSide; direction: StickDirection }

export interface Binding {
  key: string
  target: Target
}

export interface KeyboardDevice {
  id: string
  name: string
  vendorId: number
  productId: number
  path: string
}

export interface DriverStatus {
  available: boolean
  message: string
}

export type ControllerStatus = 'connected' | 'error' | 'notConnected'

export interface ControllerState {
  status: ControllerStatus
  message?: string | null
}

export interface PlayerInfo {
  index: number
  name: string
  keyboard?: string | null
  keyboardName?: string | null
  controllerState: ControllerState
  bindings: Binding[]
}

export interface Snapshot {
  engineRunning: boolean
  driver: DriverStatus
  devices: KeyboardDevice[]
  players: PlayerInfo[]
  activeProfile: string
  /** Player slot waiting for a "press any key" keyboard assignment, if any. */
  tapAssign: number | null
  /** Live-input feedback toggled by the user ("test mode"). */
  testMode: boolean
}

export interface PresetMeta {
  id: string
  name: string
  description: string
  keyCount: number
}

export interface KeyEventDto {
  device: string
  deviceName: string
  key: string
  down: boolean
}

/** Live "test mode" payload: one bound key press/release on a player's keyboard. */
export interface TestEventDto {
  player: number
  key: string
  down: boolean
}

// ---------------------------------------------------------------------------
// Human labels for the target picker
// ---------------------------------------------------------------------------

const btnLabel: Record<GamepadButton, string> = {
  A: 'A', B: 'B', X: 'X', Y: 'Y',
  LB: 'LB', RB: 'RB', BACK: 'Back', START: 'Start',
  LTHUMB: 'LS Click', RTHUMB: 'RS Click', GUIDE: 'Guide',
}

const title = (s: string) => s.charAt(0) + s.slice(1).toLowerCase()

export function targetLabel(t: Target): string {
  switch (t.type) {
    case 'button':
      return btnLabel[t.button]
    case 'dpad':
      return `D-Pad ${title(t.direction)}`
    case 'trigger':
      return `${t.side === 'LEFT' ? 'Left' : 'Right'} Trigger`
    case 'stick':
      return `${t.side === 'LEFT' ? 'Left' : 'Right'} Stick ${title(t.direction)}`
  }
}

export function keyLabel(key: string): string {
  // Cosmetic only; canonical names come from the backend.
  if (key.length === 1) return key.toUpperCase()
  if (key.startsWith('Num')) return `Numpad ${key.slice(3)}`
  return key
}
