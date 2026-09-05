import { useState } from 'react'
import * as api from '../api'
import type { Snapshot } from '../types'
import { Button, Card, SectionTitle, StatusDot } from './ui'

const VIGEM_URL = 'https://github.com/nefarius/ViGEmBus/releases'

export default function Dashboard({
  snap,
  refresh,
  onError,
  goTo,
}: {
  snap: Snapshot
  refresh: () => Promise<void>
  onError: (msg: string) => void
  goTo: (tab: 'dashboard' | 'mapping' | 'profiles', player: number) => void
}) {
  const [busy, setBusy] = useState(false)

  const run = async (p: Promise<Snapshot>, ok?: () => void) => {
    setBusy(true)
    try {
      await p
      ok?.()
      await refresh()
    } catch (e) {
      onError(String(e))
    } finally {
      setBusy(false)
    }
  }

  const driver = snap.driver
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

      {/* Engine control */}
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
      </Card>

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Keyboards */}
        <Card>
          <div className="flex items-center justify-between">
            <SectionTitle>Detected keyboards</SectionTitle>
            <Button variant="ghost" disabled={busy} onClick={() => void refresh()}>
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
                          const target = v === '' ? (assigned ? assigned.index : null) : Number(v)
                          if (target === null || assigned?.index === target) return
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
            Tip: plug in the keyboards for each player, then assign them below.
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
              return (
                <li
                  key={p.index}
                  className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-900/80 px-3 py-2.5"
                >
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
                  <div className="flex items-center gap-2">
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
    </div>
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
