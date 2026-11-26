<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type PackageInfo, type RemoteOp, type AuditEntry, type HashResult } from '../lib/api'
  import Card from '../lib/components/Card.svelte'
  import Gated from '../lib/components/Gated.svelte'
  import PlanDialog from '../lib/components/PlanDialog.svelte'

  let pkgs = $state<PackageInfo[] | null>(null)
  let checks = $state<Record<string, HashResult>>({})
  let trusted = $state('')
  let plan = $state<RemoteOp | null>(null)
  let history = $state<AuditEntry[]>([])
  let ackUnverified = $state<Record<string, boolean>>({})

  async function loadHistory() {
    history = ((await app.run(api.listAudit)) ?? []).filter((a) => a.action === 'files_apply' && a.target.includes('/data/pkg/'))
  }
  onMount(async () => { trusted = (await app.run(() => api.getSetting('trusted_publishers'))) ?? ''; await loadHistory() })

  async function scan() { const r = await app.run(api.hbPickAndScan); if (r) { pkgs = r; checks = {} } }
  async function verify(name: string) { const r = await app.run(() => api.hbVerify(name)); if (r) checks[name] = r }
  async function saveTrusted() { await app.run(() => api.setSetting('trusted_publishers', trusted)); await scan() }
  async function send(name: string) {
    const ack = !!ackUnverified[name]
    const op = await app.run(() => api.hbUploadOp(name, ack))
    if (op) plan = op
  }
  const sendReason = (p: PackageInfo) =>
    app.blocked('homebrew.ftp_read') ?? (!app.connection?.write_unlocked ? 'Enable changes in the File Manager first.' : checks[p.file]?.status === 'verified' || ackUnverified[p.file] ? null : 'Verify the checksum, or tick “I accept the risk” for an unverified package.')
  const when = (ts: number) => new Date(ts * 1000).toLocaleString()
  const mb = (n: number) => `${(n / (1 << 20)).toFixed(1)} MB`
  const reasonInstall = $derived(app.cap('homebrew.package_install')?.reason ?? 'Connect a console to see whether installation is possible.')
  const reasonPlugins = $derived(app.cap('homebrew.plugin_management')?.reason ?? 'Connect a console to see whether plugins can be managed.')
</script>

<div class="cols">
  <Card title="Installed on the console">
    <p class="hint">Not available. PS4 Studio has no verified way to list homebrew installed on a console, and it will not guess. Use the File Manager to look at folders the console exposes.</p>
    <h3>Installing</h3>
    <p class="hint">{reasonInstall}</p>
    <h3>Plugins</h3>
    <p class="hint">{reasonPlugins}</p>
  </Card>

  <Card title="Trusted publishers">
    <p class="hint">Packages whose manifest names one of these publishers are marked as trusted. Everything else shows a warning. Comma-separated.</p>
    <label>Publishers<input bind:value={trusted} maxlength="400" placeholder="e.g. My Studio, Friend Name" /></label>
    <button onclick={saveTrusted}>Save</button>
  </Card>

  <Card title="Local packages">
    {#snippet actions()}<button class="primary" onclick={scan}>Choose folder…</button>{/snippet}
    <p class="hint">Lists <code>.pkg</code> files in a folder. A sidecar <code>name.pkg.json</code> with <code>name, version, publisher, sha256</code> lets PS4 Studio check the file. PS4 Studio never runs or installs packages, and never downloads any. Only use files you have the right to use.</p>
    {#if pkgs === null}
      <p class="hint">No folder chosen yet.</p>
    {:else if pkgs.length === 0}
      <p class="hint">No .pkg files found in that folder.</p>
    {:else}
      <table>
        <thead><tr><th>Package</th><th>Publisher</th><th>Checks</th><th></th></tr></thead>
        <tbody>
          {#each pkgs as p}
            <tr>
              <td>{p.file}<br /><small>{mb(p.size)}{p.manifest ? ` · ${p.manifest.name} ${p.manifest.version}` : ''}</small></td>
              <td>{p.manifest?.publisher ?? '—'}{p.trusted ? ' (trusted)' : ''}</td>
              <td>
                {#each p.warnings as w}<p class="warn">{w}</p>{/each}
                {#if checks[p.file]}
                  <p class={checks[p.file].status === 'verified' ? 'ok' : 'bad'}>
                    {checks[p.file].status === 'verified' ? 'Checksum matches the manifest.' : checks[p.file].status === 'mismatch' ? 'CHECKSUM DOES NOT MATCH. Do not use this file.' : 'No manifest to compare against.'}
                  </p><small>sha256 {checks[p.file].sha256}</small>
                {/if}
              </td>
              <td class="end">
                <button onclick={() => verify(p.file)}>Verify</button>
                {#if checks[p.file]?.status !== 'verified'}<label class="check"><input type="checkbox" bind:checked={ackUnverified[p.file]} disabled={checks[p.file]?.status === 'mismatch'} /> I accept the risk</label>{/if}
                <Gated reason={checks[p.file]?.status === 'mismatch' ? 'Checksum mismatch.' : sendReason(p)} onclick={() => send(p.file)}>Copy to console…</Gated>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Card>

  <Card title="Copy history">
    {#if history.length === 0}<p class="hint">No packages copied yet.</p>{:else}
      <table><thead><tr><th>Time</th><th>Result</th><th>Details</th></tr></thead>
        <tbody>{#each history as h}<tr><td>{when(h.ts)}</td><td>{h.outcome}</td><td>{h.detail}</td></tr>{/each}</tbody></table>
    {/if}
  </Card>
</div>

{#if plan}<PlanDialog op={plan} onclose={() => (plan = null)} ondone={loadHistory} />{/if}

<style>
  .cols { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  :global(.cols > :nth-child(n + 3)) { grid-column: 1 / -1; }
  :global(section.card > label) { display: grid; gap: 4px; margin: var(--space-3) 0; }
  .hint, small { color: var(--text-muted); font-size: 12px; } .warn { color: var(--warn); font-size: 12px; margin-bottom: 4px; } .ok { color: var(--ok); } .bad { color: var(--bad); font-weight: 600; }
  .end { display: grid; gap: 6px; justify-items: start; } label.check { display: flex; gap: 6px; font-size: 12px; align-items: center; }
  code { font-family: var(--mono); font-size: 12px; } td p { margin: 0 0 4px; }
</style>
