# AGENTS.md — PS4 Studio (for Codex, Claude Code, any agent)

Single source of truth for process: this file + `docs/`. `CLAUDE.md` only adds Claude-specific notes.
**Resume point: read `docs/current-status.md`, then `docs/tasks.md`, then `docs/decisions.md`.**

## Goal
Debian desktop app (Tauri 2 + Svelte 5 + Rust) to manage a PS4 over LAN: discovery, Remote Play launch
(via installed chiaki-ng), capability-gated homebrew/file features, offline Dashboard Studio,
backups, diagnostics, simulator. Independent project, not endorsed by Sony.

## Layout
- `apps/desktop` Svelte UI; `apps/desktop/src-tauri` Tauri shell (thin: commands only)
- `crates/core` capability model, safe-op state machine, SQLite, audit log, paths
- `crates/discovery` UDP 987 discovery/wake · `crates/remote-play` chiaki-ng launcher
- `crates/console-adapters` stock probe, GoldHEN FTP, companion protocol
- `crates/backups`, `crates/diagnostics` · `simulator/mock-console` fake PS4 (always `simulated:true`)
- `packages/theme-engine` theme schema/validator/templates · `apps/ps4-companion` C source (OpenOrbis, not built in CI)

## Rules
- Capabilities are verified by probes, never inferred from an open port. Unknown = unavailable, with a reason string.
- Safe-op flow for any console write: INSPECT → PLAN → CONFIRM → APPLY → VERIFY → RECOVER; audit-log every APPLY.
- Read-only by default; writes need explicit unlock; path allowlist; reject `..`/escapes; never execute downloads.
- Every network op has a timeout. No credentials/tokens in logs. Secrets only via Secret Service (`keyring`).
- No fake data presented as real. Simulator output is labelled. Never claim physical-console testing unless done.
- Do not embed chiaki-ng (AGPL); launch it as a separate process.
- No root needed at runtime. Ask before sudo, firmware/boot work, or anything touching a real console.

## Commands
```
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
pnpm -r check && pnpm -r lint && pnpm -r test
cd apps/desktop && cargo tauri dev        # run app
cd apps/desktop && cargo tauri build --bundles deb
```

## Handoff protocol
After each task add an entry to `docs/tasks.md` (done section): what, files, tests run + results,
limitations, recommended next. Update `docs/current-status.md`. Record decisions in `docs/decisions.md`.
Small, focused commits.
