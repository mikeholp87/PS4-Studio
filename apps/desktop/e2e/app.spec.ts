import { expect, test } from '@playwright/test'
import { mockTauri } from './mock-ipc'

test.beforeEach(async ({ page }) => {
  await mockTauri(page)
  await page.goto('/')
})

const nav = (page: import('@playwright/test').Page, name: string) => page.getByRole('navigation').getByRole('button', { name })

test('first run works offline with no console', async ({ page }) => {
  await expect(page.getByRole('heading', { name: 'No console selected' })).toBeVisible()
  await expect(page.getByRole('status')).toHaveText(/Not connected/)
  await nav(page, 'Dashboard Studio').click()
  await expect(page.getByRole('application', { name: /Dashboard preview/ })).toBeVisible()
})

test('simulator: scenario is labelled SIMULATED and file service starts unverified', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByLabel('Scenario').selectOption('homebrew')
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
  await expect(page.getByText('SIMULATED', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Browse Files' })).toBeEnabled()
  await expect(page.getByText('Firmware compatibility has not been established.')).toBeVisible()
  await expect(page.getByText('File service has not been checked yet.')).toBeVisible()
})

test('offline scenario explains that the console was not found', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByLabel('Scenario').selectOption('offline')
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
  await expect(page.getByRole('status')).toHaveText(/no reply/)
  await expect(page.getByText('Console not found. Confirm that the Debian computer').first()).toBeVisible()
})

test('profiles: validation error, add, delete', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByLabel('Name').fill('Den PS4')
  await page.getByLabel('IP address').fill('nope')
  await page.getByRole('button', { name: 'Save console' }).click()
  await expect(page.getByRole('alert')).toContainText('Address must be an IP address')
  await page.getByLabel('IP address').fill('192.168.1.50')
  await page.getByRole('button', { name: 'Save console' }).click()
  await expect(page.getByRole('cell', { name: 'Den PS4' })).toBeVisible()
  await page.getByRole('button', { name: 'Remove' }).click()
  await expect(page.getByText('No saved consoles yet.')).toBeVisible()
})

test('discovery results can prefill the add form', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByRole('button', { name: 'Search local network' }).click()
  await page.getByRole('button', { name: 'Use' }).click()
  await expect(page.getByLabel('IP address')).toHaveValue('192.168.1.50')
  await expect(page.getByLabel('Firmware (optional, as shown on the PS4)')).toHaveValue('9.00')
})

