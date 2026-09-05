import { useEffect, useRef, useState } from 'react'
import * as api from '../api'
import type { Snapshot } from '../types'
import { keyLabel, targetLabel } from '../types'
import { Button, Card, SectionTitle, StatusDot } from './ui'

const VIGEM_URL = 'https://github.com/nefarius/ViGEmBus/releases'
const TESTER_URL = 'https://gamepad-tester.com'

export default function Dashboard({
  snap,
  applySnapshot,
  onError,
  goTo,
}: {
  snap: Snapshot
  applySnapshot: (s: Snapshot) => void
  onError: (msg: string) => void
  goTo: (tab: 'dashboard' | 'mapping' | 'profiles', player: number) => void
}) {
  const [busy, setBusy] = useState(false)
  // Test-mode live feedback: last bound key press per player slot.
  const [feed, setFeed] = useState<Record<number, { key: string; label: string }>>({})

  // Mutations already return the fresh Snapshot - apply it directly instead of
  // issuing a second snapshot request per click.
  const run = async (p: Promise<Snapshot>, ok?: () => void) => {
    setBusy(true)
    try {
      applySnapshot(await p)
      ok?.()
    } catch (e) {
      onError(String(e))
    } finally {
      setBusy(false)
    }
  }

  // Latest snapshot kept in a ref so the test-event listener (registered once)
  // can resolve the controller-action label from current bindings.
  const playersRef = useRef(snap.players)
  useEffect(() => {
    playersRef.current = snap.players
  })

  useEffect(() => {
    let unlisten: (() => void) | undefined
    api
      .onTestEvent((e) => {
        if (!e.down) return // releasing a key clears nothing; show presses
        const players = playersRef.current
        const player = players.find((p) => p.index === e.player)
        const binding = player?.bindings.find((b) => b.key === e.key)
        if (!binding) return
        setFeed((f) => ({ ...f, [e.player]: { key: e.key, label: targetLabel(binding.target) } }))
      })
      .then((u) => (unlisten = u))
    return () => unlisten?.()
  }, [])

  const driver = snap.driver
  const listeningPlayer =
    snap.tapAssign === null ? null : snap.players.find((p) => p.index === snap.tapAssign)

  return (
    <div className="space-y-6">
      {/* Driver status */}
      {!driver.available ? (
        <Card className="border-amber-800/70 bg-amber-950/30">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 className="text-sm font-semibold text-amber-200">
                Virtual controller driver not found
              </h3>
              <p className="mt-1 max-w-2xl text-sm text-zinc-400">{driver.message}</p>
            </div>
            <a
              href={VIGEM_URL}
              target="_blank"
              rel="noreferrer"
              className="rounded-lg bg-amber-600 px-3 py-1.5 text-sm font-medium text-amber-950 hover:bg-amber-500"
            >
              Download ViGEmBus driver
            </a>
          </div>
          <ol className="mt-3 list-inside list-decimal space-y-1 text-xs text-zinc-500">
            <li>Download and run the ViGEmBus installer.</li>
            <li>Restart Windows if the installer asks for it (the driver loads at boot).</li>
            <li>Reopen Keyboard Splitter. Still not detected? Run this app as administrator.</li>
          </ol>
        </Card>
      ) : (
        <Card className="border-emerald-900/60 bg-emerald-950/20">
          <div className="flex items-center justify-between gap-3">
            <div>
              <h3 className="text-sm font-semibold text-emerald-300">Driver ready</h3>
              <p className="mt-0.5 text-sm text-zinc-400">
                ViGEmBus detected - virtual Xbox 360 controllers will be created when you assign keyboards.
              </p>
            </div>
            <StatusDot tone="green" label="ViGEmBus OK" />
          </div>
        </Card>
      )}

      {/* First-run checklist */}
      <SetupChecklist snap={snap} />

      {/* Engine control + test mode */}
      <Card>
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h3 className="text-sm font-semibold text-zinc-100">
              {snap.engineRunning ? 'Engine is running' : 'Engine is paused'}
            </h3>
            <p className="mt-0.5 text-sm text-zinc-500">
              {snap.engineRunning
                ? 'Keys on assigned keyboards are being converted to controllers.'
                : 'Key capture is off. Start the engine to play.'}
            </p>
          </div>
          <Button
            variant={snap.engineRunning ? 'secondary' : 'primary'}
            disabled={busy}
            onClick={() =>
              run(api.setRunning(!snap.engineRunning), () =>
                snap.engineRunning
                  ? onError('Engine paused - controllers keep their current state.')
                  : undefined,
              )
            }
          >
            {snap.engineRunning ? '■ Stop engine' : '▶ Start engine'}
          </Button>
        </div>

        {/* Test mode: watch bound keys become controller actions, live */}
        <div className="mt-3 flex flex-wrap items-center justify-between gap-3 border-t border-zinc-800 pt-3">
          <div>
            <p className="text-sm font-medium text-zinc-200">
              Test mode:{' '}
              <span className={snap.testMode ? 'text-emerald-300' : 'text-zinc-500'}>
                {snap.testMode ? 'on' : 'off'}
              </span>
            </p>
            <p className="mt-0.5 text-xs text-zinc-500">
              {snap.testMode && !snap.engineRunning
                ? 'Start the engine, then press bound keys to see them light up below.'
                : 'Verify a mapping before launching the game: press keys and watch the controller action they trigger.'}
            </p>
          </div>
          <Button
            variant={snap.testMode ? 'secondary' : 'ghost'}
            disabled={busy}
            title="Stream bound key presses to the dashboard for live verification"
            onClick={() => void run(api.setTestMode(!snap.testMode))}
            className={snap.testMode ? 'border-emerald-700 text-emerald-300' : ''}
          >
            {snap.testMode ? '● Watching inputs' : '○ Test inputs'}
          </Button>
        </div>
      </Card>

      {/* Press-a-key assignment banner */}
      {listeningPlayer && (
        <Card className="border-amber-700/70 bg-amber-950/30">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 className="text-sm font-semibold text-amber-200">
                Press a key to assign {listeningPlayer.name}'s keyboard
              </h3>
              <p className="mt-1 text-sm text-zinc-400">
                Press any key on the keyboard you want for {listeningPlayer.name} — it must not
                already belong to another player. Esc cancels.
              </p>
            </div>
            <Button variant="secondary" disabled={busy} onClick={() => void run(api.setTapAssign(null))}>
              Cancel
            </Button>
          </div>
        </Card>
      )}

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Keyboards */}
        <Card>
          <div className="flex items-center justify-between">
            <SectionTitle>Detected keyboards</SectionTitle>
            <Button
              variant="ghost"
              disabled={busy}
              onClick={() => void api.snapshot().then(applySnapshot).catch((e) => onError(String(e)))}
            >
              ↻ Refresh
            </Button>
          </div>
          {snap.devices.length === 0 ? (
            <p className="mt-3 text-sm text-zinc-500">
              No keyboards detected. Plug one in - this app watches for new devices live.
            </p>
          ) : (
            <ul className="mt-3 divide-y divide-zinc-800">
              {snap.devices.map((d) => {
                const assigned = snap.players.find((p) => p.keyboard === d.id)
                return (
                  <li key={d.id} className="flex flex-wrap items-center justify-between gap-2 py-2.5">
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium text-zinc-200">{d.name}</p>
                      <p className="text-xs text-zinc-600">
                        VID {d.vendorId.toString(16).padStart(4, '0')} · PID{' '}
                        {d.productId.toString(16).padStart(4, '0')}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      {assigned ? (
                        <StatusDot
                          tone={
                            assigned.controllerState.status === 'connected'
                              ? 'green'
                              : assigned.controllerState.status === 'error'
                                ? 'red'
                                : 'amber'
                          }
                          label={assigned.name}
                        />
                      ) : (
                        <span className="text-xs text-zinc-600">unassigned</span>
                      )}
                      <select
                        value={assigned?.index ?? ''}
                        onChange={async (e) => {
                          const v = e.target.value
                          if (v === '') {
                            // "unassign": detach this keyboard from its player
                            if (assigned) await run(api.assignKeyboard(assigned.index, null))
                            return
                          }
                          const target = Number(v)
                          if (assigned?.index === target) return // no-op
                          await run(api.assignKeyboard(target, d.id))
                        }}
                        className="rounded-lg border border-zinc-700 bg-zinc-800 px-2 py-1 text-xs text-zinc-200 outline-none focus:border-emerald-500"
                      >
                        <option value="">{assigned ? 'unassign' : '— assign —'}</option>
                        {snap.players.map((p) => (
                          <option key={p.index} value={p.index}>
                            {p.name}
                          </option>
                        ))}
                      </select>
                    </div>
                  </li>
                )
              })}
            </ul>
          )}
          <p className="mt-3 text-xs text-zinc-600">
            Tip: identical keyboards look the same here - use “press a key to assign” under a
            player instead of guessing which row is which.
          </p>
        </Card>

        {/* Players */}
        <Card>
          <div className="flex items-center justify-between">
            <SectionTitle>Players</SectionTitle>
            <Button
              variant="ghost"
              disabled={snap.players.length >= 4 || busy}
              onClick={() => run(api.addPlayer())}
              title="Add a player slot (XInput supports up to 4)"
            >
              + Add player
            </Button>
          </div>
          <ul className="mt-3 space-y-3">
            {snap.players.map((p) => {
              const tone = p.controllerState.status === 'connected'
                ? 'green'
                : p.controllerState.status === 'error'
                  ? 'red'
                  : p.keyboard
                    ? 'amber'
                    : 'zinc'
              const last = feed[p.index]
              return (
                <li
                  key={p.index}
                  className="rounded-lg border border-zinc-800 bg-zinc-900/80 px-3 py-2.5"
                >
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-semibold text-zinc-100">{p.name}</span>
                        <StatusDot tone={tone} label={controllerLabel(p)} />
                      </div>
                      <p className="mt-0.5 text-xs text-zinc-500">
                        {p.keyboardName ? `Keyboard: ${p.keyboardName}` : 'No keyboard assigned'} ·{' '}
                        {p.bindings.length} bindings
                      </p>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <Button
                        variant="ghost"
                        disabled={busy || snap.engineRunning || snap.tapAssign !== null}
                        title={
                          snap.engineRunning
                            ? 'Pause the engine, then press a key on the keyboard you want'
                            : `Press a key on the keyboard for ${p.name}`
                        }
                        onClick={() => run(api.setTapAssign(p.index))}
                      >
                        {p.keyboard ? '⌨ Reassign by key' : '⌨ Assign by key'}
                      </Button>
                      <Button
                        variant="secondary"
                        onClick={() => goTo('mapping', p.index)}
                        title={`Configure ${p.name}`}
                      >
                        Configure
                      </Button>
                      {snap.players.length > 2 && (
                        <Button
                          variant="ghost"
                          title={`Remove ${p.name}`}
                          onClick={() => run(api.removePlayer(p.index))}
                        >
                          ✕
                        </Button>
                      )}
                    </div>
                  </div>
                  {snap.testMode && last && (
                    <p className="mt-2 flex items-center gap-1.5 rounded-md bg-emerald-950/40 px-2 py-1 text-xs text-emerald-300">
                      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-400" />
                      Last input:{' '}
                      <kbd className="rounded border border-emerald-800 bg-zinc-900 px-1.5 py-0.5 font-mono">
                        {keyLabel(last.key)}
                      </kbd>
                      <span className="text-zinc-400">→</span>
                      {last.label}
                    </p>
                  )}
                </li>
              )
            })}
          </ul>
          <p className="mt-3 text-xs text-zinc-600">
            Assign a physical keyboard to each player slot to create its virtual Xbox 360
            controller (XInput: up to 4).
          </p>
        </Card>
      </div>

      {/* Troubleshooting / verification */}
      <Card>
        <SectionTitle>Not working? Read this first</SectionTitle>
        <ul className="mt-2 space-y-1.5 text-xs leading-relaxed text-zinc-500">
          <li>
            <span className="text-zinc-300">“The game doesn't see a controller.”</span> Xbox pads
            never show in the old <code className="text-zinc-400">joy.cpl</code> panel. While the
            engine runs, check the Windows game controller settings (Settings → Bluetooth &amp;
            devices → Game controllers) or{' '}
            <a
              href={TESTER_URL}
              target="_blank"
              rel="noreferrer"
              className="text-emerald-400 underline-offset-2 hover:underline"
            >
              gamepad-tester.com
            </a>{' '}
            — press a bound key and watch it register. Enable <span className="text-zinc-300">Test
            mode</span> above for the same feedback right here.
          </li>
          <li>
            <span className="text-zinc-300">Driver installed but no controller appears.</span> Run
            Keyboard Splitter as administrator and press “Reconnect controllers”. If the driver
            installer asked for a restart, do it — the bus only loads at boot.
          </li>
          <li>
            <span className="text-zinc-300">Works locally but not in an online match.</span> Some
            online modes (EA FC / FIFA, ranked fighters) run anti-cheat that blocks virtual
            controllers. That restriction is the game's, not this app's — use it for local and
            co-op play.
          </li>
          <li>
            <span className="text-zinc-300">Keys also type into the game or chat.</span> Raw Input
            observes keys but Windows does not let an app swallow them without a kernel driver.
            Controller-only games ignore the keyboard, so this rarely matters.
          </li>
        </ul>
      </Card>
    </div>
  )
}

