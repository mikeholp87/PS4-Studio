import { expect, test } from '@playwright/test'
import { mockTauri } from './mock-ipc'

// Regression: the packaged app rendered a blank window because a dependency used eval, which the
// app's CSP forbids. This runs the production bundle under that exact CSP.
test('production bundle renders under the real CSP with no violations or errors', async ({ page }) => {
  const problems: string[] = []
  page.on('pageerror', (e) => problems.push(`pageerror: ${e.message}`))
  page.on('console', (m) => {
    if (m.type() === 'error') problems.push(`console: ${m.text()}`)
  })
  await mockTauri(page)
  await page.goto('http://127.0.0.1:4173/')
  await expect(page.getByRole('heading', { name: 'No console selected' })).toBeVisible()
  await page.getByRole('navigation').getByRole('button', { name: 'Dashboard Studio' }).click()
  await expect(page.getByRole('application', { name: /Dashboard preview/ })).toBeVisible()
  expect(problems).toEqual([])
})
