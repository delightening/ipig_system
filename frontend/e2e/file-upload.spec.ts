import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('附件與詳情頁面', () => {
    test('計畫書列表頁面應顯示', async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await expect(page).toHaveURL(/\/protocols/, { timeout: 12_000 })

        // 頁面應有表格、loading skeleton、或空狀態
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no protocol/i)
        const pageTitle = page.getByText(/計畫書|Protocol|實驗計畫/i)
        await expect(table.or(empty).or(pageTitle).first()).toBeVisible({ timeout: 15_000 })
    })

    test('計畫書詳情頁面應有附件 Tab', async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await expect(page).toHaveURL(/\/protocols/, { timeout: 12_000 })

        // 等待表格或空狀態
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no protocol/i)
        const pageTitle = page.getByText(/計畫書|Protocol|實驗計畫/i)
        await expect(table.or(empty).or(pageTitle).first()).toBeVisible({ timeout: 15_000 })

        // 確認有資料列可點擊
        const rows = table.locator('tbody tr')
        const rowCount = await rows.count().catch(() => 0)
        if (rowCount === 0) {
            test.skip(true, '無計畫書資料，跳過附件測試')
            return
        }

        const firstRow = rows.first()
        const link = firstRow.locator('a').first()
        if (await link.isVisible().catch(() => false)) {
            await link.click()
        } else {
            await firstRow.click()
        }
        await page.waitForTimeout(2000)

        // 計畫書詳情頁有 Attachments / 附件 tab
        const attachTab = page.getByText(/附件|Attachments/i)
        await expect(attachTab.first()).toBeVisible({ timeout: 15_000 })
    })

    test('動物列表頁面應顯示', async ({ page }) => {
        await ensureAdminOnPage(page, '/animals')
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })

        // 動物頁面應有表格、empty state、或 loading skeleton
        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|no animal|尚無|新增/i)
        const pageTitle = page.getByText(/動物|Animal/i)
        await expect(table.or(emptyState).or(pageTitle).first()).toBeVisible({ timeout: 15_000 })
    })
})
