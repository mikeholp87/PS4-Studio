<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api } from '../lib/api'
  import { fromTemplate, newTheme, parseThemeText, validateTheme, TEMPLATE_IDS, type TemplateId, type Theme } from '@ps4studio/theme-engine'
  import DashboardPreview from '../lib/components/DashboardPreview.svelte'
  import ThemeEditor from '../lib/components/ThemeEditor.svelte'

  let saved = $state<Theme[]>([])
  let draft = $state<Theme>(fromTemplate('aurora'))
  let isSaved = $state(false)
  let info = $state('')
  let newTpl = $state<TemplateId>('aurora')

  const snapshot = $derived($state.snapshot(draft) as Theme)
  const result = $derived(validateTheme(snapshot))
  const errors = $derived(result.ok ? [] : result.errors)
  const dirty = $derived(!isSaved || JSON.stringify(saved.find((t) => t.id === draft.id)) !== JSON.stringify(snapshot))

  async function load() {
    const rows = (await app.run(api.listThemes)) ?? []
    saved = rows.flatMap((r) => { const p = parseThemeText(r.json); return p.ok ? [p.theme] : [] })
  }
  onMount(load)

  function open(t: Theme) { draft = structuredClone(t); isSaved = true; info = '' }
  function create() {
    draft = newTheme(`${newTpl[0].toUpperCase()}${newTpl.slice(1)} copy`, newTpl, saved.map((t) => t.id))
    isSaved = false; info = ''
  }
  function duplicate() {
    const t = newTheme(`${draft.name} copy`, draft.template, saved.map((x) => x.id))
    draft = { ...structuredClone(snapshot), id: t.id, name: t.name }
    isSaved = false; info = 'Duplicated. Save to keep it.'
  }
  async function save() {
    if (!result.ok) return
    const r = await app.run(() => api.saveTheme(JSON.stringify(snapshot)))
    if (r) { await load(); isSaved = true; info = 'Saved.' }
  }
  async function remove() {
    if (!isSaved || !confirm(`Delete “${draft.name}”? This cannot be undone.`)) return
    if (await app.run(() => api.deleteTheme(draft.id))) { await load(); create(); info = 'Deleted.' }
  }
  function reset() {
    const t = fromTemplate(draft.template)
    draft = { ...t, id: draft.id, name: draft.name }
    info = 'Reset to the template defaults (not saved yet).'
  }
  function revert() { const s = saved.find((t) => t.id === draft.id); if (s) open(s) }
  async function exportIt() {
    const p = await app.run(() => api.exportTheme(draft.id))
    if (p) info = `Exported to ${p}`
  }
  async function importIt() {
    const t = await app.run(api.importTheme)
    if (t) { await load(); const p = parseThemeText(t.json); if (p.ok) open(p.theme); info = 'Imported and validated.' }
  }
</script>

<div class="studio">
  <aside>
    <h3>Your themes</h3>
    <ul>
      {#each saved as t (t.id)}
        <li><button class:active={isSaved && draft.id === t.id} onclick={() => open(t)}>{t.name}<small>{t.template}</small></button></li>
      {:else}
        <li class="hint">None saved yet.</li>
      {/each}
    </ul>
    <div class="new">
      <select bind:value={newTpl} aria-label="Template for new theme">{#each TEMPLATE_IDS as t}<option value={t}>{t}</option>{/each}</select>
      <button onclick={create}>New from template</button>
    </div>
    <button onclick={importIt}>Import theme…</button>
    <p class="hint">Themes are plain data files. They cannot run code, and they do not replace the official PS4 home screen.</p>
  </aside>

  <section class="center">
    <div class="toolbar">
      <b>{draft.name}</b>{#if dirty}<span class="dirty">unsaved</span>{/if}
      <span class="sp"></span>
      <button class="primary" onclick={save} disabled={!result.ok || !dirty}>Save</button>
      <button onclick={revert} disabled={!isSaved || !dirty}>Revert</button>
      <button onclick={reset}>Reset to template</button>
      <button onclick={duplicate}>Duplicate</button>
      <button onclick={exportIt} disabled={!isSaved || dirty} title={!isSaved || dirty ? 'Save the theme first.' : undefined}>Export…</button>
      <button class="danger" onclick={remove} disabled={!isSaved}>Delete</button>
    </div>
    {#if errors.length}<div class="err" role="alert">Cannot save: {errors.join('; ')}</div>{/if}
    {#if info}<p class="hint" role="status">{info}</p>{/if}
    <DashboardPreview theme={snapshot} />
  </section>

  <section class="right"><ThemeEditor bind:theme={draft} /></section>
</div>

<style>
  .studio { display: grid; grid-template-columns: 190px minmax(0, 1fr) 330px; gap: var(--space-4); align-items: start; }
  @media (max-width: 1180px) { .studio { grid-template-columns: 1fr; } }
  aside { display: grid; gap: var(--space-3); }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; }
  li button { width: 100%; justify-content: space-between; } li button.active { border-color: var(--accent); }
  small { color: var(--text-muted); }
  .new { display: grid; gap: 6px; }
  .center { display: grid; gap: var(--space-3); min-width: 0; }
  .toolbar { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; } .sp { flex: 1; }
  .dirty { color: var(--warn); font-size: 12px; margin-left: 8px; }
  .err { padding: 8px 12px; border: 1px solid var(--bad); color: var(--bad); border-radius: var(--radius-sm); font-size: 12px; }
  .hint { color: var(--text-muted); font-size: 12px; }
</style>
