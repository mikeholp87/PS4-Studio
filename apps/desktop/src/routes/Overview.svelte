<script lang="ts">
  import { app } from '../lib/app.svelte'
  import Card from '../lib/components/Card.svelte'
  import Gated from '../lib/components/Gated.svelte'
  import CapabilityList from '../lib/components/CapabilityList.svelte'
  import type { ConsoleModel, FirmwareSource } from '../lib/api'

  const conn = $derived(app.connection)
  const models: Record<ConsoleModel, string> = { phat: 'PS4 (original)', slim: 'PS4 Slim', pro: 'PS4 Pro', unknown: 'Unknown' }
  const fwSrc: Record<FirmwareSource, string> = { unknown: 'not verified', reported: 'reported by console, not verified', user_entered: 'entered by you, not verified' }
  const online = $derived(conn?.capabilities.find((c) => c.capability === 'console.basic_status')?.status === 'available')
  const hb = $derived(conn?.capabilities.find((c) => c.capability === 'homebrew.ftp_read')?.status)
  const unavailable = ['CPU temperature', 'Fan status', 'Storage capacity', 'Memory use']
</script>

{#if !conn}
  <Card>
    <div class="empty">
      <h2>No console selected</h2>
      <p>PS4 Studio works without a console. Try the built-in simulator, add your PS4 by IP address, or design a dashboard offline.</p>
      <div class="row">
        <button class="primary" onclick={() => app.go('connections')}>Choose a console</button>
        <button onclick={() => app.go('studio')}>Open Dashboard Studio</button>
      </div>
    </div>
  </Card>
{:else}
  <div class="grid">
    <Card title="Console">
      <dl>
        <dt>Name</dt><dd>{conn.profile.name}</dd>
        <dt>Model</dt><dd>{models[conn.profile.model]} <small>(selected by you)</small></dd>
        <dt>Connection</dt><dd>{conn.simulated ? `Simulator · ${conn.scenario_id}` : 'Network'}</dd>
        <dt>Address</dt><dd>{conn.profile.address ?? '—'}</dd>
        <dt>Status</dt><dd>{online ? 'Answered discovery' : 'No answer yet'}</dd>
        <dt>Firmware</dt><dd>{conn.profile.firmware.value ?? 'Unknown'} <small>{fwSrc[conn.profile.firmware.source]}</small></dd>
        <dt>Homebrew</dt><dd>{hb === 'available' ? 'File service verified' : 'Not verified'}</dd>
      </dl>
      <p class="note">Firmware compatibility has not been established.</p>
    </Card>

    <Card title="Quick actions">
      <div class="actions">
        <button class="danger" onclick={() => app.disconnect()}>Disconnect</button>
        <Gated primary reason={app.blocked('console.remote_play')} onclick={() => app.go('remote')}>Start Remote Play</Gated>
        <button onclick={() => app.go('studio')}>Open Dashboard Studio</button>
        <button onclick={() => app.go('files')}>Browse Files</button>
        <button onclick={() => app.go('backups')}>Create Backup</button>
        <button onclick={() => app.go('diagnostics')}>Open Diagnostics</button>
      </div>
    </Card>

    <Card title="Console measurements">
      <ul class="na">
        {#each unavailable as m}<li><span>{m}</span><em>Unavailable</em></li>{/each}
      </ul>
      <p class="note">A stock PS4 exposes none of these over the network. They only appear when a verified companion reports them.</p>
    </Card>

    <Card title="Capabilities">
      <CapabilityList entries={conn.capabilities} />
    </Card>
  </div>
{/if}

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: var(--space-4); align-items: start; }
  .empty { display: grid; gap: var(--space-3); max-width: 560px; }
  .row, .actions { display: flex; flex-wrap: wrap; gap: var(--space-3); align-items: flex-start; }
  dl { display: grid; grid-template-columns: 110px 1fr; gap: 6px var(--space-3); margin: 0; }
  dt { color: var(--text-muted); } dd { margin: 0; } small { color: var(--text-muted); }
  .note { color: var(--text-muted); font-size: 12px; margin-top: var(--space-3); }
  .na { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
  .na li { display: flex; justify-content: space-between; } em { color: var(--text-muted); font-style: normal; }
</style>
