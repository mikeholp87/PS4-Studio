<script lang="ts">
  // 1080p TV simulation. Renders a theme as plain DOM (no theme string is ever evaluated as code),
  // supports arrow keys, a gamepad, and on-screen D-pad buttons.
  import { onMount, onDestroy } from 'svelte'
  import { Gamepad2, Tv, Music, Film, Globe, Settings, Folder, ShoppingBag, Users, Camera, Star, Download, Play, Image as ImageIcon } from 'lucide-svelte'
  import { FONT_STACKS, moveFocus, type IconName, type Theme } from '@ps4studio/theme-engine'

  let { theme }: { theme: Theme } = $props()

  const icons: Record<IconName, typeof Star> = {
    gamepad: Gamepad2, tv: Tv, music: Music, film: Film, globe: Globe, settings: Settings, folder: Folder,
    store: ShoppingBag, users: Users, camera: Camera, star: Star, download: Download, play: Play, image: ImageIcon,
  }
  const sizes = { small: 140, medium: 200, large: 270 }
  const radii = { square: '10px', rounded: '34px', circle: '50%' }

  let focus = $state(0)
  let message = $state('')
  let box: HTMLDivElement
  let scale = $state(0.4)
  let now = $state(new Date())
  let pad = $state<string | null>(null)

  const count = $derived(theme.tiles.length)
  const cols = $derived(theme.template === 'grid' ? theme.layout.columns : Math.max(1, count))
  const px = $derived(sizes[theme.layout.tileSize])
  const bg = $derived(
    theme.background.kind === 'solid'
      ? theme.background.from
      : `linear-gradient(${theme.background.angle}deg, ${theme.background.from}, ${theme.background.to})`,
  )
  const time = $derived(now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: !theme.widgets.clock.hour24 }))
  // Single-row templates scroll horizontally so the focused tile stays on screen (like a real TV shelf).
  const gapRatio = $derived(theme.template === 'aurora' ? 0.12 : 0.16)
  const shift = $derived.by(() => {
    if (theme.template === 'grid') return 0
    const step = px * (1 + gapRatio)
    const viewport = 1920 - 240
    const max = Math.max(0, count * step - px * gapRatio - viewport)
    return Math.min(max, Math.max(0, focus * step - (viewport - px) / 2))
  })
  $effect(() => { if (focus >= count) focus = Math.max(0, count - 1) })

  function move(dir: 'left' | 'right' | 'up' | 'down') {
    focus = moveFocus(focus, dir, count, cols, theme.navigation.wrapFocus)
    message = ''
  }
  function activate() {
    const t = theme.tiles[focus]
    // Preview only: nothing is launched.
    message = t ? `Preview: would open “${t.label}” (${t.kind === 'app' ? 'application' : 'shortcut'}). Nothing is launched.` : ''
  }
  function onKey(e: KeyboardEvent) {
    const map: Record<string, 'left' | 'right' | 'up' | 'down'> = { ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'up', ArrowDown: 'down' }
    if (map[e.key]) { move(map[e.key]); e.preventDefault() }
    else if (e.key === 'Enter' || e.key === ' ') { activate(); e.preventDefault() }
  }

  // Gamepad: poll only while a pad is connected and the preview has focus/hover (rAF is paused by the browser when hidden).
  let raf = 0
  let active = false
  let held: Record<number, number> = {}
  function poll() {
    raf = 0
    if (!active) return
    const gp = navigator.getGamepads?.().find((g) => g)
    if (gp) {
      const t = performance.now()
      const press = (i: number, f: () => void) => {
        if (gp.buttons[i]?.pressed) { if (!held[i] || t - held[i] > 220) { f(); held[i] = held[i] ? t - 120 : t } } else held[i] = 0
      }
      press(14, () => move('left')); press(15, () => move('right')); press(12, () => move('up')); press(13, () => move('down')); press(0, activate)
      const [ax, ay] = [gp.axes[0] ?? 0, gp.axes[1] ?? 0]
      if (Math.abs(ax) > 0.6 && (!held[100] || t - held[100] > 220)) { move(ax < 0 ? 'left' : 'right'); held[100] = t } else if (Math.abs(ax) < 0.4) held[100] = 0
      if (Math.abs(ay) > 0.6 && (!held[101] || t - held[101] > 220)) { move(ay < 0 ? 'up' : 'down'); held[101] = t } else if (Math.abs(ay) < 0.4) held[101] = 0
    }
    raf = requestAnimationFrame(poll)
  }
  function setActive(v: boolean) { active = v && pad !== null; if (active && !raf) raf = requestAnimationFrame(poll) }

  let clock: ReturnType<typeof setInterval>
  let ro: ResizeObserver
  onMount(() => {
    ro = new ResizeObserver(() => requestAnimationFrame(() => { scale = box.clientWidth / 1920 }))
    ro.observe(box)
    clock = setInterval(() => (now = new Date()), 30_000)
    const on = (e: GamepadEvent) => (pad = e.gamepad.id.slice(0, 40))
    const off = () => { pad = null; setActive(false) }
    window.addEventListener('gamepadconnected', on)
    window.addEventListener('gamepaddisconnected', off)
    return () => { window.removeEventListener('gamepadconnected', on); window.removeEventListener('gamepaddisconnected', off) }
  })
  onDestroy(() => { clearInterval(clock); ro?.disconnect(); if (raf) cancelAnimationFrame(raf) })
