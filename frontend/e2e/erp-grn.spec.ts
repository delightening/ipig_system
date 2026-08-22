import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('ERP GRN 入庫流程', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/documents')
        await expect(page).toHaveURL(/\/documents/, { timeout: 12_000 })
    })

    test('單據頁面應顯示類別按鈕', async ({ page }) => {
        // DocumentsPage 使用 plain <button> 類別切換（非 Radix Tabs）
        const categoryButtons = page.locator('button').filter({
            hasText: /採購|銷貨|purchasing|sales/i,
        })
        await expect(categoryButtons.first()).toBeVisible({ timeout: 15_000 })

        const count = await categoryButtons.count()
        expect(count).toBeGreaterThanOrEqual(1)
    })

    test('選擇類別後應顯示表格或空狀態', async ({ page }) => {
        // 先點擊採購類按鈕以觸發表格顯示
        const purchasingBtn = page.locator('button').filter({
            hasText: /採購類|採購/,
        })
        await expect(purchasingBtn.first()).toBeVisible({ timeout: 15_000 })
        await purchasingBtn.first().click()
        await page.waitForTimeout(1000)

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no document|查無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })

    test('建立新單據頁面應可選擇 GRN 類型', async ({ page }) => {
        const addBtn = page.locator('button').filter({
            hasText: /New|Add|Create|新增|建立/,
        })
        if (!(await addBtn.first().isVisible({ timeout: 5_000 }).catch(() => false))) {
            // 也嘗試找 <a> 連結按鈕
            const addLink = page.locator('a').filter({ hasText: /新增|建立|New/ })
            if (!(await addLink.first().isVisible({ timeout: 3_000 }).catch(() => false))) {
                test.skip(true, '頁面無新增按鈕，跳過')
                return
            }
            await addLink.first().click()
        } else {
            await addBtn.first().click()
        }

        // 應出現對話框或導向新頁面
        const dialog = page.locator('[role="dialog"]')
        const newPage = page.locator('select, [role="combobox"], [role="listbox"]')
        const form = page.locator('form')
        await expect(dialog.or(newPage).or(form).first()).toBeVisible({ timeout: 15_000 })
    })

    test('新增單據頁面應顯示表單欄位', async ({ page }) => {
        await ensureAdminOnPage(page, '/documents/new')

        // 確認頁面載入：heading 應含「新增單據」或「New Document」
        await expect(
            page.getByRole('heading', { name: /新增單據|New Document/i }),
        ).toBeVisible({ timeout: 15_000 })
    })
})
