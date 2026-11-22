<script lang="ts">
  import { onMount } from 'svelte'
  import { Folder, File as FileIcon, Lock, ArrowUp, Link as LinkIcon } from 'lucide-svelte'
  import { app } from '../lib/app.svelte'
  import { api, type Entry, type RemoteOp } from '../lib/api'
  import Card from '../lib/components/Card.svelte'
  import Gated from '../lib/components/Gated.svelte'
  import Modal from '../lib/components/Modal.svelte'
  import PlanDialog from '../lib/components/PlanDialog.svelte'

  let localPath = $state('')
  let localParent = $state<string | null>(null)
  let local = $state<Entry[]>([])
  let remotePath = $state('/')
  let remote = $state<Entry[]>([])
  let selLocal = $state<string | null>(null)
  let selRemote = $state<string | null>(null)
  let user = $state('')
  let pass = $state('')
  let busy = $state(false)
  let plan = $state<RemoteOp | null>(null)
  let ask = $state<null | { kind: 'mkdir' | 'rename'; value: string }>(null)
  let unlock = $state(false)
  let ack = $state(false)
  let queue = $state<{ id: number; label: string; state: 'running' | 'done' | 'failed'; detail: string }[]>([])
  let qid = 0

  const conn = $derived(app.connection)
  const ftp = $derived(conn?.capabilities.find((c) => c.capability === 'homebrew.ftp_read'))
  const connected = $derived(ftp?.status === 'available')
  const writeOn = $derived(conn?.write_unlocked ?? false)
  const join = (d: string, n: string) => (d.endsWith('/') ? d : d + '/') + n
  const parent = (p: string) => p.replace(/\/[^/]*$/, '') || '/'
  const sel = $derived(remote.find((e) => e.name === selRemote))
  const writable = $derived(remotePath === '/data' || remotePath.startsWith('/data/'))
  const size = (n: number) => (n > 1 << 20 ? `${(n / (1 << 20)).toFixed(1)} MB` : n > 1024 ? `${(n / 1024).toFixed(1)} kB` : `${n} B`)

  async function loadLocal(p: string | null) {
    const l = await app.run(() => api.filesListLocal(p))
    if (l) { localPath = l.path; localParent = l.parent; local = l.entries; selLocal = null }
  }
  async function loadRemote(p: string) {
    const r = await app.run(() => api.filesListRemote(p))
    if (r) { remotePath = p; remote = r; selRemote = null }
  }
  onMount(() => { loadLocal(null); if (connected) loadRemote('/') })

  async function connect() {
    busy = true
    const c = await app.run(() => api.filesConnect(user || null, pass || null))
    pass = ''
    busy = false
    if (c) { app.connection = c; if (c.capabilities.find((x) => x.capability === 'homebrew.ftp_read')?.status === 'available') await loadRemote('/') }
  }
  async function disconnect() { const c = await app.run(api.filesDisconnect); if (c) { app.connection = c; remote = [] } }
  async function setLock() {
    const c = await app.run(() => api.filesSetWriteLock(true, ack))
    if (c) { app.connection = c; unlock = false; ack = false }
  }
  async function lockAgain() { const c = await app.run(() => api.filesSetWriteLock(false, false)); if (c) app.connection = c }

  async function download() {
    if (!sel || sel.kind !== 'file') return
    const item = { id: ++qid, label: `Download ${sel.name}`, state: 'running' as const, detail: '' }
    queue.unshift(item)
    const r = await app.run(() => api.filesDownload(join(remotePath, sel.name), localPath))
    const q = queue.find((x) => x.id === item.id)!
    q.state = r ? 'done' : 'failed'
    q.detail = r ?? (app.error ?? 'failed')
    if (r) await loadLocal(localPath)
  }
  function upload() { if (selLocal) plan = { kind: 'upload', local: join(localPath, selLocal), remote: join(remotePath, selLocal) } }
  function del() { if (sel) plan = { kind: 'delete', path: join(remotePath, sel.name), dir: sel.kind === 'dir' } }
  function submitAsk() {
    if (!ask) return
    const v = ask.value.trim()
    if (ask.kind === 'mkdir') plan = { kind: 'mkdir', path: join(remotePath, v) }
    else if (sel) plan = { kind: 'rename', from: join(remotePath, sel.name), to: join(remotePath, v) }
    ask = null
  }
  const reasonWrite = $derived(!connected ? (app.blocked('homebrew.ftp_read') ?? 'Connect first.') : !writeOn ? 'Writes are locked. Use “Enable changes…” first.' : !writable ? 'Changes are only allowed inside /data.' : null)
  const reasonSel = (need: 'local' | 'remote') => reasonWrite ?? (need === 'local' ? (selLocal && local.find((e) => e.name === selLocal)?.kind === 'file' ? null : 'Select a file on this computer.') : sel ? null : 'Select an item on the console.')
