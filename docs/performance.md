# Performance

Measured 2026-11-22 on the author's machine (12 threads, 31 GB RAM, Debian forky, KDE Plasma/Wayland) with the **release binary extracted from the `.deb`**, scratch XDG directories, no console connected. Method: `scripts/measure-perf.sh` (3 runs).

| Metric | Target | Measured (3 runs) | Notes |
|---|---|---|---|
| Database open ("app logic ready") | < 500 ms | 60 / 72 / 94 ms | Proxy for startup. **Time until the window is visible was not measured** (no reliable Wayland hook available). |
| Idle CPU (app + WebKit processes, 10 s window after 8 s settle) | < 1 % steady state | 2.0 % / 0 % / 0 % | First run included webview settling. |
| Memory (PSS summed over 3 processes: app, WebKitWebProcess, WebKitNetworkProcess) | < 400 MB | 271 / 265 / 264 MB | Dominated by the WebKit web process, which is typical for a Tauri app. |
| Package size | — | 4.65 MiB `.deb`, 12.4 MB binary | `lto = "thin"`, `strip = true` |

Design choices that keep idle cost low: no background daemons or timers in Rust; discovery is user-initiated; the Remote Play status is polled only while a session runs; the gamepad `requestAnimationFrame` loop runs only while a pad is connected *and* the preview is focused/hovered; the preview clock ticks every 30 s.

Not measured: large file transfers (the simulator moves small payloads; transfers are async and chunked at 64 KB, with cancellation, but throughput and UI smoothness against a real console are unknown), and GNOME/X11.
