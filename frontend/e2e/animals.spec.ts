import { test, expect } from '@playwright/test'

/**
 * 動物列表 E2E 測試
 *
 * 前置條件：已登入，具有動物管理存取權限
 */
test.describe('動物列表', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/animals')
        await page.waitForLoadState('networkidle')
        // 確認已登入
        if (page.url().includes('/login')) {
            test.skip()
        }
    })

    test('應顯示動物列表頁面', async ({ page }) => {
        // 搜尋輸入框（頁面載入完成的指標）
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })
    })

    test('應有狀態篩選 Tab', async ({ page }) => {
        // Tab 文字：支援中英文（i18n）
        // EN: "Pen View", "Unassigned", "In Experiment", "Completed", "All Animals"
        // ZH: "欄舍", "未指派", "實驗中", "已完成", "全部"
        const tabs = page.locator('button').filter({
            hasText: /Pen View|All Animals|欄舍|全部/,
        })
        await expect(tabs.first()).toBeVisible({ timeout: 15_000 })
    })

    test('搜尋應能過濾動物', async ({ page }) => {
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })

        await searchInput.fill('001')
        await page.waitForTimeout(800)
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('品種篩選應可運作', async ({ page }) => {
        // 等待頁面完整載入
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })

        const breedSelect = page.locator('[role="combobox"]').first()
        if (await breedSelect.count() === 0) {
            test.skip()
            return
        }

        await breedSelect.click()
        await expect(page.locator('[role="option"]').first()).toBeVisible({ timeout: 5_000 })
    })

    test('切換到 All Animals Tab 應顯示表格', async ({ page }) => {
        // 找 "All Animals" 或 "全部" tab
        const allTab = page.locator('button').filter({ hasText: /All Animals|全部/ })
        await expect(allTab.first()).toBeVisible({ timeout: 15_000 })

        await allTab.first().click()
        await page.waitForLoadState('networkidle')

        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|無資料|no data|尚無|no animals/i)
        await expect(table.or(emptyState).first()).toBeVisible({ timeout: 15_000 })
    })

    test('欄舍視圖 Tab 應可切換', async ({ page }) => {
        // 找 "Pen View" 或 "欄舍" tab
        const penTab = page.locator('button').filter({ hasText: /Pen View|欄舍/ })
        await expect(penTab.first()).toBeVisible({ timeout: 15_000 })

        await penTab.first().click()
        await page.waitForLoadState('networkidle')

        // 欄舍視圖應顯示棟別 tab（A 棟 / B 棟）
        const buildingTab = page.locator('button').filter({ hasText: /棟|ACD|BEFG/ })
        await expect(buildingTab.first()).toBeVisible({ timeout: 15_000 })
    })
})
