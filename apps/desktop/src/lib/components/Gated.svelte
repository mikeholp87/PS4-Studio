<script lang="ts">
  // A button that is disabled with a visible reason instead of silently doing nothing.
  import type { Snippet } from 'svelte'
  let { reason, onclick, children, primary = false }: { reason: string | null; onclick: () => void; children: Snippet; primary?: boolean } = $props()
</script>
<span class="g">
  <button class:primary disabled={reason !== null} title={reason ?? undefined} {onclick}>{@render children()}</button>
  {#if reason}<small>{reason}</small>{/if}
</span>
<style>
  .g { display: inline-grid; gap: 4px; max-width: 260px; }
  small { color: var(--text-muted); font-size: 11px; line-height: 1.3; }
</style>
