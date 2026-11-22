# Current status

Phase: P7 done; next P8 (release engineering).
Works (simulator-verified): discovery, Remote Play launcher, diagnostics, Dashboard Studio (+ send to companion), File Manager, Homebrew inventory/checksums/guarded copy, Backups, companion pairing + theme transfer, Console UI simulation, alt-OS research docs.
Quality: 82 cargo tests, C host protocol test, 31 vitest, 24 Playwright; clippy/eslint/svelte-check clean.
Not done: packaging (.deb/AppImage/Flatpak), README/LICENSE/CONTRIBUTING/CoC/SECURITY/CHANGELOG, CI hardening (deny/audit/deb build), license audit with real tool output, first-run flow, manual console notes, TV (ADB) module, perf numbers, screenshots in README.
Never tested on real hardware. PS4 companion console app is an untested skeleton. chiaki-ng/adb/patchelf not installed on dev box.
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