// ---------------------------------------------------------------------------
// First-run checklist
// ---------------------------------------------------------------------------

function SetupChecklist({ snap }: { snap: Snapshot }) {
  const assigned = snap.players.filter((p) => p.keyboard).length
  const steps: {
    done: boolean
    title: string
    hint: string
  }[] = [
    {
      done: snap.driver.available,
      title: 'Install the virtual gamepad driver',
      hint: snap.driver.available
        ? 'ViGEmBus is installed and reachable.'
        : 'Download and run the ViGEmBus installer above; restart Windows if it asks.',
    },
    {
      done: snap.devices.length > 0,
      title: 'Plug in a keyboard per player',
      hint:
        snap.devices.length > 0
          ? `${snap.devices.length} keyboard${snap.devices.length === 1 ? '' : 's'} detected — plug one in per player.`
          : 'Plug in a keyboard — new devices appear here live.',
    },
    {
      done: assigned > 0,
      title: 'Give each player a keyboard',
      hint:
        assigned > 0
          ? `${assigned} player${assigned === 1 ? '' : 's'} assigned.`
          : 'Use “⌨ Assign by key” under a player and press any key on their keyboard.',
    },
    {
      done: snap.engineRunning,
      title: 'Start the engine',
      hint: snap.engineRunning
        ? 'Running — keys are being converted to controller input.'
        : 'Press “▶ Start engine” above once keyboards are assigned.',
    },
  ]
  const doneCount = steps.filter((s) => s.done).length

  return (
    <Card>
      <div className="flex items-center justify-between">
        <SectionTitle>Getting started</SectionTitle>
        <span className="text-xs text-zinc-600">
          {doneCount}/{steps.length}
        </span>
      </div>
      <ol className="mt-3 space-y-2">
        {steps.map((s, i) => (
          <li key={s.title} className="flex items-start gap-3">
            <span
              className={`mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[11px] font-bold ${
                s.done
                  ? 'bg-emerald-600 text-emerald-950'
                  : 'border border-zinc-700 text-zinc-500'
              }`}
            >
              {s.done ? '✓' : i + 1}
            </span>
            <div className="min-w-0">
              <p className={`text-sm font-medium ${s.done ? 'text-zinc-300' : 'text-zinc-100'}`}>
                {s.title}
              </p>
              <p className="text-xs text-zinc-600">{s.hint}</p>
            </div>
          </li>
        ))}
        {doneCount === steps.length && (
          <li className="flex items-start gap-3 rounded-lg border border-emerald-900/60 bg-emerald-950/30 px-3 py-2">
            <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-emerald-600 text-[11px] font-bold text-emerald-950">
              ✓
            </span>
            <div>
              <p className="text-sm font-semibold text-emerald-300">All set — open your game!</p>
              <p className="text-xs text-emerald-900">
                If a game doesn't detect a controller, read the “Not working?” card at the bottom.
              </p>
            </div>
          </li>
        )}
      </ol>
    </Card>
  )
}

function controllerLabel(p: {
  name: string
  keyboard?: string | null
  controllerState: { status: string; message?: string | null }
}) {
  const s = p.controllerState.status
  if (s === 'connected') return 'Virtual controller' // slot index comes from XInput
  if (s === 'error') return 'Controller error'
  return p.keyboard ? 'No controller yet' : 'No keyboard'
}
