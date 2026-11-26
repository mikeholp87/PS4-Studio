# Tasks

## Phases
- [x] P0 Bootstrap
- [x] P1 Research docs (4 docs written; open items listed in research.md)
- [x] P2 Desktop foundation
- [x] P3 Connection system
- [x] P4 Dashboard Studio
- [x] P5 Homebrew / files / backups
- [x] P6 Companion (protocol + simulator verified; console app is an untested skeleton)
- [x] P7 Alt-OS research + simulation
- [x] P8 Release engineering (release prep complete; publishing/hardware validation pending)

## Done log
### 2026-11-23 — Remove debug build artifacts

- Removed `target/debug` at user request; updated `docs/current-status.md`.
- Verification: no Cargo/Rust compiler process found; directory absence check passed.
- Limits: no build or tests run for this artifact cleanup.
- Next: Cargo will regenerate debug artifacts on the next debug build.

### 2026-11-22 — Claude Code context recovered into Codex

- Recovered the original 27-section user brief and approved P0–P8 plan into
  `docs/continuity/original-brief.md` and `docs/continuity/claude-approved-plan.md`.
- Added `docs/continuity/session-resume.md`; updated `docs/current-status.md`.
- Inspected the staged skeleton and actual source files. Bootstrap is incomplete;
  feature modules and simulator are placeholders, and UI is still the Vite starter.
- Validation: desktop Svelte/TypeScript check passed; Svelte reported 0 errors/warnings.
  `cargo check --workspace --locked` passed.
- Limitation: no real-console testing, app launch, package build, or full test suite run.
  Historical upstream claims in the recovered plan require verification before integration.
- Next: finish P0 validation/tooling, then research (P1), then desktop foundation (P2).