test('remote control: gated without chiaki-ng, explains why', async ({ page }) => {
  await nav(page, 'Remote Control').click()
  await expect(page.getByText('Not found').first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Start Remote Play' })).toBeDisabled()
  await expect(page.getByText('chiaki-ng is not installed.').last()).toBeVisible()
})

test('theme switch applies and persists through settings', async ({ page }) => {
  await nav(page, 'Settings').click()
  await page.getByLabel('Theme').selectOption('light')
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
})

test('diagnostics lists checks and exports a report path', async ({ page }) => {
  await nav(page, 'Diagnostics').click()
  await expect(page.getByText('chiaki-ng (Remote Play)')).toBeVisible()
  await page.getByRole('button', { name: 'Export sanitised report' }).click()
  await expect(page.getByText('diagnostics-1.txt')).toBeVisible()
})

test('keyboard shortcut Alt+3 opens Remote Control', async ({ page }) => {
  await page.keyboard.press('Alt+3')
  await expect(page.getByRole('heading', { name: 'Remote Control' })).toBeVisible()
})

test.describe('Dashboard Studio (offline, no console)', () => {
  test.beforeEach(async ({ page }) => { await nav(page, 'Dashboard Studio').click() })

  test('keyboard moves focus in the preview and select is preview-only', async ({ page }) => {
    const tv = page.getByRole('application', { name: /Dashboard preview/ })
    await tv.focus()
    await page.keyboard.press('ArrowRight')
    await page.keyboard.press('ArrowRight')
    await expect(tv.locator('.tile.focused .label')).toHaveText('Music')
    await page.keyboard.press('Enter')
    await expect(page.getByText('Nothing is launched.')).toBeVisible()
  })

  test('edit, save, reload list, export, import, delete', async ({ page }) => {
    await page.getByRole('button', { name: 'New from template' }).click()
    await page.getByLabel('Name', { exact: true }).fill('Evening')
    await page.getByRole('button', { name: 'Save', exact: true }).click()
    await expect(page.getByRole('status').filter({ hasText: 'Saved.' })).toBeVisible()
    await expect(page.getByRole('button', { name: /^Evening/ })).toBeVisible()
    await page.getByRole('button', { name: 'Export…' }).click()
    await expect(page.getByRole('status').filter({ hasText: 'Exported to' })).toBeVisible()
    await page.getByRole('button', { name: 'Import theme…' }).click()
    await expect(page.getByRole('button', { name: /^Imported theme/ })).toBeVisible()
    page.once('dialog', (d) => d.accept())
    await page.getByRole('button', { name: 'Delete' }).click()
    await expect(page.getByRole('button', { name: /^Imported theme/ })).toHaveCount(0)
  })

  test('template switch and tile editing update the live preview', async ({ page }) => {
    await page.getByLabel('Layout template').selectOption('grid')
    await expect(page.locator('.screen.grid')).toBeVisible()
    await page.getByLabel('New shortcut label').fill('Netflix-like')
    await page.getByRole('button', { name: 'Add shortcut' }).click()
    await expect(page.locator('.screen .tile')).toHaveCount(9)
    await page.getByRole('button', { name: 'Remove Netflix-like' }).click()
    await expect(page.locator('.screen .tile')).toHaveCount(8)
  })

  test('invalid edits block saving with a visible reason', async ({ page }) => {
    await page.getByLabel('Name', { exact: true }).fill('')
    await expect(page.getByRole('alert')).toContainText('Cannot save')
    await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled()
  })

  test('reset restores template defaults', async ({ page }) => {
    await page.getByLabel('Layout template').selectOption('grid')
    await page.getByLabel('Tile shape').selectOption('circle')
    await page.getByRole('button', { name: 'Reset to template' }).click()
    await expect(page.getByLabel('Tile shape')).toHaveValue('square')
  })
})

test('screenshots of all templates (visual review artefacts)', async ({ page }) => {
  await nav(page, 'Dashboard Studio').click()
  for (const t of ['minimal', 'aurora', 'grid', 'classic']) {
    await page.getByLabel('Layout template').selectOption(t)
    await page.getByRole('application', { name: /Dashboard preview/ }).focus()
    await page.keyboard.press('ArrowRight')
    await page.screenshot({ path: `test-results/studio-${t}.png` })
  }
})

async function simulate(page: import('@playwright/test').Page, scenario: string) {
  await nav(page, 'Connections').click()
  await page.getByLabel('Scenario').selectOption(scenario)
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
}

test.describe('File Manager', () => {
  test('stock console: no file service is detected and the reason is shown', async ({ page }) => {
    await simulate(page, 'stock')
    await nav(page, 'File Manager').click()
    await expect(page.getByRole('note')).toContainText('FTP is not encrypted')
    await page.getByRole('button', { name: 'Connect to file service' }).click()
    await expect(page.getByText('File management is unavailable because no supported file service was detected.').first()).toBeVisible()
    await expect(page.getByRole('button', { name: 'Download →' })).toBeDisabled()
  })

  test('read-only by default; unlocking needs acknowledgement; upload goes through plan and confirm', async ({ page }) => {
    await simulate(page, 'homebrew')
    await nav(page, 'File Manager').click()
    await page.getByRole('button', { name: 'Connect to file service' }).click()
    await expect(page.getByText('Read-only', { exact: true })).toBeVisible()
    await expect(page.getByRole('list', { name: 'Console files' }).getByText('data')).toBeVisible()
    await expect(page.getByRole('button', { name: '← Upload…' })).toBeDisabled()
    await expect(page.getByText('Writes are locked. Use “Enable changes…” first.').first()).toBeVisible()

    await page.getByRole('button', { name: 'Enable changes…' }).click()
    const enable = page.getByRole('button', { name: 'Enable changes', exact: true })
    await expect(enable).toBeDisabled()
    await page.getByLabel(/I understand FTP is not encrypted/).check()
    await enable.click()
    await expect(page.getByText('Changes enabled for this session')).toBeVisible()

    // system area is read-only even when unlocked
    await page.getByRole('list', { name: 'Console files' }).getByText('system').dblclick()
    await expect(page.getByText('read-only here')).toBeVisible()
    await page.getByRole('button', { name: 'Parent folder' }).nth(1).click()
    await page.getByRole('list', { name: 'Console files' }).getByText('data').dblclick()
    await page.getByRole('list', { name: 'Local files' }).getByText('notes.txt').click()
    await page.getByRole('button', { name: '← Upload…' }).click()

    const dlg = page.getByRole('dialog', { name: 'Review before changing the console' })
    await expect(dlg.getByText('What will change')).toBeVisible()
    await expect(dlg.getByText('FTP is not encrypted')).toBeVisible()
    await dlg.getByRole('button', { name: 'Confirm and apply' }).click()
    await expect(page.getByRole('dialog', { name: 'Done' }).getByText('Verified: the console shows the expected result.')).toBeVisible()
  })

  test('declining a plan changes nothing', async ({ page }) => {
    await simulate(page, 'homebrew')
    await nav(page, 'File Manager').click()
    await page.getByRole('button', { name: 'Connect to file service' }).click()
    await page.getByRole('button', { name: 'Enable changes…' }).click()
    await page.getByLabel(/I understand FTP is not encrypted/).check()
    await page.getByRole('button', { name: 'Enable changes', exact: true }).click()
    await page.getByRole('list', { name: 'Console files' }).getByText('data').dblclick()
    await page.getByRole('button', { name: 'New folder…' }).click()
    await page.getByLabel('Name', { exact: true }).fill('tools')
    await page.getByRole('button', { name: 'Review change' }).click()
    await page.getByRole('button', { name: 'Do not change anything' }).click()
    await expect(page.getByRole('dialog')).toHaveCount(0)
  })
})

test.describe('Homebrew and Backups', () => {
  test('packages warn about unfamiliar publishers; copying needs a connected, unlocked file service and a verified or accepted package', async ({ page }) => {
    await simulate(page, 'homebrew')
    await nav(page, 'Homebrew').click()
    await expect(page.getByText('PS4 Studio has no verified way to list homebrew installed on a console')).toBeVisible()
    await page.getByRole('button', { name: 'Choose folder…' }).click()
    await expect(page.getByText('Unfamiliar publisher “Acme”')).toBeVisible()
    await expect(page.getByText('No manifest (.json) next to this file')).toBeVisible()
    const send = page.getByRole('row', { name: /demo\.pkg/ }).getByRole('button', { name: 'Copy to console…' })
    await expect(send).toBeDisabled()
    await expect(page.getByText('File service has not been checked yet.').first()).toBeVisible()

    // connect + unlock in the File Manager, then come back
    await nav(page, 'File Manager').click()
    await page.getByRole('button', { name: 'Connect to file service' }).click()
    await page.getByRole('button', { name: 'Enable changes…' }).click()
    await page.getByLabel(/I understand FTP is not encrypted/).check()
    await page.getByRole('button', { name: 'Enable changes', exact: true }).click()
    await nav(page, 'Homebrew').click()
    await page.getByRole('button', { name: 'Choose folder…' }).click()
    const row = page.getByRole('row', { name: /demo\.pkg/ })
    await expect(row.getByRole('button', { name: 'Copy to console…' })).toBeDisabled()
    await row.getByRole('button', { name: 'Verify' }).click()
    await expect(page.getByText('Checksum matches the manifest.')).toBeVisible()
    await row.getByRole('button', { name: 'Copy to console…' }).click()
    const dlg = page.getByRole('dialog', { name: 'Review before changing the console' })
    await expect(dlg.getByText('/data/pkg/demo.pkg')).toBeVisible()
    await dlg.getByRole('button', { name: 'Confirm and apply' }).click()
    await expect(page.getByRole('dialog', { name: 'Done' })).toBeVisible()

    // a package with no manifest needs an explicit risk acceptance
    const bare = page.getByRole('row', { name: /bare\.pkg/ })
    await page.getByRole('button', { name: 'Close' }).click()
    await expect(bare.getByRole('button', { name: 'Copy to console…' })).toBeDisabled()
    await bare.getByLabel('I accept the risk').check()
    await expect(bare.getByRole('button', { name: 'Copy to console…' })).toBeEnabled()
  })

  test('backups: create, verify, restore guardrails, delete with confirmation', async ({ page }) => {
    await nav(page, 'Backups').click()
    await expect(page.getByText('Remote Play does not give that access')).toBeVisible()
    await page.getByLabel('Label (optional)').fill('Before update')
    await page.getByRole('button', { name: 'Create backup' }).click()
    await expect(page.getByRole('cell', { name: 'Before update' })).toBeVisible()
    await page.getByRole('button', { name: 'Verify' }).click()
    await expect(page.getByText('Verified: every item matches its checksum.')).toBeVisible()
    await page.getByRole('button', { name: 'Details' }).click()
    // console files cannot be restored automatically; settings can, after confirmation
    const items = page.locator('table.inner')
    await expect(items.getByRole('row', { name: /x\.cfg/ }).getByRole('button', { name: 'Restore' })).toHaveCount(0)
    await items.getByRole('row', { name: /settings\.json/ }).getByRole('button', { name: 'Restore' }).click()
    await page.getByRole('dialog').getByRole('button', { name: 'Restore' }).click()
    await expect(page.getByText('Restored 1 settings.')).toBeVisible()
    await page.getByRole('button', { name: 'Delete' }).click()
    await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click()
    await expect(page.getByText('No backups yet.')).toBeVisible()
  })
})
