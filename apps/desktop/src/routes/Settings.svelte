<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type AppInfo, type AuditEntry } from '../lib/api'
  import Card from '../lib/components/Card.svelte'

  let info = $state<AppInfo | null>(null)
  let audit = $state<AuditEntry[]>([])
  onMount(async () => { info = (await app.run(api.appInfo)) ?? null; audit = (await app.run(api.listAudit)) ?? [] })
  const when = (ts: number) => new Date(ts * 1000).toLocaleString()
</script>

<div class="cols">
  <Card title="Appearance">
    <label>Theme
      <select value={app.theme} onchange={(e) => app.setTheme(e.currentTarget.value as 'dark' | 'light' | 'system')}>
        <option value="dark">Dark</option><option value="light">Light</option><option value="system">Follow system</option>
      </select>
    </label>
    <label>Accent colour
      <input type="color" value={app.accent} onchange={(e) => app.setAccent(e.currentTarget.value)} />
    </label>
  </Card>

  <Card title="About">
    <dl>
      <dt>Version</dt><dd>{info?.version}</dd>
      <dt>Data</dt><dd><code>{info?.data_dir}</code></dd>
      <dt>Config</dt><dd><code>{info?.config_dir}</code></dd>
    </dl>
    <p class="hint">PS4 Studio is an independent project and is not endorsed by Sony Interactive Entertainment.</p>
  </Card>

  <Card title="Activity log">
    {#if audit.length === 0}<p class="hint">Nothing recorded yet.</p>{:else}
      <table>
        <thead><tr><th>Time</th><th>Action</th><th>Target</th><th>Result</th></tr></thead>
        <tbody>{#each audit as a}<tr><td>{when(a.ts)}</td><td>{a.action}</td><td>{a.target}</td><td>{a.outcome}</td></tr>{/each}</tbody>
      </table>
    {/if}
  </Card>
</div>

<style>
  .cols { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  :global(.cols > :last-child) { grid-column: 1 / -1; }
  :global(section.card > label) { display: grid; gap: var(--space-2); margin-bottom: var(--space-3); }
  dl { display: grid; grid-template-columns: 80px 1fr; gap: 6px var(--space-3); margin: 0; } dt { color: var(--text-muted); } dd { margin: 0; overflow-wrap: anywhere; }
  code { font-family: var(--mono); font-size: 12px; }
  .hint { color: var(--text-muted); font-size: 12px; margin-top: var(--space-3); }
</style>
