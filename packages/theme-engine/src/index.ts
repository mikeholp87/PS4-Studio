import minimal from './templates/minimal.json' with { type: 'json' }
import aurora from './templates/aurora.json' with { type: 'json' }
import grid from './templates/grid.json' with { type: 'json' }
import classic from './templates/classic.json' with { type: 'json' }
import type { IconName, TemplateId, Theme, Tile } from './types'

export * from './types'
export * from './validate'

export const templates: Record<TemplateId, Theme> = { minimal, aurora, grid, classic } as Record<TemplateId, Theme>

// JSON clone, not structuredClone: callers may pass reactive (Proxy) objects, which structuredClone rejects.
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v))

export function fromTemplate(t: TemplateId): Theme {
  return clone(templates[t])
}

export function slugify(s: string): string {
  const slug = s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 30)
  return slug || 'theme'
}

/** Returns `base`, or `base-2`, `base-3`… not present in `taken`. */
export function uniqueId(base: string, taken: Iterable<string>): string {
  const set = new Set(taken)
  if (!set.has(base)) return base
  for (let i = 2; ; i++) {
    const c = `${base.slice(0, 36)}-${i}`
    if (!set.has(c)) return c
  }
}

export function newTheme(name: string, template: TemplateId, takenIds: Iterable<string>): Theme {
  const t = fromTemplate(template)
  t.name = name.trim().slice(0, 48) || 'My theme'
  t.id = uniqueId(slugify(t.name), takenIds)
  return t
}

export function addTile(t: Theme, label: string, icon: IconName = 'star'): Theme {
  if (t.tiles.length >= 40) return t
  const next = clone(t)
  const tile: Tile = { id: uniqueId(slugify(label), t.tiles.map((x) => x.id)), label: label.trim().slice(0, 32) || 'Shortcut', icon, color: t.colors.accent, kind: 'shortcut' }
  next.tiles.push(tile)
  return next
}

export function removeTile(t: Theme, id: string): Theme {
  return { ...clone(t), tiles: t.tiles.filter((x) => x.id !== id).map(clone) }
}

export function moveTile(t: Theme, id: string, delta: -1 | 1): Theme {
  const i = t.tiles.findIndex((x) => x.id === id)
  const j = i + delta
  if (i < 0 || j < 0 || j >= t.tiles.length) return t
  const next = clone(t)
  ;[next.tiles[i], next.tiles[j]] = [next.tiles[j], next.tiles[i]]
  return next
}

export const FONT_STACKS = {
  system: 'system-ui, -apple-system, "Segoe UI", Roboto, "Noto Sans", sans-serif',
  serif: 'Georgia, "Noto Serif", "DejaVu Serif", serif',
  mono: 'ui-monospace, "DejaVu Sans Mono", monospace',
  rounded: 'ui-rounded, "Nunito", "Quicksand", system-ui, sans-serif',
} as const

/** Focus movement on a grid of `count` tiles with `cols` columns. Pure; used by keyboard and gamepad. */
export function moveFocus(index: number, dir: 'left' | 'right' | 'up' | 'down', count: number, cols: number, wrap: boolean): number {
  if (count === 0) return 0
  const safeCols = Math.max(1, Math.floor(cols) || 1)
  const row = Math.floor(index / safeCols)
  const col = index % safeCols
  const rows = Math.ceil(count / safeCols)
  let r = row, c = col
  if (dir === 'left') c--
  if (dir === 'right') c++
  if (dir === 'up') r--
  if (dir === 'down') r++
  if (wrap) {
    if (c < 0) c = safeCols - 1
    if (c >= safeCols) c = 0
    if (r < 0) r = rows - 1
    if (r >= rows) r = 0
  }
  c = Math.max(0, Math.min(safeCols - 1, c))
  r = Math.max(0, Math.min(rows - 1, r))
  let n = r * safeCols + c
  if (n >= count) n = wrap && (dir === 'right' || dir === 'down') ? 0 : dir === 'left' || dir === 'up' ? index : count - 1
  return n
}
