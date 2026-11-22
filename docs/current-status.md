# Current status

All phases P0–P8 are implemented to the level the dev environment allows. **Release prep complete; not yet published, and nothing has been validated on real hardware.**

Works (simulator-verified): discovery, Remote Play launcher, diagnostics, Dashboard Studio (+ send to companion), File Manager (read-only default, plan/confirm/apply, backups before overwrite), Homebrew inventory/checksums/guarded copy, Backups, companion pairing + theme transfer, Console UI simulation, first-run welcome, notes.
Artifacts: `.deb` builds via `cd apps/desktop/src-tauri && cargo tauri build --bundles deb` (4.65 MiB).
Quality: 83 cargo tests (incl. real-ACL Tauri IPC flow), C host protocol test, 31 vitest, 26 Playwright; clippy/eslint/svelte-check/shellcheck/shfmt clean; cargo-deny ok; cargo-audit: 0 vulnerabilities, 3 transitive warnings.
Open items: see the P8 entry in docs/tasks.md ("NOT done / limits" and "Next").
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
