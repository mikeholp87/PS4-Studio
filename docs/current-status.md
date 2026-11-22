# Current status

Phase: P5 done; next P6 (companion protocol + C source), P7 (alt-OS docs + console UI simulation), P8 (release).
Works: offline simulator (14 scenarios, real UDP + FTP servers), discovery, Remote Play launcher, diagnostics, Dashboard Studio, File Manager (read-only default, plan/confirm/apply, backups before overwrite), Homebrew inventory + checksum + guarded copy, Backups.
Quality: 71 cargo tests (incl. real Tauri IPC flow with ACL), 31 vitest, 20 Playwright; clippy/eslint/svelte-check clean.
Not done: companion, alt-OS docs/simulation, packaging (.deb), README/LICENSE/CONTRIBUTING etc., CI hardening (deny/audit), first-run flow, manual console notes, TV module.
Nothing tested against a real console. chiaki-ng/adb/patchelf not installed on dev box.
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
