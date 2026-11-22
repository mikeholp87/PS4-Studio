// Shared app state (Svelte 5 runes). Single source of truth for navigation, connection and theme.
import { api, errText, type ConnectionView, type CapabilityEntry } from './api'

export const sections = [
  { id: 'overview', label: 'Overview' },
  { id: 'connections', label: 'Connections' },
  { id: 'remote', label: 'Remote Control' },
  { id: 'studio', label: 'Dashboard Studio' },
  { id: 'homebrew', label: 'Homebrew' },
  { id: 'files', label: 'File Manager' },
  { id: 'backups', label: 'Backups' },
  { id: 'diagnostics', label: 'Diagnostics' },
  { id: 'settings', label: 'Settings' },
] as const
export type SectionId = (typeof sections)[number]['id']

class AppState {
  section = $state<SectionId>('overview')
  connection = $state<ConnectionView | null>(null)
  error = $state<string | null>(null)
  theme = $state<'dark' | 'light' | 'system'>('dark')
  accent = $state('#6d8cff')

  cap(id: string): CapabilityEntry | undefined {
    return this.connection?.capabilities.find((c) => c.capability === id)
  }
  /** Reason an action is disabled, or null when it may run. */
  blocked(id: string): string | null {
    if (!this.connection) return 'Connect to a console or the simulator first.'
    const c = this.cap(id)
    if (!c) return 'Capability not reported.'
    return c.status === 'available' ? null : (c.reason ?? 'Not verified.')
  }

  go(s: SectionId) { this.section = s }

  async run<T>(fn: () => Promise<T>): Promise<T | undefined> {
    try { this.error = null; return await fn() } catch (e) { this.error = errText(e); return undefined }
  }

  async refresh() { this.connection = (await this.run(api.getConnection)) ?? null }

  async disconnect() { await this.run(api.disconnect); this.connection = null }

  applyTheme() {
    const r = document.documentElement
    r.dataset.theme = this.theme
    r.style.setProperty('--accent', this.accent)
  }
  async setTheme(t: 'dark' | 'light' | 'system') { this.theme = t; this.applyTheme(); await this.run(() => api.setSetting('theme', t)) }
  async setAccent(hex: string) { this.accent = hex; this.applyTheme(); await this.run(() => api.setSetting('accent', hex)) }

  async init() {
    const t = await this.run(() => api.getSetting('theme'))
    if (t === 'dark' || t === 'light' || t === 'system') this.theme = t
    const a = await this.run(() => api.getSetting('accent'))
    if (a) this.accent = a
    this.applyTheme()
    await this.refresh()
  }
}
export const app = new AppState()
