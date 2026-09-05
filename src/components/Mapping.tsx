import { useEffect, useMemo, useRef, useState } from 'react'
import * as api from '../api'
import {
  keyLabel,
  targetLabel,
  type GamepadButton,
  type PresetMeta,
  type Snapshot,
  type Target,
} from '../types'
import { Button, Card, SectionTitle } from './ui'

export default function Mapping({
  snap,
  selectedPlayer,
  onSelectPlayer,
  refresh,
  onError,
}: {
  snap: Snapshot
  selectedPlayer: number
  onSelectPlayer: (index: number) => void
  refresh: () => Promise<void>
  onError: (msg: string) => void
}) {
  const player = snap.players.find((p) => p.index === selectedPlayer) ?? snap.players[0]
  const [busy, setBusy] = useState(false)
  const [capturing, setCapturing] = useState(false)
  const [pendingKey, setPendingKey] = useState<string | null>(null)
  const [nameDraft, setNameDraft] = useState<string | null>(null)
  const [presets, setPresets] = useState<PresetMeta[]>([])

  const playerRef = useRef(player)

  // Keep the latest player in a ref so the capture listener (registered once
  // per capture session) always sees the current assignment.
  useEffect(() => {
    playerRef.current = player
  })

  // Built-in game presets (loaded once; the backend owns the definitions).
  useEffect(() => {
    void api.listPresets().then(setPresets).catch((e) => onError(String(e)))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const run = async (p: Promise<unknown>) => {
    setBusy(true)
    try {
      await p
      await refresh()
    } catch (e) {
      onError(String(e))
    } finally {
      setBusy(false)
    }
  }

  // Live key capture: while enabled, the next key-down event from THIS
  // player's assigned keyboard becomes the "pending key".
  useEffect(() => {
    if (!capturing) return
    let unlisten: (() => void) | undefined
    api.onKeyEvent((e) => {
      if (!e.down) return
      const player_ = playerRef.current
      if (!player_.keyboard) {
        onError('Assign a keyboard to this player before capturing keys.')
        setCapturing(false)
        return
      }
      if (e.device !== player_.keyboard) return // ignore other keyboards
      setPendingKey(e.key)
      setCapturing(false)
    }).then((u) => (unlisten = u))
    return () => unlisten?.()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [capturing])

  if (!player) return null

  const assignTarget = (target: Target) => {
    if (!pendingKey) return
    const key = pendingKey
    void run(api.setBinding(player.index, { key, target }))
    setPendingKey(null)
  }

  const startCapture = () => {
    if (!player.keyboard) {
      onError('Assign a keyboard to this player first (Dashboard).')
      return
    }
    setPendingKey(null)
    setCapturing(true)
  }

  const saveName = () => {
    if (nameDraft && nameDraft.trim() && nameDraft.trim() !== player.name) {
      void run(api.renamePlayer(player.index, nameDraft.trim()))
    }
    setNameDraft(null)
  }

  return (
    <div className="space-y-6">
      {/* Player tabs */}
      <div className="flex flex-wrap items-center gap-2">
        {snap.players.map((p) => (
          <button
            key={p.index}
            type="button"
            onClick={() => {
              onSelectPlayer(p.index)
              setPendingKey(null)
              setCapturing(false)
            }}
            className={`rounded-lg px-3 py-1.5 text-sm font-medium transition-colors ${
              p.index === player.index
                ? 'bg-emerald-600 text-white'
                : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700'
            }`}
          >
            {p.name}
          </button>
        ))}
        <span className="ml-auto text-xs text-zinc-600">
          {snap.engineRunning ? 'engine running' : 'engine paused'}
        </span>
      </div>

      {/* Player header */}
      <Card>
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <label className="text-[11px] font-semibold uppercase tracking-widest text-zinc-500">
              Player name
            </label>
            <div className="mt-1 flex items-center gap-2">
              {nameDraft === null ? (
                <>
                  <span className="text-lg font-semibold text-zinc-100">{player.name}</span>
                  <Button variant="ghost" onClick={() => setNameDraft(player.name)}>
                    ✎
                  </Button>
                </>
              ) : (
                <>
                  <input
                    autoFocus
                    value={nameDraft}
                    onChange={(e) => setNameDraft(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') saveName()
                      if (e.key === 'Escape') setNameDraft(null)
                    }}
                    className="rounded-lg border border-zinc-700 bg-zinc-800 px-2 py-1 text-sm text-zinc-100 outline-none focus:border-emerald-500"
                  />
                  <Button variant="secondary" onClick={saveName}>
                    Save
                  </Button>
                </>
              )}
            </div>
          </div>
          <div className="text-right text-xs text-zinc-500">
            {player.keyboardName ? (
              <>
                <span className="text-zinc-300">{player.keyboardName}</span> assigned
              </>
            ) : (
              <span className="text-amber-400/90">No keyboard assigned yet</span>
            )}
            <div className="mt-1">
              <Button
                variant="secondary"
                disabled={busy}
                onClick={() => void run(api.reconnectControllers())}
              >
                ↻ Reconnect controllers
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <div className="grid gap-6 lg:grid-cols-5">
        {/* Binding list */}
        <Card className="lg:col-span-2">
          <div className="flex items-center justify-between">
            <SectionTitle>Key bindings</SectionTitle>
            <div className="flex gap-1.5">
              <Button
                variant="ghost"
                disabled={busy}
                onClick={() => void run(api.resetDefault(player.index))}
                title="Restore the starter layout"
              >
                Defaults
              </Button>
              <Button
                variant="ghost"
                disabled={busy || player.bindings.length === 0}
                onClick={() => void run(api.clearMapping(player.index))}
              >
                Clear
              </Button>
            </div>
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            <label
              htmlFor="preset-picker"
              className="text-[11px] font-semibold uppercase tracking-widest text-zinc-500"
            >
              Game preset
            </label>
            <select
              id="preset-picker"
              value=""
              disabled={busy}
              onChange={(e) => {
                const id = e.target.value
                if (!id) return
                void run(api.applyPreset(player.index, id))
              }}
              className="max-w-56 flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-2 py-1.5 text-xs text-zinc-200 outline-none focus:border-emerald-500"
            >
              <option value="">— choose a starter layout —</option>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <ul className="mt-3 space-y-1.5">
            {player.bindings.length === 0 && (
              <li className="py-6 text-center text-sm text-zinc-600">
                No bindings. Capture a key and pick a controller action.
              </li>
            )}
            {[...player.bindings]
              .sort((a, b) => a.key.localeCompare(b.key))
              .map((b) => (
                <li
                  key={b.key}
                  className="flex items-center justify-between gap-2 rounded-lg bg-zinc-800/60 px-2.5 py-1.5"
                >
                  <span className="flex items-center gap-2 text-sm">
                    <kbd className="rounded-md border border-zinc-700 bg-zinc-900 px-2 py-0.5 font-mono text-xs text-emerald-300">
                      {keyLabel(b.key)}
                    </kbd>
                    <span className="text-zinc-300">{targetLabel(b.target)}</span>
                  </span>
                  <Button
                    variant="ghost"
                    disabled={busy}
                    title={`Unbind ${keyLabel(b.key)}`}
                    onClick={() => void run(api.removeBinding(player.index, b.key))}
                  >
                    ✕
                  </Button>
                </li>
              ))}
          </ul>
          <p className="mt-3 text-xs leading-relaxed text-zinc-600">
            Presets are per-player templates (football, fighters, platformers…). Applying one
            replaces this player's bindings, then you can fine-tune below. Keys are tracked
            physically per keyboard, so every player can use the same preset on their own
            keyboard without conflicts.
          </p>
        </Card>

        {/* Capture + picker */}
        <Card className="lg:col-span-3">
          <SectionTitle>Bind a key</SectionTitle>

          <div className="mt-3 flex flex-wrap items-center gap-3 rounded-lg border border-dashed border-zinc-700 p-3">
            {pendingKey ? (
              <>
                <span className="text-sm text-zinc-400">
                  Key captured:
                  <kbd className="ml-2 rounded-md border border-emerald-700 bg-zinc-900 px-2 py-0.5 font-mono text-sm text-emerald-300">
                    {keyLabel(pendingKey)}
                  </kbd>
                </span>
                <Button variant="ghost" onClick={() => setPendingKey(null)}>
                  cancel
                </Button>
              </>
            ) : capturing ? (
              <span className="animate-pulse text-sm text-amber-300">
                Listening… press any key on your keyboard
                {player.keyboardName ? ` (${player.keyboardName})` : ''}
              </span>
            ) : snap.engineRunning ? (
              <span className="text-sm text-zinc-500">
                Pause the engine to capture keys for editing.
              </span>
            ) : (
              <>
                <span className="text-sm text-zinc-500">
                  Press a key on the player's keyboard, then choose its controller action:
                </span>
                <Button variant="primary" onClick={startCapture}>
                  ⌨ Capture key
                </Button>
              </>
            )}
          </div>

          {!pendingKey && !capturing && (
            <p className="mt-2 text-xs text-zinc-600">
              Pick an action below first to see captured keys here, or capture first and then
              choose an action.
            </p>
          )}

          <TargetPicker
            disabled={!pendingKey || busy}
            onPick={(t) => assignTarget(t)}
          />
        </Card>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------

const BUTTONS: GamepadButton[] = [
  'A', 'B', 'X', 'Y', 'LB', 'RB', 'BACK', 'START', 'LTHUMB', 'RTHUMB',
]

function TargetPicker({ disabled, onPick }: { disabled: boolean; onPick: (t: Target) => void }) {
  const groups = useMemo(() => {
    const g: { label: string; targets: { text: string; t: Target }[] }[] = [
      {
        label: 'Buttons',
        targets: BUTTONS.map((b) => ({ text: b, t: { type: 'button', button: b } as Target })),
      },
      {
        label: 'D-Pad',
        targets: (['UP', 'DOWN', 'LEFT', 'RIGHT'] as const).map((d) => ({
          text: d === 'UP' ? 'D-Pad ↑' : d === 'DOWN' ? 'D-Pad ↓' : d === 'LEFT' ? 'D-Pad ←' : 'D-Pad →',
          t: { type: 'dpad', direction: d },
        })),
      },
      {
        label: 'Left stick',
        targets: (['UP', 'DOWN', 'LEFT', 'RIGHT'] as const).map((d) => ({
          text: d === 'UP' ? 'LS ↑' : d === 'DOWN' ? 'LS ↓' : d === 'LEFT' ? 'LS ←' : 'LS →',
          t: { type: 'stick', side: 'LEFT', direction: d },
        })),
      },
      {
        label: 'Right stick',
        targets: (['UP', 'DOWN', 'LEFT', 'RIGHT'] as const).map((d) => ({
          text: d === 'UP' ? 'RS ↑' : d === 'DOWN' ? 'RS ↓' : d === 'LEFT' ? 'RS ←' : 'RS →',
          t: { type: 'stick', side: 'RIGHT', direction: d },
        })),
      },
      {
        label: 'Triggers',
        targets: [
          { text: 'LT', t: { type: 'trigger', side: 'LEFT' } },
          { text: 'RT', t: { type: 'trigger', side: 'RIGHT' } },
        ],
      },
    ]
    return g
  }, [])

  return (
    <div className={`mt-4 space-y-4 transition-opacity ${disabled ? 'opacity-40' : ''}`}>
      {groups.map((group) => (
        <div key={group.label}>
          <div className="mb-1.5 text-[11px] font-semibold uppercase tracking-widest text-zinc-600">
            {group.label}
          </div>
          <div className="flex flex-wrap gap-1.5">
            {group.targets.map(({ text, t }) => (
              <button
                key={text}
                type="button"
                disabled={disabled}
                onClick={() => onPick(t)}
                className="rounded-lg border border-zinc-700 bg-zinc-800/70 px-2.5 py-1.5 text-xs font-medium text-zinc-200 transition-colors hover:border-emerald-600 hover:text-emerald-300 disabled:cursor-not-allowed"
              >
                {text}
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  )
}
