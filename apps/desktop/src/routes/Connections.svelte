<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type ConsoleProfile, type ConsoleModel, type Scenario } from '../lib/api'
  import Card from '../lib/components/Card.svelte'
  import Gated from '../lib/components/Gated.svelte'

  let profiles = $state<ConsoleProfile[]>([])
  let scenarios = $state<Scenario[]>([])
  let pick = $state('stock')
  let form = $state({ name: '', model: 'slim' as ConsoleModel, address: '', firmware: '' })

  async function load() {
    profiles = (await app.run(api.listProfiles)) ?? []
  }
  onMount(async () => { scenarios = (await app.run(api.listScenarios)) ?? []; await load() })

  async function add(e: Event) {
    e.preventDefault()
    const p = await app.run(() => api.addProfile({ ...form, firmware: form.firmware || null }))
    if (p) { form = { name: '', model: 'slim', address: '', firmware: '' }; await load() }
  }
  async function remove(id: number) { if (await app.run(() => api.deleteProfile(id))) await load() }
  async function sim() { const c = await app.run(() => api.connectSimulator(pick)); if (c) { app.connection = c; app.go('overview') } }
  const desc = $derived(scenarios.find((s) => s.id === pick)?.description ?? '')
</script>

<div class="cols">
  <Card title="Offline simulator">
    <p class="hint">A built-in fake console for trying the app without hardware. Everything it shows is labelled SIMULATED.</p>
    <label>Scenario
      <select bind:value={pick}>{#each scenarios as s}<option value={s.id}>{s.label}</option>{/each}</select>
    </label>
    <p class="hint">{desc}</p>
    <button class="primary" onclick={sim}>Connect to simulator</button>
  </Card>

  <Card title="Add your console">
    <form onsubmit={add}>
      <label>Name<input bind:value={form.name} maxlength="48" required placeholder="Living room PS4" /></label>
      <label>Model (what you own)
        <select bind:value={form.model}>
          <option value="slim">PS4 Slim</option><option value="phat">PS4 (original)</option>
          <option value="pro">PS4 Pro</option><option value="unknown">Not sure</option>
        </select>
      </label>
      <label>IP address<input bind:value={form.address} required placeholder="192.168.1.50" inputmode="decimal" /></label>
      <label>Firmware (optional, as shown on the PS4)<input bind:value={form.firmware} placeholder="9.00" /></label>
      <button class="primary" type="submit">Save console</button>
    </form>
    <p class="hint">Saving does not contact the console. Network discovery and connection tests arrive in the next phase.</p>
  </Card>

  <Card title="Saved consoles">
    {#if profiles.length === 0}
      <p class="hint">No saved consoles yet.</p>
    {:else}
      <table>
        <thead><tr><th>Name</th><th>Model</th><th>Address</th><th>Firmware</th><th></th></tr></thead>
        <tbody>
          {#each profiles as p}
            <tr>
              <td>{p.name}</td><td>{p.model}</td><td>{p.address}</td><td>{p.firmware.value ?? '—'}</td>
              <td class="end">
                <Gated reason="Connection tests arrive in phase 3 (network discovery)." onclick={() => {}}>Connect</Gated>
                <button class="danger" onclick={() => remove(p.id)}>Remove</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Card>
</div>

<style>
  .cols { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  :global(.cols > :last-child) { grid-column: 1 / -1; }
  form, :global(section.card > label) { display: grid; gap: var(--space-3); }
  .hint { color: var(--text-muted); font-size: 12px; margin: var(--space-2) 0 var(--space-3); }
  .end { display: flex; gap: var(--space-2); justify-content: flex-end; align-items: flex-start; }
</style>
