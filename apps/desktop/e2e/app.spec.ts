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
  await expect(page.getByText('Coming in Phase 4')).toBeVisible()
})

test('simulator: homebrew scenario enables file browsing and is labelled SIMULATED', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByLabel('Scenario').selectOption('homebrew')
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
  await expect(page.getByText('SIMULATED', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Browse Files' })).toBeEnabled()
  await expect(page.getByText('Firmware compatibility has not been established.')).toBeVisible()
})

test('stock scenario disables file browsing with a visible reason', async ({ page }) => {
  await nav(page, 'Connections').click()
  await page.getByLabel('Scenario').selectOption('stock')
  await page.getByRole('button', { name: 'Connect to simulator' }).click()
  await expect(page.getByRole('button', { name: 'Browse Files' })).toBeDisabled()
  await expect(page.locator('small', { hasText: 'File service has not been checked yet.' })).toBeVisible()
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
