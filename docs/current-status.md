# Current status

Phase: P4 done; next P5 (homebrew FTP adapter, file manager, backups).
Works: everything from P3 plus Dashboard Studio (offline): 4 templates, editor, 1080p preview with keyboard/gamepad nav, save/import/export themes as schema-validated JSON files in XDG data dir.
Not done: file manager/FTP, homebrew manager, backups, companion, alt-OS docs, packaging/release docs.
Quality: 41 cargo tests, 28+3 vitest, 16 Playwright, clippy/eslint/svelte-check clean.
Nothing tested against a real console. chiaki-ng/adb/patchelf not installed on dev box.
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
