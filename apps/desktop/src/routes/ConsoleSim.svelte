<script lang="ts">
  // Console UI SIMULATION. Not an operating system and not connected to any console. All content is
  // invented demo data; nothing here reads or reports real hardware values.
  import { onMount, onDestroy } from 'svelte'
  import { app } from '../lib/app.svelte'
  import { api } from '../lib/api'
  import { parseThemeText, FONT_STACKS, type Theme } from '@ps4studio/theme-engine'

  type Item = { label: string; sub?: string; kind: 'game' | 'media' | 'dir' | 'file' | 'toggle' | 'action' | 'theme' | 'info'; key?: string }
  const cats = ['Games', 'Media', 'Files', 'Network', 'Themes', 'System', 'Settings'] as const

  const fs: Record<string, Item[]> = {
    '/': [{ label: 'Documents', kind: 'dir' }, { label: 'Captures', kind: 'dir' }, { label: 'notes.txt', kind: 'file', sub: 'demo file' }],
    '/Documents': [{ label: 'readme.txt', kind: 'file', sub: 'demo file' }],
    '/Captures': [{ label: 'screenshot-001.png', kind: 'file', sub: 'demo image' }, { label: 'clip-001.mp4', kind: 'file', sub: 'demo video' }],
  }
  let cat = $state(0)
  let idx = $state(0)
  let path = $state('/')
  let overlay = $state<string | null>(null)
  let toast = $state('')
  let themes = $state<Theme[]>([])
  let active = $state<Theme | null>(null)
  let settings = $state<Record<string, boolean>>({ clock: true, hour24: true, reduceMotion: false })
  let net = $state({ wifi: true, dhcp: true, last: 'Not tested' })
  let now = $state(new Date())
  let box: HTMLDivElement
  let scale = $state(0.5)
  let progress = $state(0)
  let timers: ReturnType<typeof setInterval>[] = []

  function saveSettings() { try { localStorage.setItem('ps4studio.sim.settings', JSON.stringify(settings)) } catch { /* ignore */ } }

  const items = $derived.by<Item[]>(() => {
    switch (cats[cat]) {
      case 'Games': return ['Starlight Runner', 'Block Garden', 'Neon Courier', 'Cave Echo'].map((l) => ({ label: l, sub: 'demo title (not a real game)', kind: 'game' as const }))
      case 'Media': return [{ label: 'Sample film', sub: 'video · demo', kind: 'media' }, { label: 'Ambient loop', sub: 'music · demo', kind: 'media' }, { label: 'Photo album', sub: 'images · demo', kind: 'media' }]
      case 'Files': return [...(path !== '/' ? [{ label: '..', sub: 'up', kind: 'dir' as const }] : []), ...(fs[path] ?? [])]
      case 'Network': return [
        { label: `Connection: ${net.wifi ? 'Wi-Fi' : 'Wired'}`, sub: 'toggle (simulated)', kind: 'toggle', key: 'wifi' },
        { label: `Address: ${net.dhcp ? 'Automatic (DHCP)' : 'Manual'}`, sub: 'toggle (simulated)', kind: 'toggle', key: 'dhcp' },
        { label: 'Test connection', sub: net.last, kind: 'action', key: 'test' }]
      case 'Themes': return [{ label: 'Default', sub: 'built-in', kind: 'theme', key: '' }, ...themes.map((t) => ({ label: t.name, sub: `${t.template} · saved theme`, kind: 'theme' as const, key: t.id }))]
      case 'System': return [
        { label: 'Device', sub: 'Simulated console', kind: 'info' }, { label: 'System software', sub: 'ps4studio-sim 0.1', kind: 'info' },
        { label: 'Storage', sub: 'not measured: simulation has no hardware', kind: 'info' }, { label: 'Temperature / fan', sub: 'not available in a simulation', kind: 'info' }]
      default: return [
        { label: `Show clock: ${settings.clock ? 'On' : 'Off'}`, kind: 'toggle', key: 'clock' }, { label: `24-hour time: ${settings.hour24 ? 'On' : 'Off'}`, kind: 'toggle', key: 'hour24' },
        { label: `Reduce motion: ${settings.reduceMotion ? 'On' : 'Off'}`, kind: 'toggle', key: 'reduceMotion' }]
    }
  })
  $effect(() => { if (idx >= items.length) idx = Math.max(0, items.length - 1) })

  const bg = $derived(!active ? 'linear-gradient(160deg,#10131c,#1a1440)' : active.background.kind === 'solid' ? active.background.from : `linear-gradient(${active.background.angle}deg,${active.background.from},${active.background.to})`)
  const accent = $derived(active?.colors.accent ?? '#6d8cff')
  const text = $derived(active?.colors.text ?? '#eef0f6')
  const time = $derived(now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: !settings.hour24 }))

  function go(dc: number, di: number) {
    if (overlay) return
    if (dc) { cat = (cat + dc + cats.length) % cats.length; idx = 0; toast = '' }
    if (di) idx = Math.max(0, Math.min(items.length - 1, idx + di))
  }
  function back() {
    if (overlay) { overlay = null; timers.forEach(clearInterval); timers = []; return }
    if (cats[cat] === 'Files' && path !== '/') path = path.replace(/\/[^/]+$/, '') || '/'
  }
  function select() {
    const it = items[idx]
    if (!it) return
    if (it.kind === 'game') overlay = `Launching “${it.label}”…`
    else if (it.kind === 'media') {
      timers.forEach(clearInterval); timers = []
      overlay = `Now playing “${it.label}”`; progress = 0
      timers.push(setInterval(() => (progress = (progress + 4) % 100), 400))
    }
    else if (it.kind === 'dir') path = it.label === '..' ? path.replace(/\/[^/]+$/, '') || '/' : (path === '/' ? '' : path) + '/' + it.label
    else if (it.kind === 'toggle' && it.key) {
      if (it.key === 'wifi' || it.key === 'dhcp') net[it.key] = !net[it.key]
      else { settings[it.key] = !settings[it.key]; saveSettings() }
    } else if (it.key === 'test') { net.last = 'Testing…'; setTimeout(() => (net.last = 'Simulated result: reachable (not a real test)'), 700) }
    else if (it.kind === 'theme') { active = it.key ? (themes.find((t) => t.id === it.key) ?? null) : null; toast = `Theme: ${it.label}` }
    else if (it.kind === 'file') toast = 'Demo file: nothing to open in a simulation.'
  }
  function key(e: KeyboardEvent) {
    const k = e.key
    if (k === 'ArrowRight') go(1, 0); else if (k === 'ArrowLeft') go(-1, 0)
    else if (k === 'ArrowDown') go(0, 1); else if (k === 'ArrowUp') go(0, -1)
    else if (k === 'Enter' || k === ' ') select(); else if (k === 'Escape' || k === 'Backspace') back()
    else return
    e.preventDefault()
  }

  // Gamepad: DualShock-style mapping via the browser Gamepad API (D-pad 12-15, cross 0, circle 1).
  let raf = 0, held: Record<number, number> = {}, focused = false
  function poll() {
    raf = 0
    if (!focused) return
    const gp = navigator.getGamepads?.().find((g) => g)
    if (gp) {
      const t = performance.now()
      const press = (i: number, f: () => void) => { if (gp.buttons[i]?.pressed) { if (!held[i] || t - held[i] > 250) { f(); held[i] = held[i] ? t - 130 : t } } else held[i] = 0 }
      press(14, () => go(-1, 0)); press(15, () => go(1, 0)); press(12, () => go(0, -1)); press(13, () => go(0, 1)); press(0, select); press(1, back)
    }
    raf = requestAnimationFrame(poll)
  }
  const setFocus = (v: boolean) => { focused = v; if (v && !raf) raf = requestAnimationFrame(poll) }

  let ro: ResizeObserver, clock: ReturnType<typeof setInterval>
  onMount(async () => {
    try { const s = localStorage.getItem('ps4studio.sim.settings'); if (s) Object.assign(settings, JSON.parse(s)) } catch { /* storage unavailable */ }
    ro = new ResizeObserver(() => requestAnimationFrame(() => (scale = box.clientWidth / 1920)))
    ro.observe(box)
    clock = setInterval(() => (now = new Date()), 30_000)
    const rows = (await app.run(api.listThemes)) ?? []
    themes = rows.flatMap((r) => { const p = parseThemeText(r.json); return p.ok ? [p.theme] : [] })
  })
  onDestroy(() => { ro?.disconnect(); clearInterval(clock); timers.forEach(clearInterval); if (raf) cancelAnimationFrame(raf) })
