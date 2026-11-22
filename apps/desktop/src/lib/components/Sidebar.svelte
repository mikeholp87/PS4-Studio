<script lang="ts">
  import { LayoutDashboard, Plug, Gamepad2, Palette, Package, FolderOpen, Archive, Stethoscope, Settings } from 'lucide-svelte'
  import { app, sections, type SectionId } from '../app.svelte'
  const icons: Record<SectionId, typeof Settings> = {
    overview: LayoutDashboard, connections: Plug, remote: Gamepad2, studio: Palette,
    homebrew: Package, files: FolderOpen, backups: Archive, diagnostics: Stethoscope, settings: Settings,
  }
</script>

<nav aria-label="Sections">
  <div class="logo" aria-hidden="true">PS4<br />Studio</div>
  <ul>
    {#each sections as s, i}
      {@const Icon = icons[s.id]}
      <li>
        <button class:active={app.section === s.id} aria-current={app.section === s.id ? 'page' : undefined}
          title="{s.label} (Alt+{i + 1})" onclick={() => app.go(s.id)}>
          <Icon size={20} aria-hidden="true" />
          <span>{s.label}</span>
        </button>
      </li>
    {/each}
  </ul>
</nav>

<style>
  nav { background: var(--surface); border-right: 1px solid var(--border); display: flex; flex-direction: column; align-items: center; padding: var(--space-3) 0; }
  .logo { font-weight: 700; font-size: 11px; line-height: 1.1; text-align: center; margin-bottom: var(--space-4);
    background: linear-gradient(135deg, var(--accent), var(--accent-2)); -webkit-background-clip: text; background-clip: text; color: transparent; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; width: 100%; }
  button { flex-direction: column; gap: 3px; width: 100%; border: 0; border-radius: 0; background: transparent; padding: 9px 2px; font-size: 10px; color: var(--text-muted); border-left: 3px solid transparent; }
  button:hover:not(:disabled) { color: var(--text); background: var(--surface-2); }
  button.active { color: var(--text); border-left-color: var(--accent); background: var(--surface-2); }
  span { text-align: center; line-height: 1.15; }
</style>
