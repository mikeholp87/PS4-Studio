<script lang="ts">
  import { onMount } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api, type ConsoleProfile, type ConsoleModel, type Scenario, type Discovered } from '../lib/api'
  import Card from '../lib/components/Card.svelte'

  let profiles = $state<ConsoleProfile[]>([])
  let scenarios = $state<Scenario[]>([])
  let found = $state<Discovered[] | null>(null)
  let searching = $state(false)
  let busy = $state<number | null>(null)
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
  async function search() {
    searching = true
    found = (await app.run(api.discoverLan)) ?? null
    searching = false
  }
  function use(d: Discovered) {
    form = { ...form, name: d.name ?? 'My PS4', address: d.address, firmware: d.firmware_hint ?? '' }
  }
  async function connect(id: number) {
    busy = id
    const c = await app.run(() => api.connectProfile(id))
    busy = null
    if (c) { app.connection = c; await load(); app.go(c.capabilities.some((x) => x.capability === 'console.basic_status' && x.status === 'available') ? 'overview' : 'diagnostics') }
  }
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

  <Card title="Find consoles on this network">
    <p class="hint">Sends one discovery broadcast (UDP 987) on your local network and lists consoles that answer. Nothing else is probed. A console in Rest Mode may answer as standby.</p>
    <button onclick={search} disabled={searching}>{searching ? 'Searching…' : 'Search local network'}</button>
    {#if found}
      {#if found.length === 0}
        <p class="hint">No console answered. Confirm that the Debian computer and PS4 are on the same local network, then try again or add the console by IP address.</p>
      {:else}
        <table>
          <thead><tr><th>Name</th><th>Address</th><th>State</th><th></th></tr></thead>
          <tbody>{#each found as d}<tr><td>{d.name ?? '—'}</td><td>{d.address}</td><td>{d.state}</td><td class="end"><button onclick={() => use(d)}>Use</button></td></tr>{/each}</tbody>
        </table>
      {/if}
    {/if}
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
    <p class="hint">Saving does not contact the console. Press Connect on a saved console to test it.</p>
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
                <button class="primary" disabled={busy === p.id} onclick={() => connect(p.id)}>{busy === p.id ? 'Testing…' : 'Connect'}</button>
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
