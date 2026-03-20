import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('庫存流水帳', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/inventory/ledger')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/inventory/ledger')
        }
        await expect(page).toHaveURL(/\/inventory\/ledger/, { timeout: 12_000 })
    })

    test('應顯示庫存流水頁面標題', async ({ page }) => {
        await expect(
            page.getByText(/庫存流水|Stock Ledger/i)
        ).toBeVisible({ timeout: 15_000 })
    })

    test('應顯示流水帳表格欄位', async ({ page }) => {
        // 等待表格載入
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })

        // 驗證表頭欄位
        const headerRow = table.locator('thead tr')
        await expect(headerRow.getByText(/時間|Time/i)).toBeVisible({ timeout: 10_000 })
        await expect(headerRow.getByText(/倉庫|Warehouse/i)).toBeVisible()
        await expect(headerRow.getByText(/品項|Product|Item/i)).toBeVisible()
        await expect(headerRow.getByText(/單據|Document/i)).toBeVisible()
        await expect(headerRow.getByText(/方向|Direction/i)).toBeVisible()
        await expect(headerRow.getByText(/數量|Quantity|Qty/i)).toBeVisible()
    })

    test('表格應正確顯示資料或空狀態', async ({ page }) => {
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })

        // 等待載入完成（Loader 消失）
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 應顯示資料列或空狀態訊息
        const dataRows = table.locator('tbody tr')
        const emptyState = page.getByText(/尚無庫存流水資料|No data|no records/i)
        await expect(dataRows.first().or(emptyState)).toBeVisible({ timeout: 10_000 })
    })

    test('方向欄位應顯示入庫/出庫標籤', async ({ page }) => {
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 如果有資料，檢查方向 badge
        const dataRows = table.locator('tbody tr')
        const count = await dataRows.count()
        if (count > 0) {
            // 方向 badge 應存在（入庫/出庫/調入/調出/調增/調減）
            const directionBadge = dataRows.first().locator('[class*="badge"], [data-slot="badge"]')
            await expect(directionBadge.first()).toBeVisible({ timeout: 5_000 })
        }
    })
})

test.describe('庫存查詢', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/inventory')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/inventory')
        }
        await expect(page).toHaveURL(/\/inventory/, { timeout: 12_000 })
    })

    test('應顯示庫存查詢頁面', async ({ page }) => {
        await expect(
            page.getByText(/庫存查詢|Inventory|Stock/i).first()
        ).toBeVisible({ timeout: 15_000 })
    })

    test('應有搜尋功能', async ({ page }) => {
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })
    })
})
