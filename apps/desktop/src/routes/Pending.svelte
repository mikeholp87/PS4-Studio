<script lang="ts">
  // Sections whose feature code lands in later phases. They state what is coming and what gates it —
  // no dead buttons.
  import { app, type SectionId } from '../lib/app.svelte'
  import Card from '../lib/components/Card.svelte'
  let { id }: { id: SectionId } = $props()
  const info: Partial<Record<SectionId, { phase: string; what: string; gate: string | null; extra?: string }>> = {
    studio: { phase: 'Phase 4', gate: null, what: 'Design PlayStation-style dashboard themes (Minimal, Aurora, Grid, Classic) with a 1080p TV preview, entirely offline. Themes do not replace the official PS4 home screen.' },
    homebrew: { phase: 'Phase 5', gate: 'homebrew.ftp_read', what: 'List and validate homebrew packages for consoles that already run a homebrew environment. Nothing is installed without your explicit confirmation.' },
    files: { phase: 'Phase 5', gate: 'homebrew.ftp_read', what: 'Two-pane file manager: your computer on the left, the console file service on the right. Read-only by default; writes need an explicit unlock. FTP is unencrypted.' },
    backups: { phase: 'Phase 5', gate: null, what: 'Back up PS4 Studio settings, profiles and themes with checksums and a manifest. Saved games and system data on a stock PS4 can only be backed up with Sony’s built-in tools (Settings → System → Back Up and Restore).' },
  }
  const p = $derived(info[id])
  const blocked = $derived(p?.gate ? app.blocked(p.gate) : null)
</script>
{#if p}
  <Card title="Coming in {p.phase}">
    <p>{p.what}</p>
    {#if p.gate}
      <p class="gate"><code>{p.gate}</code> — {blocked ?? 'Available for the selected console.'}</p>
    {/if}
  </Card>
{/if}
<style>
  p { max-width: 640px; } .gate { margin-top: var(--space-3); color: var(--text-muted); font-size: 12px; }
  code { font-family: var(--mono); }
</style>
