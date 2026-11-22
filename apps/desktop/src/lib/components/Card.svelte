<script lang="ts">
  import type { Snippet } from 'svelte'
  let { title, children, actions }: { title?: string; children: Snippet; actions?: Snippet } = $props()
</script>
<section class="card">
  {#if title || actions}<div class="head">{#if title}<h2>{title}</h2>{/if}{@render actions?.()}</div>{/if}
  {@render children()}
</section>
<style>
  .card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: var(--space-4); }
  .head { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-3); }
</style>
