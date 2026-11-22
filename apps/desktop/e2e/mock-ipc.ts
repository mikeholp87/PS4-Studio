import type { Page } from '@playwright/test'
import fixtures from './fixtures/scenarios.json' with { type: 'json' }
import importable from '../../../packages/theme-engine/fixtures/valid/minimal.json' with { type: 'json' }

/** Installs a fake `window.__TAURI_INTERNALS__` implementing the app's commands in memory. */
export async function mockTauri(page: Page, opts: { chiaki?: boolean } = {}) {
  await page.addInitScript(
    ({ fx, chiaki, imp }) => {
      const themes: Record<string, string> = {}
      const settings: Record<string, string> = {}
      const profiles: any[] = []
      const audit: any[] = []
      let conn: any = null
      let nextId = 1
      const log = (action: string, target: string) => audit.unshift({ id: audit.length + 1, ts: Math.floor(Date.now() / 1000), action, target, detail: '', outcome: 'ok' })
      const handlers: Record<string, (a: any) => any> = {
        app_info: () => ({ version: '0.1.0', data_dir: '/home/u/.local/share/ps4-studio', config_dir: '/home/u/.config/ps4-studio' }),
        get_setting: (a) => settings[a.key] ?? null,
        set_setting: (a) => { settings[a.key] = a.value; return null },
        list_scenarios: () => fx.map((f: any) => f.scenario),
        connect_simulator: (a) => {
          const f = fx.find((x: any) => x.scenario.id === a.scenarioId)
          conn = {
            profile: { id: 0, name: f.scenario.console_name, model: 'slim', address: null, firmware: { value: f.scenario.reported_firmware, source: f.scenario.reported_firmware ? 'reported' : 'unknown' }, simulated: true, last_connected: null },
            scenario_id: f.scenario.id, simulated: true, capabilities: f.capabilities,
          }
          log('connect_simulator', f.scenario.id)
          return conn
        },
        disconnect: () => { conn = null; return null },
        get_connection: () => conn,
        list_profiles: () => profiles,
        add_profile: (a) => {
          if (!/^\d+\.\d+\.\d+\.\d+$/.test(a.profile.address)) throw 'Address must be an IP address such as 192.168.1.50.'
          const p = { id: nextId++, name: a.profile.name, model: a.profile.model, address: a.profile.address, firmware: { value: a.profile.firmware, source: a.profile.firmware ? 'user_entered' : 'unknown' }, simulated: false, last_connected: null }
          profiles.push(p)
          return p
        },
        delete_profile: (a) => { const i = profiles.findIndex((p) => p.id === a.id); if (i >= 0) profiles.splice(i, 1); return i >= 0 },
        list_audit: () => audit,
        discover_lan: () => [{ address: '192.168.1.50', name: 'PS4-TEST', state: 'ready', firmware_hint: '9.00' }],
        remote_play_status: () => ({ installed: chiaki, binary: chiaki ? '/usr/bin/chiaki' : null, nickname: settings.chiaki_nickname ?? null, running: false }),
        run_diagnostics: () => ({ checks: [{ id: 'chiaki', title: 'chiaki-ng (Remote Play)', level: chiaki ? 'pass' : 'warn', detail: chiaki ? 'Found' : 'Not found on PATH or as a Flatpak.', hint: null }] }),
        list_themes: () => Object.entries(themes).map(([id, json]) => ({ id, name: JSON.parse(json).name, json })),
        save_theme: (a) => { const t = JSON.parse(a.json); themes[t.id] = a.json; return { id: t.id, name: t.name, json: a.json } },
        delete_theme: (a) => delete themes[a.id],
        export_theme: (a) => (themes[a.id] ? `/home/u/${a.id}.ps4studio-theme.json` : null),
        import_theme: () => { const t = { ...imp, id: 'imported', name: 'Imported theme' }; themes.imported = JSON.stringify(t); return { id: 'imported', name: t.name, json: themes.imported } },
        export_diagnostics: () => '/home/u/.local/share/ps4-studio/reports/diagnostics-1.txt',
      }
      ;(window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
          const h = handlers[cmd]
          if (!h) throw `unknown command ${cmd}`
          return h(args ?? {})
        },
        transformCallback: () => 0,
      }
    },
    { fx: fixtures as any, chiaki: opts.chiaki ?? false, imp: importable as any },
  )
}
