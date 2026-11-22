# Tasks

## Phases
- [ ] P0 Bootstrap (in progress)
- [x] P1 Research docs (4 docs written; open items listed in research.md)
- [x] P2 Desktop foundation
- [x] P3 Connection system
- [x] P4 Dashboard Studio
- [x] P5 Homebrew / files / backups
- [ ] P6 Companion
- [ ] P7 Alt-OS research + simulation
- [ ] P8 Release engineering

## Done log
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
