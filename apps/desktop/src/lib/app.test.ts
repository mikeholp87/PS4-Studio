import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
const { app } = await import('./app.svelte')

const conn = (status: 'available' | 'unavailable', reason: string | null) => ({
  profile: { id: 0, name: 'x', model: 'slim' as const, address: null, firmware: { value: null, source: 'unknown' as const }, simulated: true, last_connected: null },
  scenario_id: 'stock', simulated: true, write_unlocked: false,
  capabilities: [{ capability: 'homebrew.ftp_read', tier: 'homebrew' as const, status, reason, evidence: null }],
})

describe('app.blocked', () => {
  it('blocks with a reason when nothing is connected', () => {
    app.connection = null
    expect(app.blocked('homebrew.ftp_read')).toMatch(/Connect to a console/)
  })
  it('explains unavailable capabilities', () => {
    app.connection = conn('unavailable', 'No file service.')
    expect(app.blocked('homebrew.ftp_read')).toBe('No file service.')
  })
  it('allows available capabilities and rejects unreported ones', () => {
    app.connection = conn('available', null)
    expect(app.blocked('homebrew.ftp_read')).toBeNull()
    expect(app.blocked('nope')).toMatch(/not reported/)
  })
})
