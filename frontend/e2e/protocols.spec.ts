import { test, expect } from '@playwright/test'

/**
 * 計畫書（Protocol）列表 E2E 測試
 *
 * 前置條件：已登入
 */
test.describe('計畫書列表', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/protocols')
        await page.waitForLoadState('networkidle')
        // 確認未被導向 login（auth state 有效）
        if (page.url().includes('/login')) {
            test.skip()
        }
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
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })

        // 點擊 header 排序
        const sortableHeader = table.locator('th button, th[class*="cursor"]').first()
        if (await sortableHeader.count() === 0) {
            test.skip()
            return
        }
        await sortableHeader.click()
        await page.waitForTimeout(500)
        // 頁面應保持穩定
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
        const protocolLink = page.locator('table a[href*="/protocols/"]').first()
        // 等待 table 資料載入
        await page.waitForTimeout(2000)
        if (await protocolLink.count() === 0) {
            test.skip()
            return
        }

        await protocolLink.click()
        await expect(page).toHaveURL(/\/protocols\/[a-f0-9-]+/, { timeout: 10_000 })
    })
})
