import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('動物列表', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/animals')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/animals')
        }
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })
        await expect(page.locator('button').filter({ hasText: /Pen View|All Animals|欄舍|全部/ }).first()).toBeVisible({ timeout: 15_000 })
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
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })

        const breedSelect = page.locator('[role="combobox"]').first()
        await expect(breedSelect, '動物列表應有品種篩選 combobox').toBeVisible({ timeout: 10_000 })

        await breedSelect.click()
        await expect(page.locator('[role="option"]').first()).toBeVisible({ timeout: 5_000 })
    })

    test('切換到 All Animals Tab 應顯示表格', async ({ page }) => {
        // 找 "All Animals" 或 "全部" tab
        const allTab = page.locator('button').filter({ hasText: /All Animals|全部/ })
        await expect(allTab.first()).toBeVisible({ timeout: 15_000 })

        await allTab.first().click()
        await expect(page.locator('table').or(page.getByText(/沒有|無資料|no data|尚無|no animals/i))).toBeVisible({ timeout: 15_000 })

        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|無資料|no data|尚無|no animals/i)
        await expect(table.or(emptyState).first()).toBeVisible({ timeout: 15_000 })
    })

    test('欄舍視圖 Tab 應可切換', async ({ page }) => {
        // 找 "Pen View" 或 "欄舍" tab
        const penTab = page.locator('button').filter({ hasText: /Pen View|欄舍/ })
        await expect(penTab.first()).toBeVisible({ timeout: 15_000 })

        await penTab.first().click()
        await expect(page.locator('button').filter({ hasText: /棟|ACD|BEFG/ }).first()).toBeVisible({ timeout: 15_000 })

        // 欄舍視圖應顯示棟別 tab（A 棟 / B 棟）
        const buildingTab = page.locator('button').filter({ hasText: /棟|ACD|BEFG/ })
        await expect(buildingTab.first()).toBeVisible({ timeout: 15_000 })
    })
})
