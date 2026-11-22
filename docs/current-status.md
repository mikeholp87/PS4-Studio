# Current status

Phase: P6 done; next P7 (alt-OS research docs + console UI simulation), P8 (release engineering).
Works (simulator-verified): discovery, Remote Play launcher, diagnostics, Dashboard Studio (+send to companion), File Manager (read-only default, plan/confirm/apply, backups), Homebrew inventory/checksums/guarded copy, Backups, companion pairing + theme transfer.
Quality: 82 cargo tests (incl. real-ACL Tauri IPC flow), C host protocol test, 31 vitest, 21 Playwright; clippy/eslint/svelte-check clean.
Not done: alt-OS docs/simulation, packaging (.deb/AppImage/Flatpak), README/LICENSE/CONTRIBUTING/etc., CI hardening (deny/audit), first-run flow, manual console notes, TV (ADB) module, real screenshots.
Never tested on real hardware. PS4 companion console app is an untested skeleton. chiaki-ng/adb/patchelf not installed on dev box.
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
