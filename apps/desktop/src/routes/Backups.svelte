<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type Manifest, type AuditEntry } from '../lib/api'
  import Card from '../lib/components/Card.svelte'
  import Modal from '../lib/components/Modal.svelte'

  let list = $state<Manifest[]>([])
  let label = $state('')
  let open = $state<string | null>(null)
  let note = $state('')
  let confirm = $state<null | { kind: 'delete' | 'restore'; id: string; index?: number; text: string }>(null)
  let ops = $state<AuditEntry[]>([])
  const when = (ts: number) => new Date(ts * 1000).toLocaleString()
  const catName = { settings: 'Settings', profiles: 'Saved consoles', themes: 'Theme', remote_file: 'Console file', user_file: 'File' }
  const restorable = (c: string) => c === 'settings' || c === 'profiles' || c === 'themes'

  async function load() {
    list = (await app.run(api.backupList)) ?? []
    ops = ((await app.run(api.listAudit)) ?? []).filter((a) => /^(backup_|files_apply|unlock_writes|lock_writes)/.test(a.action)).slice(0, 30)
  }
  onMount(load)
  async function create() {
    const m = await app.run(() => api.backupCreate(label))
    if (m) { note = `Backup created with ${m.items.length} items.`; label = ''; await load() }
  }
  async function verify(id: string) {
    const r = await app.run(() => api.backupVerify(id))
    if (r) note = r.ok ? 'Verified: every item matches its checksum.' : `DAMAGED: ${r.problems.join('; ')}`
    await load()
  }
  async function saveCopy(id: string, i: number) {
    const p = await app.run(() => api.backupSaveItemAs(id, i))
    if (p) note = `Saved a verified copy to ${p}`
  }
  async function doConfirm() {
    const c = confirm
    confirm = null
    if (!c) return
    if (c.kind === 'delete') { if ((await app.run(() => api.backupDelete(c.id))) !== undefined) { note = 'Backup deleted.'; open = null } }
    else { const m = await app.run(() => api.backupRestoreItem(c.id, c.index ?? 0)); if (m) note = m }
    await load()
  }
</script>

<div class="cols">
  <Card title="Create a backup">
    <p class="hint">Saves PS4 Studio settings, saved console profiles (no passwords: none are stored) and your themes, with a checksum for each item. Files from a console are added automatically, as a copy, before the app replaces or deletes them.</p>
    <label>Label (optional)<input bind:value={label} maxlength="60" placeholder="Before trying a new theme" /></label>
    <button class="primary" onclick={create}>Create backup</button>
    {#if note}<p class="note" role="status">{note}</p>{/if}
  </Card>

  <Card title="Your PS4's own data">
    <p>PS4 Studio cannot back up saved games or system software. Remote Play does not give that access. Use Sony's tools on the console:</p>
    <ul>
      <li><b>Settings → System → Back Up and Restore</b> to back up to USB storage.</li>
      <li><b>Settings → Application Saved Data Management</b> to copy saves to USB, or upload them to online storage (PlayStation Plus).</li>
    </ul>
    <p class="hint">Menu names can differ slightly between system software versions. Check Sony's support pages if a menu is missing.</p>
  </Card>

  <Card title="Saved backups">
    {#if list.length === 0}<p class="hint">No backups yet.</p>{:else}
      <table>
        <thead><tr><th>When</th><th>Label</th><th>Items</th><th></th></tr></thead>
        <tbody>
          {#each list as m}
            <tr>
              <td>{when(m.created)}</td><td>{m.label}</td><td>{m.items.length}</td>
              <td class="end">
                <button onclick={() => verify(m.id)}>Verify</button>
                <button onclick={() => (open = open === m.id ? null : m.id)}>{open === m.id ? 'Hide' : 'Details'}</button>
                <button class="danger" onclick={() => (confirm = { kind: 'delete', id: m.id, text: `Delete the backup “${m.label}”? This cannot be undone.` })}>Delete</button>
              </td>
            </tr>
            {#if open === m.id}
              <tr><td colspan="4">
                <table class="inner">
                  <thead><tr><th>Item</th><th>Type</th><th>Size</th><th>SHA-256</th><th></th></tr></thead>
                  <tbody>
                    {#each m.items as it, i}
                      <tr><td>{it.original}</td><td>{catName[it.category]}</td><td>{it.size} B</td><td><code>{it.sha256.slice(0, 12)}…</code></td>
                        <td class="end">
                          {#if restorable(it.category)}<button onclick={() => (confirm = { kind: 'restore', id: m.id, index: i, text: `Restore “${it.original}”? It replaces the current version (the current one is backed up first).` })}>Restore</button>{/if}
                          <button onclick={() => saveCopy(m.id, i)}>Save a copy…</button>
                        </td></tr>
                    {/each}
                  </tbody>
                </table>
              </td></tr>
            {/if}
          {/each}
        </tbody>
      </table>
    {/if}
  </Card>

  <Card title="Operation history">
    <p class="hint">Changes made through PS4 Studio, each with its recovery note. Console changes are not rolled back automatically; use the recovery note or the saved backup.</p>
    {#if ops.length === 0}<p class="hint">Nothing recorded yet.</p>{:else}
      <table><thead><tr><th>Time</th><th>Action</th><th>Result</th><th>Details</th></tr></thead>
        <tbody>{#each ops as a}<tr><td>{when(a.ts)}</td><td>{a.action}</td><td>{a.outcome}</td><td>{a.detail}</td></tr>{/each}</tbody></table>
    {/if}
  </Card>
</div>

{#if confirm}
  <Modal title={confirm.kind === 'delete' ? 'Delete backup?' : 'Restore item?'} onclose={() => (confirm = null)}>
    <p>{confirm.text}</p>
    {#snippet footer()}<button onclick={() => (confirm = null)}>Cancel</button><button class="primary" onclick={doConfirm}>{confirm?.kind === 'delete' ? 'Delete' : 'Restore'}</button>{/snippet}
  </Modal>
{/if}

<style>
  .cols { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  :global(.cols > :nth-child(n + 3)) { grid-column: 1 / -1; }
  :global(section.card > label) { display: grid; gap: 4px; margin: var(--space-3) 0; }
  .hint { color: var(--text-muted); font-size: 12px; } .note { margin-top: var(--space-3); color: var(--accent); font-size: 13px; }
  .end { display: flex; gap: 6px; justify-content: flex-end; flex-wrap: wrap; } ul { margin: var(--space-2) 0; padding-left: 20px; display: grid; gap: 6px; }
  .inner { background: var(--bg); } code { font-family: var(--mono); font-size: 12px; }
</style>
