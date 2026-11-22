<script lang="ts">
  import { onMount } from 'svelte'
  import { app, sections } from './lib/app.svelte'
  import Sidebar from './lib/components/Sidebar.svelte'
  import Header from './lib/components/Header.svelte'
  import Overview from './routes/Overview.svelte'
  import Connections from './routes/Connections.svelte'
  import Settings from './routes/Settings.svelte'
  import Pending from './routes/Pending.svelte'
  import RemoteControl from './routes/RemoteControl.svelte'
  import Diagnostics from './routes/Diagnostics.svelte'

  onMount(() => app.init())

  function onKey(e: KeyboardEvent) {
    if (!e.altKey) return
    const i = Number(e.key) - 1
    if (i >= 0 && i < sections.length) { app.go(sections[i].id); e.preventDefault() }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="shell">
  <Sidebar />
  <div class="main">
    <Header />
    {#if app.error}<div class="error" role="alert">{app.error}</div>{/if}
    <main id="content" tabindex="-1">
      {#if app.section === 'overview'}<Overview />
      {:else if app.section === 'connections'}<Connections />
      {:else if app.section === 'remote'}<RemoteControl />
      {:else if app.section === 'diagnostics'}<Diagnostics />
      {:else if app.section === 'settings'}<Settings />
      {:else}<Pending id={app.section} />
      {/if}
    </main>
  </div>
</div>

<style>
  .shell { display: grid; grid-template-columns: 76px 1fr; height: 100%; }
  .main { display: flex; flex-direction: column; min-width: 0; }
  main { flex: 1; overflow: auto; padding: var(--space-5); }
  .error { margin: var(--space-3) var(--space-5) 0; padding: 10px 14px; border: 1px solid var(--bad); border-radius: var(--radius-sm); color: var(--bad); background: color-mix(in srgb, var(--bad) 10%, transparent); }
</style>
