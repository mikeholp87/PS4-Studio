<!-- Recovered verbatim from Claude Code session 195c2a5a-cb4a-4b40-b987-eb35bacc41c6 on 2026-11-22. Historical context; claims require verification before integration. -->

# PS4 Studio — implementation plan

## Context
Greenfield, empty dir `/home/mikeh/Projects/ps4-studio` (not yet a git repo). Goal: Debian desktop app (Tauri 2 + Svelte 5 + Rust) to manage a PS4 Slim over LAN: discovery, Remote Play launch, capability-gated homebrew/file features, Dashboard Studio (offline theme designer), backups, diagnostics, simulator, optional OpenOrbis companion, separate alt-OS research track. Spec is the pasted 27-section brief; this plan fixes order, decisions, verification.

## Environment findings (verified, read-only)
- Debian forky/sid, KDE on Wayland. rustc/cargo 1.99.1, node 24.16, pnpm 12.21, bun, gcc 16, clang 21, cmake 4.3, ffmpeg, gh 2.105, dpkg-deb, fakeroot — all present.
- Tauri deps present: webkit2gtk-4.1-dev 2.52.6, gtk-3-dev, libsoup-3.0-dev, ssl-dev, ayatana-appindicator, rsvg2, sqlite3-dev.
- Missing: `libsecret-1-dev` (NOT needed: use pure-Rust `keyring` w/ `sync-secret-service`/zbus), `patchelf` (AppImage only, optional, needs sudo → ask first), `cargo-tauri`, `cargo-deb` (user-space `cargo install`), chiaki-ng binary, adb.
- No sudo needed for core path.

## Upstream facts (verified via `gh api`, 2026-11-22)
| Repo | License | Last release | Note |
|---|---|---|---|
| chiaki-ng | **AGPL-3.0** | v1.10.1 (2026-11-03) | active; README confirms PS4+PS5 Remote Play client |
| OpenOrbis toolchain | GPL-3.0 | v0.5.4 (2026-01) | active |
| Itemzflow | GPL-3.0 | 1.08 (2026-05) | PS4 game manager / home alternative — prior art |
| GoldHEN | none declared | 2.3 (2022-12) | README lists FTP on 2121, plugins, `/data/pkg` install; firmware range must be read from release notes in Phase 1, never assumed |
| cleanyourtv | NOASSERTION | v3.6.0 (2026-11-20) | Android/Google TV debloat over ADB — prior art for optional TV module only |

## Key decisions (my calls — veto any)
1. **Remote Play = Option A**: spawn installed `chiaki-ng` as separate process. AGPL never touches our code. No embedding. Binary not installed here → feature is CONDITIONALLY SUPPORTED, detected at runtime, UI shows install instructions.
2. **License: GPL-3.0-or-later** for whole repo (companion links GPL OpenOrbis; simplest single license; compatible with later AGPL embedding if ever wanted). Revisit after `docs/dependencies-and-licenses.md` audit.
3. **Discovery**: PS4 wake/discovery uses UDP 987 `SRCH` / `WAKEUP` (documented via chiaki source) — implement own small Rust client against the documented wire format; verify against chiaki-ng source before coding; scoped to user-given subnet/unicast + limited broadcast only.
4. **Secrets**: `keyring` crate (Secret Service, no libsecret C dep). Fallback: refuse to store, never plaintext.
5. **Persistence**: `rusqlite` (bundled off, links system sqlite3) in XDG dirs via `directories`/`dirs`.
6. **FTP**: `suppaftp` (async, tokio); read-only default; write needs explicit per-session unlock; allowlist roots (`/data`, `/user/app`-style dirs confirmed in Phase 1 research, not guessed); path normalisation rejects `..`, absolute escapes, symlink tricks.
7. **Capability engine**: typed enum `Capability`, each with `Evidence` (probe result, timestamp, source) — never inferred from open port; unknown ≠ supported. Disabled-reason string per capability drives UI.
8. **Safe-op model** INSPECT→PLAN→CONFIRM→APPLY→VERIFY→RECOVER as a Rust state machine in `crates/core`, every APPLY writes audit row to SQLite.
9. **Simulator**: `simulator/mock-console` is a real tokio TCP/UDP server (fake discovery reply, fake FTP, fake companion) with scripted scenarios (the 14 listed). Every response carries `simulated:true`; UI shows persistent SIMULATED banner. Used in Rust integration tests and Playwright.
10. **Theme format**: versioned JSON (`schemaVersion`), JSON Schema + `serde` strict (`deny_unknown_fields`), size limits, no URLs/scripts, assets referenced only as embedded data-URI images ≤ limit.
11. **Monorepo**: Cargo workspace (`crates/*`, `simulator/mock-console`, `apps/desktop/src-tauri`) + pnpm workspace (`apps/desktop`, `packages/*`). Start with fewer crates than the suggested tree (merge `security` into `core`, `file-transfer` into `console-adapters`) until a boundary earns its own crate; docs say so in `decisions.md`.
12. **Companion**: C/C++ source + Makefile for OpenOrbis in `apps/ps4-companion`, **not built here** (toolchain absent); desktop build never depends on it. Protocol crate (`crates/core::companion_proto`) is Rust, tested against a Rust mock. Auth: pairing PIN shown on PS4 → HMAC-SHA256 session keys, allowlisted commands; encryption only if runtime supports it (documented, else flagged unencrypted on UI).
13. **TV (ADB) + HDMI-CEC**: separate crate/feature flag, UI behind "Optional modules", classified EXPERIMENTAL/CONDITIONAL; adb not installed → disabled with reason. Never disables system apps.
14. **Alt-OS**: docs-only + a standalone Svelte "console UI simulation" route (reuses Dashboard Studio preview renderer). No boot/flash work.
15. **Honesty rule**: no claim of physical-console testing; every doc states "tested against simulator only" unless hardware used.

