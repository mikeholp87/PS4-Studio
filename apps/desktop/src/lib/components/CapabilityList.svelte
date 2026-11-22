<script lang="ts">
  import type { CapabilityEntry, Tier } from '../api'
  let { entries }: { entries: CapabilityEntry[] } = $props()
  const tiers: [Tier, string][] = [
    ['stock', 'Stock console'], ['remote_play', 'Remote Play'], ['homebrew', 'Homebrew services'], ['companion', 'PS4 Studio companion'],
  ]
  const label = { available: 'Available', unavailable: 'Unavailable', unknown: 'Not verified' }
</script>
{#each tiers as [tier, title]}
  <h3>{title}</h3>
  <ul>
    {#each entries.filter((e) => e.tier === tier) as e}
      <li>
        <span class="dot {e.status}" aria-hidden="true"></span>
        <div>
          <code>{e.capability}</code> <b class={e.status}>{label[e.status]}</b>
          <p>{e.status === 'available' ? e.evidence : e.reason}</p>
        </div>
      </li>
    {/each}
  </ul>
{/each}
<style>
  h3 { margin: var(--space-4) 0 var(--space-2); }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-2); }
  li { display: flex; gap: var(--space-3); }
  .dot { width: 9px; height: 9px; border-radius: 50%; margin-top: 7px; flex: none; background: var(--text-muted); }
  .dot.available { background: var(--ok); } .dot.unavailable { background: var(--bad); } .dot.unknown { background: var(--warn); }
  code { font-family: var(--mono); font-size: 12px; }
  b { font-size: 11px; font-weight: 600; margin-left: 6px; } b.available { color: var(--ok); } b.unavailable { color: var(--bad); } b.unknown { color: var(--warn); }
  p { color: var(--text-muted); font-size: 12px; }
</style>
