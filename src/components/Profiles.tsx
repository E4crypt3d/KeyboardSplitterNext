import { useEffect, useState } from 'react'
import * as api from '../api'
import type { Snapshot } from '../types'
import { Button, Card, SectionTitle } from './ui'

export default function Profiles({
  snap,
  refresh,
  onError,
}: {
  snap: Snapshot
  refresh: () => Promise<void>
  onError: (msg: string) => void
}) {
  const [profiles, setProfiles] = useState<string[]>([])
  const [name, setName] = useState('')
  const [busy, setBusy] = useState(false)

  const loadList = async () => {
    try {
      setProfiles(await api.listProfiles())
    } catch (e) {
      onError(String(e))
    }
  }

  useEffect(() => {
    // Reload the profile list whenever the active profile changes.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void loadList()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [snap.activeProfile])

  const run = async (p: Promise<unknown>) => {
    setBusy(true)
    try {
      await p
      await refresh()
      await loadList()
    } catch (e) {
      onError(String(e))
    } finally {
      setBusy(false)
    }
  }

  const save = () => {
    const trimmed = name.trim()
    if (!trimmed) return
    void run(api.saveProfile(trimmed)).then(() => setName(''))
  }

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <Card>
        <SectionTitle>Save current setup</SectionTitle>
        <p className="mt-2 text-sm text-zinc-500">
          Everything is auto-saved into the active profile below. Saving with a new name
          switches the active profile, so later edits keep going there.
        </p>
        <div className="mt-4 flex items-center gap-2">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') save()
            }}
            placeholder="e.g. Cuphead 2P"
            className="flex-1 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 outline-none placeholder:text-zinc-600 focus:border-emerald-500"
          />
          <Button variant="primary" disabled={busy || !name.trim()} onClick={save}>
            Save as…
          </Button>
        </div>
        <p className="mt-4 text-xs text-zinc-600">
          Profiles store per-player names, keyboard assignments and every key binding for all
          players.
        </p>
      </Card>

      <Card>
        <SectionTitle>Profiles</SectionTitle>
        {profiles.length === 0 ? (
          <p className="mt-3 text-sm text-zinc-500">
            No saved profiles yet - make your first one on the left.
          </p>
        ) : (
          <ul className="mt-3 space-y-2">
            {profiles.map((p) => {
              const active = p === snap.activeProfile
              return (
                <li
                  key={p}
                  className={`flex items-center justify-between gap-3 rounded-lg border px-3 py-2 ${
                    active ? 'border-emerald-800 bg-emerald-950/30' : 'border-zinc-800 bg-zinc-900'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-zinc-200">{p}</span>
                    {active && (
                      <span className="rounded-full bg-emerald-700/40 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-300">
                        active
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Button
                      variant="secondary"
                      disabled={busy || active}
                      onClick={() => void run(api.loadProfile(p))}
                      title={active ? 'Already active' : 'Load this profile'}
                    >
                      Load
                    </Button>
                    <Button
                      variant="danger"
                      disabled={busy || active}
                      onClick={() => void run(api.deleteProfile(p))}
                      title={active ? 'Active profiles cannot be deleted' : 'Delete profile'}
                    >
                      Delete
                    </Button>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </Card>
    </div>
  )
}