</script>

<div class="top">
  <button onclick={() => app.go('settings')}>← Back to Settings</button>
  <span class="badge">SIMULATION · not an operating system · not connected to any console · demo data only</span>
</div>
<div class="tv" bind:this={box} style:height="{1080 * scale}px">
  <!-- Composite widget simulating a console home screen: arrow keys and a controller are its documented interaction. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="screen" role="application" aria-label="Console UI simulation. Arrow keys move, Enter selects, Escape goes back." tabindex="0"
    onkeydown={key} onfocus={() => setFocus(true)} onblur={() => setFocus(false)} onmouseenter={() => setFocus(true)} onmouseleave={() => setFocus(false)}
    class:calm={settings.reduceMotion}
    style:transform="scale({scale})" style:background={bg} style:color={text} style:--accent={accent}
    style:font-family={active ? FONT_STACKS[active.font.family] : FONT_STACKS.system}>
    {#if settings.clock}<div class="clock">{time}</div>{/if}
    <nav class="cats" aria-label="Categories">
      {#each cats as c, i}<span class:on={i === cat}>{c}</span>{/each}
    </nav>
    <ul class="items" aria-label="{cats[cat]} items">
      {#if cats[cat] === 'Files'}<li class="path">{path}</li>{/if}
      {#each items as it, i}
        <li class:on={i === idx} aria-current={i === idx}><b>{it.label}</b>{#if it.sub}<small>{it.sub}</small>{/if}</li>
      {/each}
    </ul>
    {#if toast}<div class="toast" role="status">{toast}</div>{/if}
    {#if overlay}
      <div class="overlay" role="dialog" aria-label={overlay}>
        <h2>{overlay}</h2>
        {#if overlay.startsWith('Now playing')}<div class="bar"><i style:width="{progress}%"></i></div>{/if}
        <p>Simulation only. Nothing is launched or played.</p><p class="hint">Press Escape / Circle to go back.</p>
      </div>
    {/if}
  </div>
</div>
<p class="hint">Click the screen, then use arrow keys and Enter. Escape goes back. A connected gamepad works too (D-pad, ✕ select, ◯ back). Themes saved in Dashboard Studio appear under Themes.</p>

<style>
  .top { display: flex; gap: var(--space-3); align-items: center; margin-bottom: var(--space-3); flex-wrap: wrap; }
  .badge { font-size: 11px; font-weight: 700; letter-spacing: .06em; padding: 4px 10px; border: 1px solid var(--warn); color: var(--warn); border-radius: 4px; }
  .tv { position: relative; width: 100%; overflow: hidden; border: 1px solid var(--border); border-radius: 10px; background: #000; }
  .screen { position: absolute; left: 0; top: 0; width: 1920px; height: 1080px; transform-origin: 0 0; outline: none; overflow: hidden; font-size: 34px; }
  .screen:focus-visible { box-shadow: inset 0 0 0 6px var(--accent); }
  .clock { position: absolute; top: 56px; right: 90px; font-size: 44px; }
  .cats { position: absolute; top: 150px; left: 120px; right: 120px; display: flex; gap: 70px; font-size: 40px; opacity: .85; }
  .cats span { opacity: .55; padding-bottom: 8px; border-bottom: 6px solid transparent; transition: opacity .15s; } .cats span.on { opacity: 1; border-bottom-color: var(--accent); }
  .items { position: absolute; top: 300px; left: 160px; right: 160px; list-style: none; margin: 0; padding: 0; display: grid; gap: 14px; }
  .items li { display: flex; gap: 30px; align-items: baseline; padding: 18px 28px; border-radius: 16px; opacity: .65; transition: background .15s, opacity .15s, transform .15s; }
  .items li.on { opacity: 1; background: color-mix(in srgb, var(--accent) 28%, transparent); transform: translateX(14px); box-shadow: 0 0 0 4px var(--accent); }
  .items li.path { opacity: .6; font-family: var(--mono); font-size: 28px; padding: 0 28px; background: none; box-shadow: none; transform: none; }
  small { opacity: .7; font-size: 26px; }
  .toast { position: absolute; bottom: 70px; left: 160px; font-size: 28px; padding: 12px 24px; border-radius: 12px; background: rgba(0,0,0,.5); }
  .overlay { position: absolute; inset: 0; background: rgba(5,6,12,.92); display: grid; place-content: center; text-align: center; gap: 20px; }
  .bar { width: 800px; height: 12px; border-radius: 6px; background: rgba(255,255,255,.2); margin: 0 auto; } .bar i { display: block; height: 100%; border-radius: 6px; background: var(--accent); }
  .hint { color: var(--text-muted); font-size: 12px; margin-top: var(--space-2); } .overlay .hint { color: inherit; opacity: .6; font-size: 26px; }
  .calm .items li, .calm .cats span { transition: none; }
  @media (prefers-reduced-motion: reduce) { .items li, .cats span { transition: none; } }
</style>
