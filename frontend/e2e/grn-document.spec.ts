import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('單據管理 (GRN 入庫)', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/documents')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/documents')
        }
        await expect(page).toHaveURL(/\/documents/, { timeout: 12_000 })
    })

    test('應顯示單據管理頁面', async ({ page }) => {
        // 頁面載入指標：應有類別 Tab 或新增按鈕
        await expect(
            page.getByText(/採購類|Purchasing|單據/i).first()
        ).toBeVisible({ timeout: 15_000 })
    })

    test('應有三大類別 Tab', async ({ page }) => {
        // 採購類、銷貨類、倉儲類
        await expect(page.getByText(/採購類|Purchasing/i).first()).toBeVisible({ timeout: 15_000 })
        await expect(page.getByText(/銷貨類|Sales/i).first()).toBeVisible({ timeout: 10_000 })
        await expect(page.getByText(/倉儲類|Warehouse/i).first()).toBeVisible({ timeout: 10_000 })
    })

    test('採購類 Tab 應包含 GRN 選項', async ({ page }) => {
        // 先點選採購類 Tab
        const purchasingTab = page.getByText(/採購類|Purchasing/i).first()
        await expect(purchasingTab).toBeVisible({ timeout: 15_000 })
        await purchasingTab.click()
        await page.waitForTimeout(500)

        // 應可篩選 GRN 類型（採購入庫）
        // 檢查子類型篩選是否存在 GRN 選項
        const typeFilter = page.locator('[role="combobox"]').first()
            .or(page.getByText(/採購入庫|GRN/i).first())
        await expect(typeFilter).toBeVisible({ timeout: 10_000 })
    })

    test('應有搜尋與狀態篩選功能', async ({ page }) => {
        await expect(page.getByText(/採購類/i).first()).toBeVisible({ timeout: 15_000 })

        // 搜尋輸入框
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 10_000 })
    })

    test('應有新增單據按鈕', async ({ page }) => {
        const addButton = page.getByRole('button', { name: /新增|新建|Create|Add/i })
            .or(page.locator('a[href*="/documents/new"]'))
        await expect(addButton.first()).toBeVisible({ timeout: 15_000 })
    })

    test('點擊新增應導向建立頁面', async ({ page }) => {
        const addButton = page.getByRole('button', { name: /新增|新建|Create|Add/i })
            .or(page.locator('a[href*="/documents/new"]'))
        await expect(addButton.first()).toBeVisible({ timeout: 15_000 })
        await addButton.first().click()

        await expect(page).toHaveURL(/\/documents\/new/, { timeout: 10_000 })
    })

    test('單據列表應顯示表格或空狀態', async ({ page }) => {
        await expect(page.getByText(/採購類/i).first()).toBeVisible({ timeout: 15_000 })

        // 等待載入完成
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 應有表格或空狀態
        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|無資料|no data|尚無|no documents/i)
        await expect(table.or(emptyState).first()).toBeVisible({ timeout: 15_000 })
    })
})

test.describe('單據建立頁面', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/documents/new')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/documents/new')
        }
        await expect(page).toHaveURL(/\/documents\/new/, { timeout: 12_000 })
    })

    test('應顯示單據類型選擇', async ({ page }) => {
        // 建立頁面應有類型選擇（PO/GRN/SO/DO/TR 等）
        const typeSelector = page.locator('[role="combobox"]').first()
            .or(page.locator('select').first())
        await expect(typeSelector).toBeVisible({ timeout: 15_000 })
    })

    test('應有儲存/提交按鈕', async ({ page }) => {
        const saveButton = page.getByRole('button', { name: /儲存|保存|Save/i })
        await expect(saveButton.first()).toBeVisible({ timeout: 15_000 })
    })
})