## Execution order (each phase ends: fmt, clippy, cargo test, svelte-check, eslint, vitest, run app, docs update, commit)
**P0 Bootstrap** (first, small): `git init`; workspace skeleton; `cargo install tauri-cli@^2 cargo-deb`; pnpm workspace; Tauri+Svelte5+Vite scaffold; CI skeleton; AGENTS.md/CLAUDE.md/docs/tasks.md/current-status.md/decisions.md. Commit.
**P1 Research docs**: fetch GoldHEN release notes/README/wiki, chiaki-ng docs + source (discovery/regist/wakeup protocol), OpenOrbis docs, Itemzflow, Sony Remote Play page, Linux-on-PS4 (fail0verflow/ps4-linux, Belize/Baikal GPU notes), secure-boot facts. Write `research.md`, `technical-feasibility.md`, `firmware-compatibility.md` (with source URLs + retrieved date; unknowns marked UNVERIFIED), `dependencies-and-licenses.md` (`cargo deny`/`cargo license` output). Every feature gets SUPPORTED/CONDITIONAL/EXPERIMENTAL/UNSUPPORTED + evidence.
**P2 Foundation**: design tokens (charcoal/blue/violet, dark/light/system, accent), sidebar+header+status pill, 9 section routes, Lucide (`lucide-svelte`), settings in SQLite, Tauri commands w/ minimal capability permissions, mock-console MVP, Overview page using real capability data (unavailable states for temp/storage).
**P3 Connection**: discovery client + wizard (5 pathways), manual IP, profiles (multi), connection test w/ timeout+retry, capability engine + probes, chiaki-ng launcher (arg validation, no creds in logs, process supervision, clean stop), Remote Control page (pairing guide, controller guidance via `gilrs` for DS4 detection), Diagnostics v1 + sanitized export.
**P4 Dashboard Studio**: `packages/theme-engine` (TS types, validator, 4 templates MINIMAL/AURORA/GRID/CLASSIC original), editor panels, 1080p 16:9 scaled preview, keyboard + `gilrs`/Gamepad API nav, focus animations, multi-profile, import/export/reset via Rust-validated commands.
**P5 Homebrew/Files/Backups**: GoldHEN adapter (FTP capability probe: connect, banner, auth, `LIST`), dual-pane file manager (async queue, progress events, cancel, retries, SHA-256 where possible), homebrew package list/validate (manifest+sha256, trusted-publisher list, warnings; install disabled unless verified interface—GoldHEN `/data/pkg` drop is documented, gated behind write unlock + confirm), backups (versioned manifest, verify, restore-single-item, op history, Sony backup instructions for stock), audit log.
**P6 Companion**: C source + Makefile + README prerequisites; Rust protocol + mock; pairing UI; theme transfer UI gated on `companion.theme_management`.
**P7 Alt-OS**: 4 docs + console-UI simulation page; approach A/B/C feasibility matrix.
**P8 Release**: README (+ real screenshots via Playwright/agent-browser), LICENSE, CONTRIBUTING, CoC, SECURITY, CHANGELOG, `packaging/{debian,appimage,flatpak}`, `tauri build --bundles deb` (+AppImage best-effort), GitHub Actions (fmt, clippy, test, frontend, `cargo audit`/`deny`, `pnpm audit`, deb build), install-verify in clean dir with `dpkg -x`/`lintian` if present. Measure startup/idle CPU/RSS with `hyperfine`/`/usr/bin/time -v`; record numbers in docs.

## Critical files (created)
`Cargo.toml` (workspace), `pnpm-workspace.yaml`, `apps/desktop/src-tauri/{tauri.conf.json,capabilities/default.json,src/commands/*.rs}`, `crates/core/{capability.rs,safe_op.rs,db.rs,audit.rs,paths.rs}`, `crates/discovery`, `crates/remote-play`, `crates/console-adapters/{ftp.rs,companion.rs}`, `crates/backups`, `crates/diagnostics`, `simulator/mock-console/src/{main.rs,scenarios.rs}`, `packages/theme-engine/src/{schema.ts,templates/*.json,validate.ts}`, `apps/desktop/src/{lib/stores,routes/*}`, `docs/*`.

## Verification
- `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- `pnpm -r check && pnpm -r lint && pnpm -r test` (svelte-check, eslint, vitest)
- Negative tests: hostile FTP banners/oversized listings, truncated UDP replies, path traversal corpus, malformed theme JSON, backup tampering, companion protocol version mismatch.
- E2E: Playwright against Vite dev + mock-console covering first-run, offline demo, sim connect, theme edit/export, backup/restore.
- Run `cargo tauri dev` and screenshot on this KDE/Wayland box; GNOME/X11 untested here → stated in docs.
- `cargo tauri build --bundles deb`; inspect with `dpkg-deb -c/-I`; install test only with user permission (sudo).
- Real PS4 never touched; all docs say "simulator-tested".

## Needs from user before/while executing
- OK to `cargo install tauri-cli cargo-deb` (network, user-space) — assume yes unless you object.
- Optional later: `sudo apt install patchelf` for AppImage; chiaki-ng install (Flatpak/AUR-style, not in Debian) for a real Remote Play smoke test.
- Confirm GPL-3.0-or-later (decision 2).
- Scale warning: this is multiple sessions of work; plan executes P0→P8 sequentially, committing per phase, and `docs/current-status.md` is the resume point.

