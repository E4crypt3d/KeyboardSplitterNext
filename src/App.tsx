import { useCallback, useEffect, useRef, useState } from 'react'
import * as api from './api'
import Dashboard from './components/Dashboard'
import Mapping from './components/Mapping'
import Profiles from './components/Profiles'
import type { Snapshot } from './types'

type Tab = 'dashboard' | 'mapping' | 'profiles'

const TABS: { id: Tab; label: string }[] = [
  { id: 'dashboard', label: 'Dashboard' },
  { id: 'mapping', label: 'Mapping' },
  { id: 'profiles', label: 'Profiles' },
]

export default function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null)
  const [tab, setTab] = useState<Tab>('dashboard')
  const [player, setPlayer] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const errorTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const showError = useCallback((msg: string) => {
    setError(msg)
    if (errorTimer.current) clearTimeout(errorTimer.current)
    errorTimer.current = setTimeout(() => setError(null), 6000)
  }, [])

  const refresh = useCallback(async () => {
    try {
      setSnap(await api.snapshot())
    } catch (e) {
      showError(String(e))
    }
  }, [showError])

  useEffect(() => {
    // Data bootstrapping: fetch once on mount (async, not a sync render cascade).
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void refresh()
    let unlisten: (() => void) | undefined
    void api.onEngineChanged(() => void refresh()).then((u) => (unlisten = u))
    return () => unlisten?.()
  }, [refresh])

  // The mapping view keyed by its selected player stays mounted while the tab
  // is visible; keep the editor state fresh on external changes.
  const goTo = useCallback((target: Tab, playerIndex: number) => {
    if (target === 'mapping') setPlayer(playerIndex)
    setTab(target)
  }, [])

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto max-w-6xl px-6 py-6">
        {/* Header */}
        <header className="mb-6 flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-600 text-base font-black text-zinc-950">
              ⌨
            </div>
            <div>
              <h1 className="text-lg font-bold leading-tight tracking-tight">
                Keyboard Splitter
              </h1>
              <p className="text-xs text-zinc-500">
                Multiple keyboards → independent virtual Xbox controllers
              </p>
            </div>
          </div>
          <div className="flex items-center gap-1 rounded-lg border border-zinc-800 bg-zinc-900 p-1">
            {TABS.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => setTab(t.id)}
                className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                  tab === t.id
                    ? 'bg-zinc-700 text-white'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                {t.label}
              </button>
            ))}
          </div>
        </header>

        {/* Error toast */}
        {error && (
          <div className="mb-4 rounded-lg border border-rose-800 bg-rose-950/60 px-4 py-2.5 text-sm text-rose-200">
            {error}
            <button
              type="button"
              className="ml-3 text-rose-400 hover:text-rose-200"
              onClick={() => setError(null)}
            >
              dismiss
            </button>
          </div>
        )}

        {/* Content */}
        {snap ? (
          tab === 'dashboard' ? (
            <Dashboard snap={snap} refresh={refresh} onError={showError} goTo={goTo} />
          ) : tab === 'mapping' ? (
            <Mapping
              snap={snap}
              selectedPlayer={Math.min(player, Math.max(0, snap.players.length - 1))}
              onSelectPlayer={setPlayer}
              refresh={refresh}
              onError={showError}
            />
          ) : (
            <Profiles snap={snap} refresh={refresh} onError={showError} />
          )
        ) : (
          <div className="flex h-64 items-center justify-center text-sm text-zinc-500">
            Connecting to the engine…
          </div>
        )}
      </div>
    </div>
  )
}