### P0 Bootstrap — done 2026-11-22
- Implemented: git repo, Cargo workspace (7 crates + tauri shell), Vite+Svelte5 app, minimal Tauri conf/capabilities, agent docs.
- Files: Cargo.toml, pnpm-workspace.yaml, AGENTS.md, CLAUDE.md, apps/desktop/**, crates/**, simulator/mock-console/**
- Tests: `cargo check --workspace` OK (275 crates); `pnpm -r check` 0 errors.
- Limits: Vite demo files still present; icon is flat-color placeholder-quality.
- Next: P1 (done below) → P2.

### P1 Research docs — done 2026-11-22
- Implemented: research.md, technical-feasibility.md, firmware-compatibility.md, dependencies-and-licenses.md.
- Evidence: gh API for repo licenses/releases; chiaki-ng discovery.c/discovery.h/main.cpp; GoldHEN README+release notes; PPPwn README; postmarketOS MR via web search.
- Key findings: SRCH wire format verified; chiaki-ng CLI secrets (`--registkey/--passcode`) must never be passed (argv leak); GoldHEN firmware range not asserted; Slim Linux unsupported per 2024 postmarketOS note.
- Limits: Sony Remote Play page, GoldHEN FTP auth/layout, OpenOrbis APIs not yet verified ([U] tags).
- Next: P2 foundation.

### P2 Desktop foundation — done 2026-11-22
- Implemented: `core` (typed capability engine w/ evidence+reasons, SQLite settings/profiles/audit, XDG paths, input validation); `mock-console` (14 labelled scenarios -> Facts); Tauri shell with 11 commands, each explicitly permitted (build.rs AppManifest + capabilities/default.json), settings allowlist; Svelte 5 UI: tokens (dark/light/system/accent), sidebar, header status pill + SIMULATED badge, Overview (real capability data, unavailable measurements), Connections (simulator picker, profile add/delete), Settings (theme, accent, activity log), explanatory pages for later sections, Alt+1..9 nav.
- Files: crates/core/src/*, simulator/mock-console/src/*, apps/desktop/src-tauri/{build.rs,src/*,capabilities/default.json}, apps/desktop/src/**, .github/workflows/ci.yml
- Tests: `cargo test` core 11 + mock 2 + shell 1 pass; clippy -D warnings clean; fmt clean; svelte-check 0 errors; eslint clean; vitest 3 pass. `cargo tauri dev` launched on KDE/Wayland, window rendered (screenshot checked), IPC settings read OK.
- Limits: manual profile "Connect" disabled until P3; no Playwright e2e yet (needs mockIPC); manual notes feature (spec s8) not done; GNOME/X11 untested; first-run flow not built; simulator has no network servers yet.
- Next: P3 connection (UDP discovery client + mock UDP responder test, connection test w/ timeout/retry, chiaki-ng detect+launch, Remote Control page, Diagnostics).

### P3 Connection system — done 2026-11-22
- Implemented: `discovery` crate (SRCH build, strict size-capped parser with sanitising, unicast probe w/ retries + source-IP check, user-initiated limited-broadcast search); `mock-console::serve_discovery` real UDP responder (offline = silent, timeout = late, labelled [SIMULATED], no invented firmware); `remote-play` crate (detect native/Flatpak chiaki-ng, validated LaunchSpec, argv builder proven free of --registkey/--morning/--passcode, output discarded, SIGTERM→kill stop); `diagnostics` crate (local + capability checks, sanitizer: IPv4 masking, MAC/hex ids, credential lines, $HOME); 7 new Tauri commands (each explicitly permitted); UI: Connect on saved consoles (real probe), LAN search, Remote Control page (setup guide, nickname, gated Start/Stop, polling only while running), Diagnostics page + sanitised export; Playwright e2e (10 tests) with mocked IPC and fixtures generated from the real Rust engine (scripts/gen-fixtures.sh).
- Files: crates/{discovery,remote-play,diagnostics}/**, simulator/mock-console/src/*, apps/desktop/src-tauri/src/commands.rs, apps/desktop/src/{lib/api.ts,routes/{Connections,RemoteControl,Diagnostics}.svelte}, apps/desktop/e2e/**, scripts/gen-fixtures.sh
- Tests: cargo test --workspace 34 pass (discovery 10 incl. hostile replies + simulator integration; remote-play 5; diagnostics 5; core 11; mock 2; shell 1); clippy -D warnings clean; fmt clean; svelte-check 0; eslint clean; vitest 3; Playwright 10 pass (needs `PW_CHROMIUM=<chromium path>` or `playwright install`). Real app launched on KDE/Wayland without errors.
- Bug found+fixed in testing: sanitizer skipped IP masking on text preceding a secret keyword.
- Limits: chiaki-ng not installed here -> Remote Play launch only tested with a fake binary; real discovery never run against a real PS4; broadcast search not run on this LAN; tauri commands themselves have no Rust integration tests (logic is in crates); e2e uses mocked IPC; remote wake not implemented (UNSUPPORTED); connect failure state shown via header "no reply" + Diagnostics.
- Next: P4 Dashboard Studio (theme-engine package: schema/validator/templates, editor, 1080p preview, import/export).

### P4 Dashboard Studio — done 2026-11-22
- Implemented: `packages/theme-engine` (single JSON Schema draft-07 `theme.schema.json`; ajv validator; 4 original templates MINIMAL/AURORA/GRID/CLASSIC; pure helpers: fromTemplate/newTheme/add/move/removeTile/moveFocus, proxy-safe clone); Rust `core::themes` (same schema via `jsonschema`, 256 KB cap, slug ids, atomic writes, list skips tampered files, collision-safe import); shared valid/invalid fixture corpus run by BOTH vitest and cargo test (3 valid / 14 invalid incl. CSS injection, URL fields, control chars, traversal ids); 5 new Tauri commands (list/save/delete/export/import) — file dialogs are opened from Rust so the webview never supplies a path; Studio UI: theme list, 1080p scaled TV preview (arrow keys, on-screen D-pad, Gamepad API polled only while a pad exists and the preview is focused/hovered, focus animation, focus-follow scrolling, clock + status(preview) widgets), editor (general, background solid/gradient, colours, font, widgets, navigation, tile add/reorder/remove), save/revert/reset/duplicate/export/import/delete with confirm.
- Files: packages/theme-engine/**, crates/core/src/themes.rs, apps/desktop/src-tauri/src/{commands.rs,lib.rs}, build.rs, capabilities, apps/desktop/src/{routes/Studio.svelte,lib/components/{DashboardPreview,ThemeEditor}.svelte}, e2e/*
- Tests: cargo 41 pass; theme-engine vitest 28; desktop vitest 3; Playwright 16 (adds 6 Studio tests); clippy/fmt/svelte-check/eslint clean; real app launched.
- Bug found by e2e + fixed: structuredClone throws on Svelte reactive proxies (add tile did nothing) -> JSON clone + regression test.
- Limits: image backgrounds unsupported (no URL/file refs by design); exports/imports via native dialog not exercised in automated tests (mock only; dialog code compiled but never run in CI); gamepad path untested with hardware; status widget values are placeholders labelled "preview"; deploying a theme to a PS4 companion is not implemented (needs P6); "profiles" = multiple saved theme files.
- Next: P5 (FTP adapter + file manager + homebrew list + backups + audit view).

### P5 Homebrew / files / backups — done 2026-11-22
- Implemented: `console-adapters` (path policy; hardened FTP client with caps, PASV-bounce protection, timeouts, poison-on-failure; safe ops INSPECT→PLAN→APPLY→VERIFY with pre-change backups, `.ps4studio-part` uploads, no-overwrite/no-exec downloads); simulator FTP server honouring scenarios (auth failure, refused writes, storage limit, interruption); `backups` crate (versioned manifest, SHA-256 verify, tamper-safe reads); Tauri commands for files/backups/homebrew with single-use expiring plans, write lock + acknowledgement, cancel, progress events; UI: File Manager (dual pane, lock/unlock, plan dialog, transfer list, cancel), Homebrew (local .pkg inventory, sidecar manifest, SHA-256 verify, trust list, warnings, copy via confirmed flow, history; installed list/install/plugins honestly unavailable), Backups (create/verify/details/restore settings-profiles-themes/save copy/delete, Sony backup guidance, operation history); docs/security.md, docs/connection-protocols.md.
- Tests: cargo 71 pass (incl. 9 FTP unit/hostile, 7 FTP-vs-simulator, 5 safe-ops, 5 backups, 3 homebrew, 1 real-ACL Tauri IPC end-to-end flow); vitest 28+3; Playwright 20; clippy/fmt/svelte-check/eslint clean.
- Bugs found+fixed on the way: capability looked verified before any probe in simulator sessions (now unprobed until the user connects); circular Overview gate on "Browse Files".
- Limits: never run against a real GoldHEN console; credentials session-only (no Secret Service yet); symlink caveat (see security.md); remote files not auto-restorable by design; no pause (service can't); transfer queue is a UI list of sequential operations; `dialog` plugin flows (export/import/pick-folder/save-as) compiled but not exercised in automated tests; installed-homebrew listing, package install trigger and plugin management remain UNSUPPORTED/EXPERIMENTAL per technical-feasibility.md.
- Next: P6 companion (C source + Rust protocol + mock) then P7 alt-OS docs, P8 release engineering.

### P6 Companion — done 2026-11-22 (desktop side verified; console side NOT verified)
- Implemented: `core::companion_proto` (versioned v1 framing, mutual HMAC-SHA256 pairing proofs, per-frame MACs with direction labels + sequence numbers, 400 KB frame cap checked before allocation, 7-command allowlist, 2 config keys); `console-adapters::companion::Client`; `mock-console::serve_companion` (one-time code, 3-attempt lockout, replay/forgery rejection, theme validation, honest diagnostics: no hardware numbers); Tauri `companion_connect/send_theme/disconnect`; Studio "Send to console…" dialog with explicit "authenticated, not encrypted" and "does not change the PS4 home screen" notices; portable C protocol core `apps/ps4-companion/src/proto.c` (SHA-256, HMAC, proofs, MACs, allowlists) with host tests; untested OpenOrbis skeleton `src/ps4_main.c` + Makefile + README. Simulator/companion capabilities now start unverified until the user pairs (same fix as FTP).
- Tests: Rust 82 pass (core proto 7 incl. cross-language vectors; companion-vs-simulator 6 incl. wrong code/lockout, version mismatch, rogue server (bad server proof, forged MAC, oversized length, silence), replay and forged-MAC rejection; real-ACL IPC flow extended with companion pairing/transfer/mismatch); C host test passes (`make host-test`, -Wall -Wextra -Werror); Playwright 21.
- Limits: `ps4_main.c` never compiled with OpenOrbis nor run on hardware (networking, secure random code, rendering, controller nav are TODO); companion port 9777 is our arbitrary choice; link is authenticated but not encrypted and the 8-digit code is brute-forceable offline by a passive LAN observer (PAKE not implemented); pairing is per-connection (nothing persisted); companion status/themes are not shown anywhere except the send dialog.
- Next: P7 alt-OS docs + console UI simulation, then P8 release engineering.

### P7 Alt-OS research + simulation — done 2026-11-22
- Implemented: docs/alternative-os-research.md (A/B/C feasibility matrix), hardware-compatibility.md, boot-and-recovery-risks.md (explicitly not a procedure), alternative-os-roadmap.md; Console UI simulation (Settings → Experimental): controller/keyboard navigation (Gamepad API), game launch overlay (simulated), media "now playing", file browser over an in-memory tree, network settings with simulated test, theme management (applies saved Dashboard Studio themes), system info panel (invented/"not measured" — no hardware numbers), settings toggles; permanent SIMULATION badge.
- Research correction (evidence-based): feeRnt/ps4-linux-12xx README lists Slim models (CUH-2215B, CUH-2216A) with kernels 5.4.247 / 5.15.15 (Aeolia/Belize/Baikal southbridges confirmed); earlier "Slim unsupported" claim derived from a postmarketOS packaging note was too strong and is fixed in research.md and technical-feasibility.md. (The P1 log entry above still shows the old wording; this note supersedes it.)
- Tests: Playwright 24 (3 new for the simulation); svelte-check/eslint clean.
- Limits: no hardware-specific facts beyond the cited README/MR; GPU/audio/power status for Slim remains [U]; simulation uses demo data only and is not a standalone kiosk app yet (roadmap stage 2).
- Next: P8 release engineering (README/LICENSE/etc., packaging .deb, CI hardening, license audit, first-run flow, perf measurements, screenshots).

### P8 Release engineering — done 2026-11-22
- Implemented: first-run welcome; manual console notes; README (+ real UI screenshots from simulator fixtures), LICENSE (GPL-3.0 text), CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG, docs/{architecture,installation,troubleshooting,development,performance,licenses}.md; license audit from real `cargo metadata`/`pnpm licenses` output + `deny.toml` (validated with cargo-deny: licenses/bans/sources ok); `cargo audit` found RUSTSEC-2026-0459 (libsqlite3-sys 0.35) → fixed by bumping rusqlite 0.37→0.43 (libsqlite3-sys 0.40); 3 remaining audit *warnings* are transitive from Tauri's GTK stack (chrono unmaintained, proc-macro-error unmaintained, glib 0.18 unsoundness in an API we don't call); Debian package built with Tauri (4.65 MiB, Depends: libsqlite3-0, libwebkit2gtk-4.1-0, libgtk-3-0; Maintainer, copyright file, desktop entry, icons); packaged binary extracted, linked-lib check (no missing libs) and launched; perf measured (docs/performance.md); CI: rust/frontend(+Playwright)/companion-C/shell scripts/security(deny+audit+pnpm audit)/deb jobs + tag-triggered release workflow (workflows YAML-validated, NOT executed on GitHub); shell scripts pass shellcheck+shfmt; packaging notes for AppImage/Flatpak (untested templates).
- Tests at release prep: cargo 83 pass; vitest 31; Playwright 26; C host test; clippy/fmt/eslint/svelte-check clean.
- NOT done / limits: no real PS4 testing of anything; `apt install` of the .deb not run on a clean machine (needs sudo); GNOME/X11 untested; AppImage/Flatpak not built; Android/Google TV (ADB) and HDMI-CEC optional module not implemented (deferred, documented as CONDITIONAL/EXPERIMENTAL in technical-feasibility.md); Secret Service storage not wired (no persisted secrets exist); GitHub Actions never run; repository URL placeholder `OWNER` in Cargo.toml/debian copyright; Maintainer email in the .deb is a placeholder noreply address; window-visible startup time not measured.
- Next (priority): 1) run against a real PS4 (discovery, chiaki-ng launch) and, if available, a homebrew console (FTP) and record results in docs; 2) set the real repo URL/maintainer, push, watch CI; 3) `sudo apt install` the .deb on a clean Debian KDE and GNOME VM; 4) build/test the OpenOrbis companion; 5) optional TV (ADB) module.

### FIX: packaged app showed a blank window — done 2026-11-23
- Symptom: installed `.deb` opened a window with only the dark background.
- Root cause: `ajv` compiled the theme JSON Schema with `new Function` at module load. The app's CSP (`default-src 'self'`, deliberately no `unsafe-eval`) blocked it, the main module threw, and Svelte never mounted. The Vite dev server has no CSP, so dev runs and all e2e tests passed. (Evidence: loaded the production bundle in Chromium under the exact CSP → `pageerror: Evaluating a string as JavaScript violates ... 'unsafe-eval'`.)
- Fix: `packages/theme-engine/scripts/gen-validator.mjs` precompiles the schema to plain JS (`src/validator.generated.js`, committed; Ajv's one `require()` helper inlined); `validate.ts` uses it; `ajv` is now a dev-only dependency. CSP unchanged. Bundle shrank 287 kB → 194 kB.
- Regression tests: unit tests that the generated file is up to date and contains no eval/new Function/require; Playwright `csp.spec.ts` serves the production build under the real CSP (`e2e/csp-server.mjs`) and requires a rendered UI with zero page errors. Verified the test FAILS against the old eval-based validator.
- Also added the `custom-protocol` cargo feature (standard Tauri requirement for release embedding). It was NOT the cause of the blank window (a port-1420 listener saw zero requests from the old binary) but is required practice.
- Verified: release binary rendered in an isolated Xvfb display (welcome dialog, sidebar, working IPC). Gates re-run.
- Lesson: previous "launched OK" checks only proved the process stayed alive; always look at the rendered window of the packaged build.

### FIX: audit follow-up (settings allowlist, companion sync, TOCTOU, UI) — done 2026-11-26
- Fixed: `get_setting` now shares allowlist with `set_setting` (incl. notes/ports/publishers); per-console notes keys `console_notes:profile.*` / `console_notes:sim.*`; FTP reset clears companion facts; apply re-inspects overwrite at APPLY + cancelable backup download + part cleanup; plan/apply refuse when writable already false; first success only sets writable if previously None; File Manager local list race guard; Connections busy lock + remove confirm + UDP-search copy; ConsoleSim timeout tracking; Overview Browse Files gated.
- Files: `commands.rs`, `backups_cmd.rs`, `files.rs`, `ops.rs` (+ test), Connections/Overview/FileManager/ConsoleSim routes, docs.
- Tests: cargo test --workspace 87 pass; vitest 31+3; clippy/eslint/svelte-check clean.
- Next: real PS4 Connect/chiaki; CI on GitHub; remaining hardening from audit backlog.

### AUDIT: full codebase bug pass — done 2026-11-26
- Trigger: full audit (gates + Rust/UI review). Gates were already green; audit found lifecycle and data-safety bugs tests did not cover.
- Fixed: `connect_profile` now resets FTP/companion state (was leaking prior session); FileState `reset`/`reset_async` split (no tokio `blocking_lock` on the async runtime); overwrite uploads use rename-swap via `.ps4studio-bak` instead of DELE-before-RNTO; apply failure keeps local recovery temp and no longer claims “nothing further was changed”; backup promote failures keep tmp and do not imply rollback; `companion_send_theme` requires `CompanionThemeManagement` Available; Remote Play TCP fallback records `HostState::Unknown` (not Ready); `hb_upload_op` enforces verify-or-ack server-side; UI: connection refresh/disconnect no longer wipe on IPC failure; File Manager listing/download races; `moveFocus` cols≤0 guard; ConsoleSim media timer stacking; gamepad disconnect stops poll.
- Files: `crates/{core,console-adapters}`, `apps/desktop/src-tauri/src/{commands,files,homebrew}.rs`, `apps/desktop/src/{lib,routes,components}`, `packages/theme-engine`, `simulator/mock-console`, `docs/{security,troubleshooting}.md`.
- Tests: `cargo test --workspace` 86 pass; clippy -D warnings clean; vitest 31+3; svelte-check/eslint clean; `make host-test` pass. Playwright not re-run this pass.
- Limits: no real-console retest; remaining medium UI risks (notes are global, Connections multi-click) left as-is.
- Next: real PS4 connect flow; optional Playwright smoke after UI race fixes.

### FIX: real PS4 not found by the app — done 2026-11-23 (first real-hardware observation)
- Symptom: user's powered-on PS4 was not found. A manual scan showed: no UDP 987 reply to broadcast or unicast (several source/versions), no Sony MAC in ARP, BUT one host (AzureWave Wi-Fi MAC) answered ping and had TCP 9295 open; an unauthenticated HTTP request there returned Sony's Remote Play signature (`RP-Version`, `RP-Application-Reason`). So the console was reachable; only the UDP search was ignored.
- Fix: `discovery::probe_remote_play` + `DiscoveryVia` fact; Connect on a saved console falls back to it. Hostile-input tests (non-HTTP banner, wrong service, endless data, silence, closed port). Verified on the real console with a throwaway harness (service recognised; a non-PS4 host correctly rejected).
- Limits: the broadcast "Search local network" button still relies on UDP only, so it finds nothing on this console; add by IP. Name/state/firmware are unknown on this path. Root cause of the ignored UDP search NOT determined (local ufw is active and could drop broadcast replies; untested because it needs root). Remote Play launch itself (chiaki-ng) still untested.
- Next: confirm the app flow end-to-end (add 192.168.0.239 → Connect), install chiaki-ng and try a real Remote Play session.
