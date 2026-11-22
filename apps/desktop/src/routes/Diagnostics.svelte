<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type Check } from '../lib/api'
  import Card from '../lib/components/Card.svelte'

  let checks = $state<Check[]>([])
  let exported = $state<string | null>(null)
  async function run() { checks = (await app.run(api.runDiagnostics))?.checks ?? [] }
  async function exportReport() { exported = (await app.run(api.exportDiagnostics)) ?? null }
  onMount(run)
  const label = { pass: 'Pass', warn: 'Check', fail: 'Problem', info: 'Info' }
</script>

<Card title="Checks">
  {#snippet actions()}
    <div class="row"><button onclick={run}>Run again</button><button class="primary" onclick={exportReport}>Export sanitised report</button></div>
  {/snippet}
  <table>
    <thead><tr><th>Result</th><th>Check</th><th>Details</th></tr></thead>
    <tbody>
      {#each checks as c}
        <tr>
          <td><b class={c.level}>{label[c.level]}</b></td>
          <td>{c.title}</td>
          <td>{c.detail}{#if c.hint}<p class="hint">{c.hint}</p>{/if}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if exported}<p class="hint">Report saved to <code>{exported}</code>. IP addresses, device identifiers and credential-like values are removed.</p>{/if}
  {#if !app.connection}<p class="hint">Select a console (Connections) to add console and capability checks.</p>{/if}
</Card>

<style>
  .row { display: flex; gap: var(--space-2); }
  b { font-size: 11px; font-weight: 600; } .pass { color: var(--ok); } .warn { color: var(--warn); } .fail { color: var(--bad); } .info { color: var(--text-muted); }
  .hint { color: var(--text-muted); font-size: 12px; margin-top: var(--space-2); } code { font-family: var(--mono); font-size: 12px; overflow-wrap: anywhere; }
</style>
