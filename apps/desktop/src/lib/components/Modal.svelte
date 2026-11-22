<script lang="ts">
  import type { Snippet } from 'svelte'
  let { title, onclose, children, footer }: { title: string; onclose: () => void; children: Snippet; footer?: Snippet } = $props()
  let el: HTMLDivElement
  $effect(() => { el?.focus() })
  function key(e: KeyboardEvent) {
    if (e.key === 'Escape') { onclose(); e.stopPropagation() }
  }
</script>

<div class="scrim" role="presentation" onmousedown={(e) => { if (e.target === e.currentTarget) onclose() }}>
  <div class="dlg" role="dialog" aria-modal="true" aria-label={title} tabindex="-1" bind:this={el} onkeydown={key}>
    <h2>{title}</h2>
    <div class="body">{@render children()}</div>
    {#if footer}<div class="foot">{@render footer()}</div>{/if}
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0,0,0,.55); display: grid; place-items: center; z-index: 50; padding: var(--space-4); }
  .dlg { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); width: min(620px, 100%); max-height: 90vh; overflow: auto; padding: var(--space-5); display: grid; gap: var(--space-4); outline: none; box-shadow: 0 20px 60px rgba(0,0,0,.5); }
  .foot { display: flex; gap: var(--space-2); justify-content: flex-end; flex-wrap: wrap; }
</style>
