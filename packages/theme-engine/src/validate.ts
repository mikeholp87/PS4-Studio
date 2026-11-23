// Precompiled at build time (scripts/gen-validator.mjs): no runtime eval, so it works under a strict CSP.
import check from './validator.generated.js'
import type { Theme } from './types'

export type ValidationResult = { ok: true; theme: Theme } | { ok: false; errors: string[] }

/** Strictly validates untrusted JSON (parsed value). Never throws. Themes are data only. */
export function validateTheme(value: unknown): ValidationResult {
  if (check(value)) {
    const t = value as unknown as Theme
    const ids = new Set<string>()
    for (const tile of t.tiles) {
      if (ids.has(tile.id)) return { ok: false, errors: [`duplicate tile id "${tile.id}"`] }
      ids.add(tile.id)
    }
    return { ok: true, theme: t }
  }
  return { ok: false, errors: (check.errors ?? []).map((e) => `${e.instancePath || '/'} ${e.message}`) }
}

export const MAX_THEME_BYTES = 256 * 1024

/** Parses and validates theme JSON text from a file/clipboard. */
export function parseThemeText(text: string): ValidationResult {
  if (text.length > MAX_THEME_BYTES) return { ok: false, errors: ['theme file is too large'] }
  try {
    return validateTheme(JSON.parse(text))
  } catch {
    return { ok: false, errors: ['not valid JSON'] }
  }
}
