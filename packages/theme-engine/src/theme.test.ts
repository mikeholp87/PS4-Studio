import { describe, expect, it } from 'vitest'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { addTile, fromTemplate, moveFocus, moveTile, newTheme, parseThemeText, removeTile, templates, TEMPLATE_IDS, validateTheme } from './index'

const dir = (d: string) => join(__dirname, '../fixtures', d)
const files = (d: string) => readdirSync(dir(d)).filter((f) => f.endsWith('.json'))

describe('shared corpus (the Rust side runs the same files)', () => {
  it.each(files('valid'))('accepts valid/%s', (f) => {
    expect(validateTheme(JSON.parse(readFileSync(join(dir('valid'), f), 'utf8'))).ok).toBe(true)
  })
  it.each(files('invalid'))('rejects invalid/%s', (f) => {
    expect(validateTheme(JSON.parse(readFileSync(join(dir('invalid'), f), 'utf8'))).ok).toBe(false)
  })
})

describe('templates', () => {
  it.each(TEMPLATE_IDS)('%s is valid and original (no urls)', (id) => {
    expect(validateTheme(templates[id]).ok).toBe(true)
    expect(JSON.stringify(templates[id])).not.toMatch(/https?:|url\(|<script/i)
  })
})

describe('parseThemeText', () => {
  it('rejects garbage, oversized and non-objects without throwing', () => {
    expect(parseThemeText('{nope').ok).toBe(false)
    expect(parseThemeText('x'.repeat(300_000)).ok).toBe(false)
    expect(parseThemeText('null').ok).toBe(false)
    expect(parseThemeText('{"__proto__":{"x":1}}').ok).toBe(false)
  })
  it('round-trips an exported theme', () => {
    const t = fromTemplate('aurora')
    const r = parseThemeText(JSON.stringify(t))
    expect(r.ok && r.theme).toEqual(t)
  })
})

describe('editing helpers', () => {
  it('unique ids, add/move/remove tiles without mutating input', () => {
    const base = fromTemplate('grid')
    const t = newTheme('Grid', 'grid', ['grid'])
    expect(t.id).toBe('grid-2')
    const a = addTile(base, 'Games')
    expect(a.tiles.at(-1)!.id).toBe('games-2')
    expect(base.tiles.length).toBe(8)
    const m = moveTile(a, 'games', 1)
    expect(m.tiles[1].id).toBe('games')
    expect(moveTile(a, 'games', -1)).toBe(a)
    expect(removeTile(a, 'games').tiles.some((x) => x.id === 'games')).toBe(false)
    expect(validateTheme(a).ok).toBe(true)
  })
  it('works on Proxy-wrapped themes (Svelte $state)', () => {
    const t = new Proxy(fromTemplate('grid'), {})
    expect(addTile(t, 'X').tiles.length).toBe(9)
    expect(moveTile(t, 'media', -1).tiles[0].id).toBe('media')
    expect(removeTile(t, 'media').tiles.length).toBe(7)
  })
  it('refuses a 41st tile', () => {
    let t = fromTemplate('grid')
    for (let i = 0; i < 50; i++) t = addTile(t, `t${i}`)
    expect(t.tiles.length).toBe(40)
  })
})

describe('moveFocus', () => {
  it('moves within bounds and wraps when asked', () => {
    expect(moveFocus(0, 'left', 8, 4, false)).toBe(0)
    expect(moveFocus(0, 'left', 8, 4, true)).toBe(3)
    expect(moveFocus(3, 'right', 8, 4, true)).toBe(0)
    expect(moveFocus(1, 'down', 8, 4, false)).toBe(5)
    expect(moveFocus(5, 'down', 8, 4, false)).toBe(5)
    expect(moveFocus(5, 'down', 8, 4, true)).toBe(1)
    expect(moveFocus(6, 'down', 7, 4, false)).toBe(6) // no tile below a short last row
  })
  it('handles empty lists and short last rows', () => {
    expect(moveFocus(0, 'right', 0, 4, true)).toBe(0)
    expect(moveFocus(6, 'right', 7, 4, false)).toBe(6)
  })
})
