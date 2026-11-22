# Tasks

## Phases
- [ ] P0 Bootstrap (in progress)
- [x] P1 Research docs (4 docs written; open items listed in research.md)
- [x] P2 Desktop foundation
- [ ] P3 Connection system
- [ ] P4 Dashboard Studio
- [ ] P5 Homebrew / files / backups
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
