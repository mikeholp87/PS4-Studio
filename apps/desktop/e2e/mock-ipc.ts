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
      const cur = () => fx.find((x: any) => x.scenario.id === conn?.scenario_id)
      const remoteTree: Record<string, any[]> = {
        '/': [{ name: 'data', kind: 'dir', size: 0 }, { name: 'system', kind: 'dir', size: 0 }],
        '/data': [{ name: 'homebrew', kind: 'dir', size: 0 }, { name: 'pkg', kind: 'dir', size: 0 }],
        '/data/homebrew': [{ name: 'readme.txt', kind: 'file', size: 14 }],
        '/data/pkg': [], '/system': [{ name: 'sys.dat', kind: 'file', size: 16 }],
      }
      const plans: Record<string, any> = {}
      const backups: any[] = []
      let hbDir = false
      const handlers: Record<string, (a: any) => any> = {
        files_connect: () => {
          const f = cur()
          if (f.ftp_script.kind === 'connected') conn.capabilities = f.after_ftp_probe
          else conn.capabilities = conn.capabilities.map((c: any) => c.capability.startsWith('homebrew.ftp') ? { ...c, status: 'unavailable', reason: 'File management is unavailable because no supported file service was detected.' } : c)
          conn.write_unlocked = false
          return conn
        },
        files_disconnect: () => { conn.write_unlocked = false; return conn },
        files_list_remote: (a) => remoteTree[a.path] ?? [],
        files_list_local: () => ({ path: '/home/u', parent: '/home', entries: [{ name: 'docs', kind: 'dir', size: 0 }, { name: 'game.pkg', kind: 'file', size: 2097152 }, { name: 'notes.txt', kind: 'file', size: 12 }] }),
        files_set_write_lock: (a) => {
          if (a.unlocked && !a.acknowledgeUnencrypted) throw 'You must acknowledge that FTP is unencrypted before enabling writes.'
          conn.write_unlocked = a.unlocked
          conn.capabilities = a.unlocked ? cur().after_unlock : cur().after_ftp_probe
          return conn
        },
        files_plan: (a) => {
          if (!conn.write_unlocked) throw 'Writes are locked. Unlock writes for this session to make changes.'
          const id = `p${Object.keys(plans).length}`
          const op = a.op
          const dest = op.remote ?? op.path ?? op.to
          if (!String(dest).startsWith('/data/')) throw 'changes are only allowed inside: /data'
          plans[id] = { op, changes: [op.kind === 'upload' ? `Upload ${op.local} to ${op.remote}` : `Change ${dest}`], warnings: ['FTP is not encrypted: the file and any login travel in clear text on your network.'], recovery: 'To undo, delete it on the console.', backup_remote: null, overwrites: false }
          return { plan_id: id, plan: plans[id] }
        },
        files_apply: (a) => {
          if (!plans[a.planId]) throw 'That plan has expired or was already used. Review the change again.'
          delete plans[a.planId]
          return { outcome: { lines: ['Uploaded 12 bytes', 'Verified: the console shows the expected result.'], backup: null, verified: true, recovery: 'To undo, delete it on the console.' }, backup_id: null }
        },
        companion_connect: (a) => {
          const f = cur()
          if (a.code !== '24681357') throw 'the pairing code was rejected (2 attempts left)'
          conn.capabilities = f.after_companion
          return conn
        },
        companion_send_theme: (a) => { if (!themes[a.id]) throw 'Save the theme first, then send it.'; return null },
        companion_disconnect: () => conn,
        files_cancel: () => null,
        files_download: (a) => `/home/u/${a.remote.split('/').pop()}`,
        backup_create: (a) => {
          const m = { version: 1, id: `100-${backups.length}`, created: 1700000000, label: a.label || 'PS4 Studio backup', items: [{ category: 'settings', original: 'settings.json', stored: 'files/000-settings.json', size: 20, sha256: 'ab'.repeat(32) }, { category: 'remote_file', original: '/data/x.cfg', stored: 'files/001-x.cfg', size: 3, sha256: 'cd'.repeat(32) }] }
          backups.unshift(m); return m
        },
        backup_list: () => backups,
        backup_verify: () => ({ ok: true, problems: [] }),
        backup_delete: (a) => { const i = backups.findIndex((b) => b.id === a.id); if (i >= 0) backups.splice(i, 1); return null },
        backup_restore_item: (a) => { if (backups.find((b) => b.id === a.id)?.items[a.index].category === 'remote_file') throw 'Files from a console are not restored automatically.'; return 'Restored 1 settings.' },
        backup_save_item_as: () => '/home/u/saved.txt',
        hb_pick_and_scan: () => { hbDir = true; return [{ file: 'demo.pkg', size: 3145728, manifest: { name: 'Demo', version: '1.0', publisher: 'Acme', sha256: '0'.repeat(64) }, trusted: false, warnings: ['Unfamiliar publisher “Acme”. Only continue if you got this file from a source you trust.', 'Installing homebrew can crash or destabilise a console.'] }, { file: 'bare.pkg', size: 10, manifest: null, trusted: false, warnings: ['No manifest (.json) next to this file, so its origin and integrity cannot be checked.'] }] },
        hb_verify: (a) => (a.name === 'demo.pkg' ? { sha256: '0'.repeat(64), status: 'verified' } : { sha256: '1'.repeat(64), status: 'no_manifest' }),
        hb_upload_op: (a) => { if (!hbDir) throw 'Choose a folder first.'; return { kind: 'upload', local: `/home/u/pkgs/${a.name}`, remote: `/data/pkg/${a.name}` } },
        app_info: () => ({ version: '0.1.0', data_dir: '/home/u/.local/share/ps4-studio', config_dir: '/home/u/.config/ps4-studio' }),
        get_setting: (a) => settings[a.key] ?? null,
        set_setting: (a) => { settings[a.key] = a.value; return null },
        list_scenarios: () => fx.map((f: any) => f.scenario),
        connect_simulator: (a) => {
          const f = fx.find((x: any) => x.scenario.id === a.scenarioId)
          conn = {
            profile: { id: 0, name: f.scenario.console_name, model: 'slim', address: null, firmware: { value: f.scenario.reported_firmware, source: f.scenario.reported_firmware ? 'reported' : 'unknown' }, simulated: true, last_connected: null },
            scenario_id: f.scenario.id, simulated: true, write_unlocked: false, capabilities: f.capabilities,
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
