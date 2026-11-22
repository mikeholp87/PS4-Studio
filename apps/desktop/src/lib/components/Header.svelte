<script lang="ts">
  import { app, sections } from '../app.svelte'
  const title = $derived(sections.find((s) => s.id === app.section)?.label ?? '')
  const status = $derived(
    !app.connection ? { text: 'Not connected', cls: 'off' }
      : { text: `${app.connection.profile.name}`, cls: 'on' })
</script>

<header>
  <h1>{title}</h1>
  <div class="right">
    {#if app.connection?.simulated}<span class="sim" title="All data comes from the built-in simulator, not a real console.">SIMULATED</span>{/if}
    <span class="pill {status.cls}" role="status"><i></i>{status.text}</span>
  </div>
</header>

<style>
  header { display: flex; align-items: center; justify-content: space-between; padding: var(--space-4) var(--space-5); border-bottom: 1px solid var(--border); background: var(--surface); }
  .right { display: flex; gap: var(--space-3); align-items: center; }
  .pill { display: inline-flex; align-items: center; gap: 8px; padding: 5px 12px; border: 1px solid var(--border); border-radius: 999px; font-size: 13px; background: var(--bg); }
  .pill i { width: 8px; height: 8px; border-radius: 50%; background: var(--text-muted); }
  .pill.on i { background: var(--ok); }
  .sim { font-size: 11px; font-weight: 700; letter-spacing: .08em; padding: 3px 8px; border-radius: 4px; background: color-mix(in srgb, var(--warn) 20%, transparent); color: var(--warn); border: 1px solid var(--warn); }
</style>
