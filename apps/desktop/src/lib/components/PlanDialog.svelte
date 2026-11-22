<script lang="ts">
  // INSPECT → PLAN → CONFIRM → APPLY → VERIFY → RECOVER for one remote change.
  import { onMount, onDestroy } from 'svelte'
  import { api, errText, listen, type RemoteOp, type PlanView, type ApplyResult } from '../api'
  import Modal from './Modal.svelte'

  let { op, ondone, onclose }: { op: RemoteOp; ondone: () => void; onclose: () => void } = $props()
  let pv = $state<PlanView | null>(null)
  let problem = $state<string | null>(null)
  let busy = $state(false)
  let bytes = $state(0)
  let result = $state<ApplyResult | null>(null)
  let unlisten: (() => void) | undefined

  onMount(async () => {
    try { pv = await api.filesPlan(op) } catch (e) { problem = errText(e) }
    try { unlisten = await listen<{ bytes: number }>('files-progress', (e) => (bytes = e.payload.bytes)) } catch { /* no events outside Tauri */ }
  })
  onDestroy(() => unlisten?.())

  async function apply() {
    if (!pv) return
    busy = true
    try { result = await api.filesApply(pv.plan_id); ondone() } catch (e) { problem = errText(e) }
    busy = false
  }
  const title = $derived(result ? 'Done' : problem && !pv ? 'Cannot make this change' : 'Review before changing the console')
</script>

<Modal {title} onclose={() => { if (!busy) onclose() }}>
  {#if result}
    <ul>{#each result.outcome.lines as l}<li>{l}</li>{/each}</ul>
    {#if result.backup_id}<p class="hint">Backup saved: <code>{result.backup_id}</code> (see Backups).</p>{/if}
    <p class="hint"><b>If something looks wrong:</b> {result.outcome.recovery}</p>
  {:else if pv}
    <div>
      <h3>What will change</h3>
      <ul>{#each pv.plan.changes as c}<li>{c}</li>{/each}</ul>
    </div>
    {#if pv.plan.warnings.length}
      <div class="warn"><h3>Warnings</h3><ul>{#each pv.plan.warnings as w}<li>{w}</li>{/each}</ul></div>
    {/if}
    <p class="hint"><b>Recovery:</b> {pv.plan.recovery} Changes on the console are not rolled back automatically.</p>
    {#if busy}<p role="status">Working… {bytes ? `${bytes.toLocaleString()} bytes` : ''}</p>{/if}
  {:else if !problem}
    <p>Checking the console…</p>
  {/if}
  {#if problem}<p class="bad" role="alert">{problem}</p>{/if}

  {#snippet footer()}
    {#if result}
      <button class="primary" onclick={onclose}>Close</button>
    {:else}
      {#if busy}<button onclick={() => api.filesCancel()}>Cancel transfer</button>{/if}
      <button onclick={onclose} disabled={busy}>{pv ? 'Do not change anything' : 'Close'}</button>
      {#if pv}<button class="primary" onclick={apply} disabled={busy}>Confirm and apply</button>{/if}
    {/if}
  {/snippet}
</Modal>

<style>
  ul { margin: 0; padding-left: 20px; display: grid; gap: 4px; overflow-wrap: anywhere; }
  .warn { border: 1px solid var(--warn); border-radius: var(--radius-sm); padding: var(--space-3); background: color-mix(in srgb, var(--warn) 8%, transparent); }
  .bad { color: var(--bad); } .hint { color: var(--text-muted); font-size: 12px; } code { font-family: var(--mono); }
  h3 { margin-bottom: var(--space-2); }
</style>
