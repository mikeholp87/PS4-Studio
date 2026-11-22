// Generates README screenshots: `SCREENSHOTS=1 pnpm e2e e2e/screenshots.spec.ts`.
// They show the real UI with the Tauri IPC mocked from simulator fixtures (nothing here is a real console).
import { expect, test } from '@playwright/test'
import { mockTauri } from './mock-ipc'

test.skip(!process.env.SCREENSHOTS, 'only when SCREENSHOTS=1')
test.use({ viewport: { width: 1280, height: 800 } })
const out = (n: string) => `../../docs/screenshots/${n}.png`

test('capture', async ({ page }) => {
  await mockTauri(page, { chiaki: true })
  await page.goto('/')
  const nav = (n: string) => page.getByRole('navigation').getByRole('button', { name: n })
  await nav('Connections').click()
  await page.getByLabel('Scenario').selectOption('homebrew')
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
  await expect(page.getByText('SIMULATED', { exact: true })).toBeVisible()
  await page.screenshot({ path: out('overview') })
  await nav('File Manager').click()
  await page.getByRole('button', { name: 'Connect to file service' }).click()
  await page.getByRole('list', { name: 'Console files' }).getByText('data').dblclick()
  await page.screenshot({ path: out('file-manager') })
  await nav('Remote Control').click()
  await page.screenshot({ path: out('remote-control') })
  await nav('Dashboard Studio').click()
  for (const t of ['aurora', 'grid']) {
    await page.getByLabel('Layout template').selectOption(t)
    await page.getByRole('application', { name: /Dashboard preview/ }).focus()
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(400)
    await page.screenshot({ path: out(`studio-${t}`) })
  }
  await nav('Diagnostics').click()
  await page.screenshot({ path: out('diagnostics') })
})
