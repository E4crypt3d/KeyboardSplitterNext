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

  useEffect(() => () => {
    // Clear the pending error toast timer on unmount (no stray setState).
    if (errorTimer.current) clearTimeout(errorTimer.current)
  }, [])

  // applySnapshot is the single place snapshots enter state. Every mutation
  // already answers with a fresh Snapshot, so callers pass it straight here
  // instead of asking the engine again (halves the IPC round-trips).
  const applySnapshot = useCallback((next: Snapshot) => setSnap(next), [])

  const refresh = useCallback(async () => {
    try {
      applySnapshot(await api.snapshot())
    } catch (e) {
      showError(String(e))
    }
  }, [applySnapshot, showError])

  useEffect(() => {
    // Data bootstrapping: fetch once on mount (async, not a sync render cascade).
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void refresh()
    let unlisten: (() => void) | undefined
    // Backend emits "engine:changed" in bursts (e.g. a key event can recover a
    // controller and re-emit). Coalesce to at most one snapshot per animation
    // frame so the UI stays live without hammering IPC.
    let scheduled = false
    void api.onEngineChanged(() => {
      if (scheduled) return
      scheduled = true
      requestAnimationFrame(() => {
        scheduled = false
        void refresh()
      })
    }).then((u) => (unlisten = u))
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
            <a
              href="https://github.com/E4crypt3d/KeyboardSplitterNext"
              target="_blank"
              rel="noreferrer"
              aria-label="GitHub repository"
              className="ml-1 inline-flex h-8 w-8 items-center justify-center rounded-md text-zinc-500 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
            >
              <svg viewBox="0 0 16 16" width="17" height="17" fill="currentColor" aria-hidden="true">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
              </svg>
            </a>
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
            <Dashboard
              snap={snap}
              applySnapshot={applySnapshot}
              onError={showError}
              goTo={goTo}
            />
          ) : tab === 'mapping' ? (
            <Mapping
              snap={snap}
              selectedPlayer={Math.min(player, Math.max(0, snap.players.length - 1))}
              onSelectPlayer={setPlayer}
              applySnapshot={applySnapshot}
              onError={showError}
            />
          ) : (
            <Profiles snap={snap} applySnapshot={applySnapshot} onError={showError} />
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
