<script lang="ts">
  import { app } from '../app.svelte'
  import { api, errText } from '../api'
  import Modal from './Modal.svelte'

  let { themeId, onclose }: { themeId: string; onclose: () => void } = $props()
  let code = $state('')
  let busy = $state(false)
  let msg = $state<string | null>(null)
  let problem = $state<string | null>(null)

  const cap = $derived(app.cap('companion.theme_management'))
  const connected = $derived(app.cap('companion.connected')?.status === 'available')
  const sim = $derived(app.connection?.simulated ?? false)

  async function connect() {
    busy = true; problem = null; msg = null
    try { app.connection = await api.companionConnect(code.trim()) } catch (e) { problem = errText(e) }
    busy = false
  }
  async function send() {
    busy = true; problem = null; msg = null
    try { await api.companionSendTheme(themeId); msg = 'Theme sent. The companion stored it.' } catch (e) { problem = errText(e) }
    busy = false
  }
  async function leave() { try { app.connection = await api.companionDisconnect() } catch { /* ignore */ } onclose() }
</script>

<Modal title="Send theme to the PS4 Studio companion" onclose={onclose}>
  <p>The companion is a separate homebrew app that you run on a console with an existing homebrew environment. Themes you send are only used inside that app. They do <b>not</b> change the official PS4 home screen.</p>
  <p class="warn" role="note">This link is authenticated but <b>not encrypted</b>. Pair only on a network you trust.</p>

  {#if !connected}
    <label>Pairing code shown on your TV
      <input bind:value={code} inputmode="numeric" maxlength="8" placeholder="8 digits" autocomplete="off" />
    </label>
    {#if sim}<p class="hint">Simulator: the simulated “TV” shows the code <code>24681357</code>.</p>{/if}
    {#if cap?.reason && cap.status !== 'available'}<p class="hint">{cap.reason}</p>{/if}
  {:else}
    <p class="ok">Connected and paired. {cap?.evidence}</p>
  {/if}
  {#if problem}<p class="bad" role="alert">{problem}</p>{/if}
  {#if msg}<p class="ok" role="status">{msg}</p>{/if}

  {#snippet footer()}
    {#if connected}<button onclick={leave}>Disconnect and close</button>
      <button class="primary" onclick={send} disabled={busy || cap?.status !== 'available'}>Send theme</button>
    {:else}<button onclick={onclose}>Close</button>
      <button class="primary" onclick={connect} disabled={busy || code.trim().length !== 8}>{busy ? 'Pairing…' : 'Pair'}</button>
    {/if}
  {/snippet}
</Modal>

<style>
  label { display: grid; gap: 4px; } .warn { color: var(--warn); border: 1px solid var(--warn); border-radius: var(--radius-sm); padding: 8px 12px; font-size: 12px; }
  .hint { color: var(--text-muted); font-size: 12px; } .ok { color: var(--ok); } .bad { color: var(--bad); } code { font-family: var(--mono); }
</style>
