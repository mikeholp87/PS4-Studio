# Performance

Measured 2026-11-22 on the author's machine (12 threads, 31 GB RAM, Debian forky, KDE Plasma/Wayland) with the **release binary extracted from the `.deb`**, scratch XDG directories, no console connected. Method: `scripts/measure-perf.sh` (3 runs).

| Metric | Target | Measured (3 clean runs) | Notes |
|---|---|---|---|
| Database open ("app logic ready") | < 500 ms | 59 / 59 / 59 ms | Proxy for startup. **Time until the window is visible was not measured** (no reliable Wayland hook available). |
| Idle CPU (app + WebKit processes, 10 s window after 8 s settle, UI rendered) | < 1 % steady state | 0 % / 0 % / 0 % | |
| Memory (PSS summed over 3 processes: app, WebKitWebProcess, WebKitNetworkProcess) | < 400 MB | 250 / 253 / 271 MB | Dominated by the WebKit web process, which is typical for a Tauri app. |
| Package size | — | 4.62 MiB `.deb` | `lto = "thin"`, `strip = true` |

**Correction history.** An earlier version of this page reported similar numbers measured against a build whose window rendered nothing (see the CSP bug in `docs/tasks.md`), so they proved little. These numbers were re-measured on the rendering build. Runs taken while the machine was heavily loaded (load average ≈ 31) were much worse (DB-ready 350–500 ms, idle CPU 18–28 %): measure on a quiet machine.

Design choices that keep idle cost low: no background daemons or timers in Rust; discovery is user-initiated; the Remote Play status is polled only while a session runs; the gamepad `requestAnimationFrame` loop runs only while a pad is connected *and* the preview is focused/hovered; the preview clock ticks every 30 s.

Not measured: large file transfers (the simulator moves small payloads; transfers are async and chunked at 64 KB, with cancellation, but throughput and UI smoothness against a real console are unknown), and GNOME/X11.
