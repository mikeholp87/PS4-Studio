# Current status

Phase: P3 done; next P4 (Dashboard Studio).
Works: app launches; offline simulator (14 scenarios) incl. a real UDP responder; LAN search + saved-console probe (unicast 987); capability table with reasons; chiaki-ng detect/launch/stop (fake-binary tested); diagnostics + sanitised export; theme/accent; activity log; 34 Rust tests, 3 vitest, 10 Playwright.
Not done: Dashboard Studio, file manager/FTP, homebrew, backups, companion, alt-OS docs, packaging/release docs.
Nothing tested against a real console. chiaki-ng/adb/patchelf not installed on dev box.
Run e2e: `cd apps/desktop && PW_CHROMIUM=$HOME/.cache/ms-playwright/chromium-1254/chrome-linux64/chrome pnpm e2e`
