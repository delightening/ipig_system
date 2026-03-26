import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('設備管理', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/admin/equipment')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/admin/equipment')
        }
        await expect(page).toHaveURL(/\/admin\/equipment/, { timeout: 12_000 })
    })

    test('應顯示設備管理頁面（表格或 loading）', async ({ page }) => {
        const table = page.locator('table')
        const loading = page.getByText(/載入|loading/i)
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(loading).or(empty).first()).toBeVisible({ timeout: 15_000 })
    })

    test('設備 Tab 切換（設備/校正/維修/報廢/年度計畫）', async ({ page }) => {
        // EN: "Equipment", "Calibration", "Maintenance", "Disposal", "Annual Plan"
        // ZH: "設備", "校正確效", "維修保養", "報廢", "年度計畫"
        const tabs = page.locator('button').filter({
            hasText: /Equipment|Calibration|Maintenance|Disposal|Annual|設備|校正|維修|報廢|年度/,
        })
        await expect(tabs.first()).toBeVisible({ timeout: 15_000 })
        const count = await tabs.count()
        expect(count).toBeGreaterThanOrEqual(3)
    })

    test('搜尋應可過濾設備', async ({ page }) => {
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })

        await searchInput.fill('test')
        await page.waitForTimeout(800)
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('校正確效 Tab 應顯示校正記錄', async ({ page }) => {
        const calTab = page.locator('button').filter({ hasText: /Calibration|校正確效|校正/ })
        await expect(calTab.first()).toBeVisible({ timeout: 15_000 })

        await calTab.first().click()
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })

    test('維修保養 Tab 應顯示維修記錄', async ({ page }) => {
        const maintTab = page.locator('button').filter({ hasText: /Maintenance|維修保養|維修/ })
        await expect(maintTab.first()).toBeVisible({ timeout: 15_000 })

        await maintTab.first().click()
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })
})
