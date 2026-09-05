import type { ReactNode } from 'react'

export function Button({
  children,
  onClick,
  variant = 'secondary',
  disabled,
  className = '',
  title,
}: {
  children: ReactNode
  onClick?: () => void
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  disabled?: boolean
  className?: string
  title?: string
}) {
  const styles: Record<string, string> = {
    primary:
      'bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm shadow-emerald-900/40',
    secondary:
      'bg-zinc-800 hover:bg-zinc-700 text-zinc-100 border border-zinc-700',
    danger:
      'bg-rose-950/70 hover:bg-rose-900 text-rose-200 border border-rose-900',
    ghost: 'bg-transparent hover:bg-zinc-800/70 text-zinc-300',
  }
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`inline-flex items-center justify-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${styles[variant]} ${className}`}
    >
      {children}
    </button>
  )
}

export function Card({
  children,
  className = '',
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <div className={`rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 ${className}`}>
      {children}
    </div>
  )
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h2 className="text-[11px] font-semibold uppercase tracking-widest text-zinc-500">
      {children}
    </h2>
  )
}

export type DotTone = 'green' | 'red' | 'amber' | 'zinc'

export function StatusDot({ tone, label }: { tone: DotTone; label: string }) {
  const colors: Record<DotTone, string> = {
    green: 'bg-emerald-500',
    red: 'bg-rose-500',
    amber: 'bg-amber-400',
    zinc: 'bg-zinc-600',
  }
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-zinc-300">
      <span className={`h-2 w-2 rounded-full ${colors[tone]}`} />
      {label}
    </span>
  )
}