</script>

{#if !conn}
  <Card><p>Select a console first (Connections). The simulator has scenarios with and without a file service.</p></Card>
{:else}
  <Card title="File service">
    <div class="bar">
      <span class="dot {ftp?.status}"></span>
      <span>{connected ? 'Connected (read-only until you enable changes)' : (ftp?.reason ?? 'Not connected')}</span>
      <span class="sp"></span>
      {#if connected}
        <button onclick={disconnect}>Disconnect</button>
      {:else}
        <details><summary>Login (optional)</summary>
          <label>User<input bind:value={user} placeholder="anonymous" autocomplete="off" /></label>
          <label>Password<input type="password" bind:value={pass} autocomplete="off" /></label>
          <small>Used for this session only. Never saved.</small>
        </details>
        <button class="primary" onclick={connect} disabled={busy}>{busy ? 'Connecting…' : 'Connect to file service'}</button>
      {/if}
    </div>
    <p class="warn" role="note">FTP is not encrypted. Anyone on your network can read what you transfer. Use it only on a network you trust. This only works if your console already runs a homebrew environment with its FTP service enabled; remote play never gives file access.</p>
    {#if connected}
      <div class="bar">
        {#if writeOn}
          <span class="pill ok">Changes enabled for this session</span><button onclick={lockAgain}>Lock again</button>
        {:else}
          <span class="pill"><Lock size={14} /> Read-only</span><button onclick={() => (unlock = true)}>Enable changes…</button>
        {/if}
      </div>
    {/if}
  </Card>

  <div class="panes">
    <Card title="This computer">
      <div class="path"><button aria-label="Parent folder" disabled={!localParent} onclick={() => localParent && loadLocal(localParent)}><ArrowUp size={14} /></button><code>{localPath}</code></div>
      <ul class="list" aria-label="Local files">
        {#each local as e}
          <li><button class:on={selLocal === e.name} onclick={() => (selLocal = e.name)} ondblclick={() => e.kind === 'dir' && loadLocal(join(localPath, e.name))}>
            {#if e.kind === 'dir'}<Folder size={14} />{:else}<FileIcon size={14} />{/if}<span>{e.name}</span><small>{e.kind === 'file' ? size(e.size) : ''}</small></button></li>
        {/each}
      </ul>
    </Card>

    <Card title="Console">
      <div class="path"><button aria-label="Parent folder" disabled={!connected || remotePath === '/'} onclick={() => loadRemote(parent(remotePath))}><ArrowUp size={14} /></button><code>{remotePath}</code>
        {#if connected && !writable}<span class="pill"><Lock size={12} /> read-only here</span>{/if}</div>
      {#if !connected}
        <p class="hint">{ftp?.reason ?? 'Connect to the file service to browse.'}</p>
      {:else}
        <ul class="list" aria-label="Console files">
          {#each remote as e}
            <li><button class:on={selRemote === e.name} onclick={() => (selRemote = e.name)} ondblclick={() => e.kind === 'dir' && loadRemote(join(remotePath, e.name))}>
              {#if e.kind === 'dir'}<Folder size={14} />{:else if e.kind === 'link'}<LinkIcon size={14} />{:else}<FileIcon size={14} />{/if}<span>{e.name}</span><small>{e.kind === 'file' ? size(e.size) : ''}</small></button></li>
          {:else}<li class="hint">Empty folder.</li>{/each}
        </ul>
      {/if}
    </Card>
  </div>

  <Card title="Actions">
    <div class="acts">
      <Gated reason={!connected ? 'Connect to the file service first.' : sel?.kind === 'file' ? null : 'Select a file on the console.'} onclick={download}>Download →</Gated>
      <Gated reason={reasonSel('local')} onclick={upload}>← Upload…</Gated>
      <Gated reason={reasonWrite} onclick={() => (ask = { kind: 'mkdir', value: '' })}>New folder…</Gated>
      <Gated reason={reasonSel('remote')} onclick={() => sel && (ask = { kind: 'rename', value: sel.name })}>Rename…</Gated>
      <Gated reason={reasonSel('remote')} onclick={del}>Delete…</Gated>
    </div>
    <p class="hint">Every change shows exactly what will happen and needs your confirmation. Existing files are copied to Backups before being replaced or deleted. Pause is not supported by the file service; transfers can be cancelled.</p>
    {#if queue.length}
      <h3>Transfers</h3>
      <ul class="q">{#each queue as q}<li><b class={q.state}>{q.state}</b> {q.label} <small>{q.detail}</small></li>{/each}</ul>
    {/if}
  </Card>
{/if}

{#if unlock}
  <Modal title="Enable changes on the console?" onclose={() => (unlock = false)}>
    <p>While enabled you can upload, rename, create and delete items in <code>/data</code> on the console. System locations stay read-only. You will still confirm every change.</p>
    <label class="check"><input type="checkbox" bind:checked={ack} /> I understand FTP is not encrypted and that mistakes can affect my console.</label>
    {#snippet footer()}<button onclick={() => (unlock = false)}>Cancel</button><button class="primary" disabled={!ack} onclick={setLock}>Enable changes</button>{/snippet}
  </Modal>
{/if}

{#if ask}
  <Modal title={ask.kind === 'mkdir' ? 'New folder' : 'Rename'} onclose={() => (ask = null)}>
    <label>Name<input bind:value={ask.value} maxlength="255" /></label>
    <p class="hint">In {remotePath}. You will review the change before it happens.</p>
    {#snippet footer()}<button onclick={() => (ask = null)}>Cancel</button><button class="primary" disabled={!ask?.value.trim()} onclick={submitAsk}>Review change</button>{/snippet}
  </Modal>
{/if}

{#if plan}
  <PlanDialog op={plan} onclose={() => (plan = null)} ondone={() => { loadRemote(remotePath); }} />
{/if}

<style>
  :global(.panes) { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-4); margin: var(--space-4) 0; }
  @media (max-width: 1000px) { :global(.panes) { grid-template-columns: 1fr; } }
  .bar { display: flex; gap: var(--space-3); align-items: center; flex-wrap: wrap; margin-bottom: var(--space-3); } .sp { flex: 1; }
  .dot { width: 9px; height: 9px; border-radius: 50%; background: var(--text-muted); } .dot.available { background: var(--ok); } .dot.unavailable { background: var(--bad); } .dot.unknown { background: var(--warn); }
  .warn { font-size: 12px; color: var(--warn); border: 1px solid var(--warn); border-radius: var(--radius-sm); padding: 8px 12px; }
  .pill { display: inline-flex; gap: 6px; align-items: center; font-size: 12px; padding: 3px 10px; border: 1px solid var(--border); border-radius: 999px; } .pill.ok { color: var(--ok); border-color: var(--ok); }
  details { font-size: 12px; } details label { margin-top: 6px; }
  .path { display: flex; gap: var(--space-2); align-items: center; margin-bottom: var(--space-2); } .path code { font-family: var(--mono); font-size: 12px; overflow-wrap: anywhere; } .path button { padding: 3px 8px; }
  .list { list-style: none; margin: 0; padding: 0; max-height: 300px; overflow: auto; border: 1px solid var(--border); border-radius: var(--radius-sm); }
  .list li button { width: 100%; border: 0; border-radius: 0; background: transparent; justify-content: flex-start; padding: 5px 10px; }
  .list li button span { flex: 1; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .list li button.on { background: var(--surface-2); box-shadow: inset 3px 0 var(--accent); }
  small, .hint { color: var(--text-muted); font-size: 12px; }
  .acts { display: flex; gap: var(--space-3); flex-wrap: wrap; align-items: flex-start; }
  .q { list-style: none; padding: 0; margin: 0; display: grid; gap: 4px; font-size: 12px; } .q b.done { color: var(--ok); } .q b.failed { color: var(--bad); }
  label.check { display: flex; gap: 8px; align-items: flex-start; color: var(--text); font-size: 13px; }
</style>
