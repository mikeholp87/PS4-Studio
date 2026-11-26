# Current status

Audit follow-up (2026-11-26): settings get/set allowlist unified; per-console notes; companion facts cleared when FTP reset drops the socket; APPLY re-inspects overwrite TOCTOU; Connections/File Manager race guards. See tasks.md FIX entry.

All phases P0–P8 are implemented to the level the dev environment allows. **Release prep complete; not yet published, and nothing has been validated on real hardware.**

Works (simulator-verified): discovery, Remote Play launcher, diagnostics, Dashboard Studio (+ send to companion), File Manager (read-only default, plan/confirm/apply, backups before overwrite), Homebrew inventory/checksums/guarded copy, Backups, companion pairing + theme transfer, Console UI simulation, first-run welcome, notes.
Artifacts: `.deb` builds via `cd apps/desktop/src-tauri && cargo tauri build --bundles deb` (4.65 MiB).
Quality: 87 cargo tests, 34 vitest, clippy/eslint/svelte-check clean (Playwright not re-run this pass).
Open items: see the P8 entry in docs/tasks.md ("NOT done / limits" and "Next").
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
