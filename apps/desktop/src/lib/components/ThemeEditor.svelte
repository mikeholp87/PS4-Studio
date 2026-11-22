<script lang="ts">
  import { ICONS, TEMPLATE_IDS, addTile, moveTile, removeTile, type Theme } from '@ps4studio/theme-engine'
  let { theme = $bindable() }: { theme: Theme } = $props()
  let newLabel = $state('')
  const cap = (s: string) => s[0].toUpperCase() + s.slice(1)
</script>

<div class="ed">
  <details open>
    <summary>General</summary>
    <label>Name<input bind:value={theme.name} maxlength="48" /></label>
    <label>Layout template
      <select bind:value={theme.template}>{#each TEMPLATE_IDS as t}<option value={t}>{cap(t)}</option>{/each}</select>
    </label>
    <label>Grid columns (Grid template)<input type="number" min="2" max="8" bind:value={theme.layout.columns} /></label>
    <label>Tile size<select bind:value={theme.layout.tileSize}><option value="small">Small</option><option value="medium">Medium</option><option value="large">Large</option></select></label>
    <label>Tile shape<select bind:value={theme.layout.tileShape}><option value="square">Square</option><option value="rounded">Rounded</option><option value="circle">Circle</option></select></label>
  </details>

  <details>
    <summary>Background</summary>
    <label>Type<select bind:value={theme.background.kind}><option value="solid">Solid</option><option value="gradient">Gradient</option></select></label>
    <label>{theme.background.kind === 'solid' ? 'Colour' : 'From'}<input type="color" bind:value={theme.background.from} /></label>
    {#if theme.background.kind === 'gradient'}
      <label>To<input type="color" bind:value={theme.background.to} /></label>
      <label>Angle {theme.background.angle}°<input type="range" min="0" max="360" step="5" bind:value={theme.background.angle} /></label>
    {/if}
    <p class="hint">Image backgrounds are not supported in this version.</p>
  </details>

  <details>
    <summary>Colours and font</summary>
    <label>Accent<input type="color" bind:value={theme.colors.accent} /></label>
    <label>Text<input type="color" bind:value={theme.colors.text} /></label>
    <label>Tile surface<input type="color" bind:value={theme.colors.surface} /></label>
    <label>Font<select bind:value={theme.font.family}><option value="system">System</option><option value="serif">Serif</option><option value="mono">Monospace</option><option value="rounded">Rounded</option></select></label>
    <label>Text size {theme.font.scale.toFixed(2)}×<input type="range" min="0.8" max="1.4" step="0.05" bind:value={theme.font.scale} /></label>
  </details>

  <details>
    <summary>Widgets and navigation</summary>
    <label class="check"><input type="checkbox" bind:checked={theme.widgets.clock.enabled} /> Clock</label>
    <label class="check"><input type="checkbox" bind:checked={theme.widgets.clock.hour24} /> 24-hour time</label>
    <label>Clock position<select bind:value={theme.widgets.clock.position}><option value="top-right">Top right</option><option value="top-left">Top left</option></select></label>
    <label class="check"><input type="checkbox" bind:checked={theme.widgets.status.enabled} /> Status icons (preview only)</label>
    <label class="check"><input type="checkbox" bind:checked={theme.navigation.wrapFocus} /> Wrap focus at the edges</label>
    <label class="check"><input type="checkbox" bind:checked={theme.navigation.showLabels} /> Show tile labels</label>
  </details>

  <details open>
    <summary>Tiles ({theme.tiles.length}/40)</summary>
    <ul class="tiles">
      {#each theme.tiles as t (t.id)}
        <li>
          <input class="lab" bind:value={t.label} maxlength="32" aria-label="Label for {t.id}" />
          <select bind:value={t.icon} aria-label="Icon for {t.label}">{#each ICONS as i}<option value={i}>{i}</option>{/each}</select>
          <input type="color" bind:value={t.color} aria-label="Colour for {t.label}" />
          <button onclick={() => (theme = moveTile(theme, t.id, -1))} aria-label="Move {t.label} earlier">↑</button>
          <button onclick={() => (theme = moveTile(theme, t.id, 1))} aria-label="Move {t.label} later">↓</button>
          <button class="danger" onclick={() => (theme = removeTile(theme, t.id))} aria-label="Remove {t.label}">✕</button>
        </li>
      {/each}
    </ul>
    <form onsubmit={(e) => { e.preventDefault(); if (newLabel.trim()) { theme = addTile(theme, newLabel); newLabel = '' } }}>
      <input bind:value={newLabel} maxlength="32" placeholder="New shortcut label" aria-label="New shortcut label" />
      <button disabled={theme.tiles.length >= 40}>Add shortcut</button>
    </form>
    <p class="hint">Shortcuts are labels in the preview and in exported themes. They do not launch anything on a PS4.</p>
  </details>
</div>

<style>
  .ed { display: grid; gap: var(--space-2); }
  details { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: var(--space-3); }
  summary { cursor: pointer; font-weight: 600; }
  details > label { margin-top: var(--space-3); } label.check { display: flex; gap: 8px; align-items: center; color: var(--text); font-size: 13px; }
  input[type='color'] { padding: 0; height: 30px; width: 60px; } input[type='range'] { padding: 0; }
  .tiles { list-style: none; margin: var(--space-3) 0; padding: 0; display: grid; gap: 6px; }
  .tiles li { display: grid; grid-template-columns: 1fr 86px 34px auto auto auto; gap: 4px; align-items: center; }
  .tiles li button { padding: 4px 8px; } .tiles input[type='color'] { width: 34px; }
  form { display: flex; gap: 6px; } form input { flex: 1; }
  .hint { color: var(--text-muted); font-size: 12px; margin-top: var(--space-2); }
</style>
