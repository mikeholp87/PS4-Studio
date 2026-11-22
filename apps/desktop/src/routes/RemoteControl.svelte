<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type RemotePlayStatus } from '../lib/api'
  import Card from '../lib/components/Card.svelte'
  import Gated from '../lib/components/Gated.svelte'

  let st = $state<RemotePlayStatus | null>(null)
  let nickname = $state('')
  let fullscreen = $state(true)
  let timer: ReturnType<typeof setInterval> | undefined

  async function refresh() {
    st = (await app.run(api.remotePlayStatus)) ?? null
    if (st && !nickname && st.nickname) nickname = st.nickname
    // Poll only while a session is running, to notice when the window is closed.
    if (st?.running && !timer) timer = setInterval(refresh, 3000)
    if (!st?.running && timer) { clearInterval(timer); timer = undefined }
  }
  onMount(refresh)
  onDestroy(() => timer && clearInterval(timer))

  async function start() { if (await app.run(() => api.startRemotePlay(nickname.trim(), fullscreen)) !== undefined) await refresh() }
  async function stop() { await app.run(api.stopRemotePlay); await refresh() }

  const reason = $derived.by(() => {
    if (!st?.installed) return 'chiaki-ng is not installed.'
    if (!nickname.trim()) return 'Enter the nickname you gave this console inside chiaki-ng.'
    if (app.connection?.simulated) return 'Simulated console: no real session can be started.'
    return app.blocked('console.remote_play')
  })
</script>

<div class="cols">
  <Card title="Start a session">
    <p class="hint">PS4 Studio starts the installed <b>chiaki-ng</b> client for you. It never sees or stores your PlayStation credentials: you register the console once inside chiaki-ng.</p>
    <dl>
      <dt>chiaki-ng</dt><dd>{#if !st}…{:else if st.installed}<span class="ok">Found</span> <code>{st.binary}</code>{:else}<span class="bad">Not found</span> — install from Flathub (<code>io.github.streetpea.Chiaki4deck</code>).{/if}</dd>
      <dt>Console</dt><dd>{app.connection ? app.connection.profile.name : 'None selected'}</dd>
    </dl>
    <label>Console nickname in chiaki-ng
      <input bind:value={nickname} maxlength="32" placeholder="e.g. Living room" />
    </label>
    <label class="check"><input type="checkbox" bind:checked={fullscreen} /> Start full screen</label>
    <div class="row">
      {#if st?.running}
        <button class="danger" onclick={stop}>Stop session</button><span class="hint">Session running.</span>
      {:else}
        <Gated primary {reason} onclick={start}>Start Remote Play</Gated>
      {/if}
    </div>
  </Card>

  <Card title="Set up Remote Play">
    <ol>
      <li>On the PS4: <b>Settings → Remote Play Connection Settings</b>, tick <b>Enable Remote Play</b>.</li>
      <li>To wake the console from Rest Mode: <b>Settings → Power Saving Settings → Set Features Available in Rest Mode</b>, enable <b>Stay Connected to the Internet</b> and <b>Enable Turning On PS4 from Network</b>.</li>
      <li>Open chiaki-ng, choose <b>Add manually</b> or pick the discovered console, and register using the 8-digit code from <b>Remote Play Connection Settings → Add Device</b>.</li>
      <li>Come back here, enter the nickname you used, and press Start.</li>
    </ol>
    <p class="hint">Remote Play streams video, audio and controller input only. It does not give access to files or the system software.</p>
  </Card>

  <Card title="Controllers">
    <p>A DualShock 4 works over USB or Bluetooth and is handled by chiaki-ng. If it is not detected, check that your user can read <code>/dev/input/event*</code> (the <code>input</code> group on some systems) and that no other app has grabbed the pad.</p>
    <p class="hint">PS4 Studio does not simulate button presses; all input goes through chiaki-ng.</p>
  </Card>
</div>

<style>
  .cols { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  :global(section.card > label) { display: grid; gap: var(--space-2); margin: var(--space-3) 0; }
  label.check { display: flex; align-items: center; gap: 8px; }
  .row { display: flex; gap: var(--space-3); align-items: flex-start; }
  dl { display: grid; grid-template-columns: 90px 1fr; gap: 6px var(--space-3); margin: var(--space-3) 0; } dt { color: var(--text-muted); } dd { margin: 0; overflow-wrap: anywhere; }
  .hint { color: var(--text-muted); font-size: 12px; margin-top: var(--space-2); }
  ol { margin: 0 0 var(--space-3); padding-left: 20px; display: grid; gap: var(--space-2); }
  .ok { color: var(--ok); } .bad { color: var(--bad); } code { font-family: var(--mono); font-size: 12px; }
  p + p { margin-top: var(--space-2); }
</style>