</script>

<div class="wrap">
  <div class="tv" bind:this={box} style:height="{1080 * scale}px">
    <!-- Composite widget simulating a TV screen: arrow keys are its documented interaction. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <div
      class="screen {theme.template}"
      role="application"
      aria-label="Dashboard preview. Use arrow keys to move focus."
      tabindex="0"
      onkeydown={onKey}
      onfocus={() => setActive(true)}
      onblur={() => setActive(false)}
      onmouseenter={() => setActive(true)}
      onmouseleave={() => setActive(false)}
      style:transform="scale({scale})"
      style:background={bg}
      style:color={theme.colors.text}
      style:font-family={FONT_STACKS[theme.font.family]}
      style:--accent={theme.colors.accent}
      style:--surface={theme.colors.surface}
      style:--px="{px}px"
      style:--radius={radii[theme.layout.tileShape]}
      style:--fs="{28 * theme.font.scale}px"
      style:--cols={cols}
    >
      {#if theme.template === 'aurora'}<div class="glow g1"></div><div class="glow g2"></div>{/if}

      {#if theme.widgets.clock.enabled}
        <div class="clock {theme.widgets.clock.position}">{time}</div>
      {/if}
      {#if theme.widgets.status.enabled}
        <div class="status {theme.widgets.clock.position === 'top-left' ? 'top-right' : 'top-left'}"><span>Network</span><span>Controller</span><em>preview</em></div>
      {/if}

      {#if theme.template === 'classic'}
        <div class="tabs">{#each theme.tiles.slice(0, 6) as t, i}<span class:on={i === focus}>{t.label}</span>{/each}</div>
      {/if}

      {#if count === 0}
        <p class="none">No tiles. Add one in the editor.</p>
      {:else}
        <div class="tiles" style:transform="translateX({-shift}px)">
          {#each theme.tiles as t, i (t.id)}
            {@const Icon = icons[t.icon]}
            <div class="tile" class:focused={i === focus} style:--tile={t.color}>
              <div class="face"><Icon size={px * 0.42} strokeWidth={1.6} aria-hidden="true" /></div>
              {#if theme.navigation.showLabels && (theme.template !== 'minimal' || i === focus)}<span class="label">{t.label}</span>{/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="bar">
    <div class="dpad" role="group" aria-label="Controller simulation">
      <button onclick={() => move('up')} aria-label="Up">▲</button>
      <button onclick={() => move('left')} aria-label="Left">◀</button>
      <button onclick={activate} aria-label="Select">✕</button>
      <button onclick={() => move('right')} aria-label="Right">▶</button>
      <button onclick={() => move('down')} aria-label="Down">▼</button>
    </div>
    <div class="msg" aria-live="polite">
      {#if message}{message}{:else}Click the preview, then use the arrow keys{pad ? ` or the connected controller (${pad})` : ''}.{/if}
    </div>
  </div>
</div>

<style>
  .wrap { display: grid; gap: var(--space-3); }
  .tv { position: relative; width: 100%; overflow: hidden; border-radius: 10px; border: 1px solid var(--border); background: #000; }
  .screen { position: absolute; left: 0; top: 0; width: 1920px; height: 1080px; transform-origin: 0 0; overflow: hidden; outline: none; font-size: var(--fs); }
  .screen:focus-visible { box-shadow: inset 0 0 0 6px var(--accent); }
  .clock, .status { position: absolute; top: 56px; font-size: calc(var(--fs) * 1.3); z-index: 2; }
  .top-left { left: 80px; } .top-right { right: 80px; }
  .status { display: flex; gap: 24px; font-size: calc(var(--fs) * .8); opacity: .75; align-items: center; } .status em { font-style: normal; opacity: .6; font-size: .8em; }
  .tiles { position: absolute; left: 0; right: 0; display: flex; gap: calc(var(--px) * .16); z-index: 1; transition: transform .25s ease; }
  .tile { display: grid; justify-items: center; gap: 18px; width: var(--px); flex: none; transition: transform .15s ease, opacity .15s ease; opacity: .85; }
  .face { width: var(--px); height: var(--px); border-radius: var(--radius); background: color-mix(in srgb, var(--tile) 32%, var(--surface)); display: grid; place-items: center; color: color-mix(in srgb, var(--tile) 70%, white); box-shadow: 0 10px 30px rgba(0,0,0,.35); }
  .tile.focused { transform: scale(1.14); opacity: 1; }
  .tile.focused .face { box-shadow: 0 0 0 6px var(--accent), 0 16px 44px rgba(0,0,0,.5); }
  .label { font-size: var(--fs); white-space: nowrap; }

  .minimal .tiles { top: 520px; padding: 0 120px; justify-content: flex-start; overflow: visible; }
  .aurora .tiles { top: 380px; padding: 0 120px; gap: calc(var(--px) * .12); }
  .aurora .face { background: color-mix(in srgb, var(--surface) 22%, transparent); backdrop-filter: blur(18px); border: 2px solid rgba(255,255,255,.18); }
  .glow { position: absolute; border-radius: 50%; filter: blur(120px); opacity: .55; }
  .g1 { width: 800px; height: 800px; left: -200px; top: -200px; background: var(--accent); }
  .g2 { width: 700px; height: 700px; right: -100px; bottom: -250px; background: #38bdf8; }
  .grid .tiles { display: grid; grid-template-columns: repeat(var(--cols), var(--px)); justify-content: center; gap: 56px calc(var(--px) * .2); top: 200px; padding: 0; }
  .classic .tiles { top: 440px; padding: 0 120px; align-items: flex-start; }
  .classic .tabs { position: absolute; top: 190px; left: 120px; display: flex; gap: 56px; font-size: calc(var(--fs) * 1.1); opacity: .7; }
  .classic .tabs .on { opacity: 1; border-bottom: 5px solid var(--accent); }
  .none { position: absolute; inset: 0; display: grid; place-items: center; opacity: .6; }

  .bar { display: flex; gap: var(--space-4); align-items: center; flex-wrap: wrap; }
  .dpad { display: flex; gap: 6px; } .dpad button { padding: 4px 10px; }
  .msg { color: var(--text-muted); font-size: 12px; flex: 1; min-width: 200px; }
  @media (prefers-reduced-motion: reduce) { .tile, .tiles { transition: none; } }
</style>
