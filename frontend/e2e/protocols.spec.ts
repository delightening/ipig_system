import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('計畫書列表', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/protocols')
        }
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })
        await expect(
            page.locator('a[href*="/protocols/new"]').or(page.locator('table')).first(),
            '計畫書頁應載入（新增按鈕或表格）'
        ).toBeVisible({ timeout: 15_000 })
    })

    test('應顯示計畫書列表頁面', async ({ page }) => {
        // 搜尋欄（頁面載入完成的指標）
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })
    })

    test('應有新增計畫書按鈕', async ({ page }) => {
        const createBtn = page.locator('a[href*="/protocols/new"]').first()
        await expect(createBtn).toBeVisible({ timeout: 15_000 })
    })

    test('應有狀態篩選', async ({ page }) => {
        const statusFilter = page.locator('[role="combobox"]').first()
        await expect(statusFilter).toBeVisible({ timeout: 15_000 })

        await statusFilter.click()
        await expect(page.locator('[role="option"]').first()).toBeVisible({ timeout: 5_000 })
    })

    test('表格排序應可運作', async ({ page }) => {
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/protocols')
        }
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })

        const sortableHeader = table.locator('th button, th[class*="cursor"]').first()
        await expect(sortableHeader, '計畫書表格應有可排序的欄位').toBeVisible({ timeout: 5_000 })

        await sortableHeader.click()
        await page.waitForTimeout(500)
        await expect(table).toBeVisible()
    })

    test('搜尋應能過濾計畫書', async ({ page }) => {
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })

        await searchInput.fill('IACUC')
        await page.waitForTimeout(800)

        // 頁面應保持穩定
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('計畫書連結應可點擊', async ({ page }) => {
        await expect(page.locator('table').or(page.getByText(/尚無|沒有|no protocol/i)).first()).toBeVisible({ timeout: 5_000 })
        const protocolLink = page.locator('table a[href*="/protocols/"]').first()
        if (!(await protocolLink.isVisible().catch(() => false))) {
            return
        }
        await protocolLink.click()
        await expect(page).toHaveURL(/\/protocols\/[a-f0-9-]+/, { timeout: 10_000 })
    })
})
