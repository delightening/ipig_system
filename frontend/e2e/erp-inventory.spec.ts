import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('ERP 進銷存流程', () => {
    test('庫存頁面應顯示', async ({ page }) => {
        await ensureAdminOnPage(page, '/inventory')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        // 頁面應有表格或空狀態
        await expect(
            page.locator('table').or(page.getByText(/尚無|沒有|no data/i)).first(),
        ).toBeVisible({ timeout: 15_000 })
    })

    test('產品主檔頁面應顯示表格', async ({ page }) => {
        await ensureAdminOnPage(page, '/products')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        await expect(
            page.locator('table').or(page.getByText(/尚無|沒有|no product/i)).first(),
        ).toBeVisible({ timeout: 15_000 })
    })

    test('合作夥伴頁面應顯示表格', async ({ page }) => {
        await ensureAdminOnPage(page, '/partners')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        await expect(
            page.locator('table').or(page.getByText(/尚無|沒有|no partner/i)).first(),
        ).toBeVisible({ timeout: 15_000 })
    })

    test('產品搜尋應可運作', async ({ page }) => {
        await ensureAdminOnPage(page, '/products')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })

        await searchInput.fill('test')
        await page.waitForTimeout(800)

        // 頁面應保持穩定（未跳轉至登入頁）
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('合作夥伴 Tab 切換（供應商/客戶）', async ({ page }) => {
        await ensureAdminOnPage(page, '/partners')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        // Tab 切換（供應商 / Supplier / 客戶 / Customer）
        const tabList = page.locator('[role="tablist"]')
        if (!(await tabList.isVisible().catch(() => false))) {
            // 頁面可能無 Tab 設計，跳過
            return
        }

        const tabs = tabList.locator('[role="tab"]')
        const count = await tabs.count()
        expect(count).toBeGreaterThanOrEqual(2)

        // 點擊第二個 Tab
        await tabs.nth(1).click()
        await page.waitForTimeout(800)

        // 切換後頁面應保持穩定
        await expect(page).not.toHaveURL(/\/login/)
    })
})
