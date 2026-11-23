import { defineConfig } from '@playwright/test'

// UI-level e2e against Vite with a mocked Tauri IPC (see e2e/mock-ipc.ts). Capability tables in the
// fixtures are generated from the real Rust engine (scripts/gen-fixtures.sh); Rust behaviour itself
// is covered by cargo tests.
export default defineConfig({
  testDir: 'e2e',
  fullyParallel: true,
  workers: 4,
  retries: process.env.CI ? 1 : 0,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:1420',
    viewport: { width: 1280, height: 800 },
    // Set PW_CHROMIUM to use an already-installed Chromium instead of `playwright install`.
    launchOptions: { executablePath: process.env.PW_CHROMIUM || undefined },
  },
  webServer: [
    { command: 'pnpm dev', url: 'http://127.0.0.1:1420', reuseExistingServer: true, timeout: 60_000 },
    // production build under the app's real CSP (see e2e/csp-server.mjs)
    { command: 'pnpm build && node e2e/csp-server.mjs', url: 'http://127.0.0.1:4173', reuseExistingServer: true, timeout: 90_000 },
  ],
})
