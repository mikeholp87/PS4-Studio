<script lang="ts">
  import { onMount } from 'svelte'
  import { app, sections } from './lib/app.svelte'
  import Sidebar from './lib/components/Sidebar.svelte'
  import Header from './lib/components/Header.svelte'
  import Overview from './routes/Overview.svelte'
  import Connections from './routes/Connections.svelte'
  import Settings from './routes/Settings.svelte'
  import RemoteControl from './routes/RemoteControl.svelte'
  import Diagnostics from './routes/Diagnostics.svelte'
  import Studio from './routes/Studio.svelte'
  import FileManager from './routes/FileManager.svelte'
  import Homebrew from './routes/Homebrew.svelte'
  import Backups from './routes/Backups.svelte'
  import ConsoleSim from './routes/ConsoleSim.svelte'
  import Modal from './lib/components/Modal.svelte'
  import { api } from './lib/api'

  let welcome = $state(false)
  onMount(async () => {
    await app.init()
    welcome = (await app.run(() => api.getSetting('first_run_done'))) !== '1'
  })
  async function finishWelcome(then?: () => void) { await app.run(() => api.setSetting('first_run_done', '1')); welcome = false; then?.() }

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
      {:else if app.section === 'studio'}<Studio />
      {:else if app.section === 'files'}<FileManager />
      {:else if app.section === 'homebrew'}<Homebrew />
      {:else if app.section === 'backups'}<Backups />
      {:else if app.section === 'diagnostics'}<Diagnostics />
      {:else if app.section === 'settings'}<Settings />
      {:else if app.section === 'lab'}<ConsoleSim />
      {/if}
    </main>
  </div>
</div>

{#if welcome}
  <Modal title="Welcome to PS4 Studio" onclose={() => finishWelcome()}>
    <p>PS4 Studio helps you manage a PlayStation 4 from Debian and design console-style dashboards. It is an independent project and is not endorsed by Sony.</p>
    <ul>
      <li>It works <b>without a console</b>: try the built-in simulator, or design a dashboard in Dashboard Studio.</li>
      <li>A stock PS4 supports discovery and Remote Play (through chiaki-ng). File and homebrew features need a homebrew environment you already set up yourself. PS4 Studio never installs exploits or changes firmware.</li>
      <li>Nothing changes on a console without showing you exactly what will change and asking you first.</li>
    </ul>
    {#snippet footer()}
      <button onclick={() => finishWelcome(() => app.go('studio'))}>Open Dashboard Studio</button>
      <button class="primary" onclick={() => finishWelcome(() => app.go('connections'))}>Choose a console</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .shell { display: grid; grid-template-columns: 76px 1fr; height: 100%; }
  .main { display: flex; flex-direction: column; min-width: 0; }
  main { flex: 1; overflow: auto; padding: var(--space-5); }
  ul { margin: 0; padding-left: 20px; display: grid; gap: 8px; }
  .error { margin: var(--space-3) var(--space-5) 0; padding: 10px 14px; border: 1px solid var(--bad); border-radius: var(--radius-sm); color: var(--bad); background: color-mix(in srgb, var(--bad) 10%, transparent); }
</style>
