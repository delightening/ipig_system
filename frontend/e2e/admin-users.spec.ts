import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

/**
 * 管理員：使用者管理 E2E 測試（使用 worker 共用 admin context，無需 admin.json）
 */
test.describe('使用者管理（Admin）', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/admin/users')
    })

    test('應能存取使用者管理頁面', async ({ page }) => {
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })
        await expect(page.getByText(/使用者管理|User Management/)).toBeVisible()
    })

    test('應顯示使用者列表', async ({ page }) => {
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 10_000 })
        const rows = table.locator('tbody tr')
        await expect(rows.first()).toBeVisible({ timeout: 10_000 })
    })

    test('應有新增使用者按鈕', async ({ page }) => {
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })
        const createBtn = page.getByRole('button', { name: /新增使用者|Add User/ })
        await expect(createBtn).toBeVisible({ timeout: 10_000 })
    })

    test('新增使用者對話框應可開啟', async ({ page }) => {
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })
        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 15_000 })
        const createBtn = page.getByRole('button', { name: /新增使用者|Add User/ })
        await expect(createBtn).toBeVisible({ timeout: 10_000 })
        await createBtn.click()
        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 5_000 })
        await expect(dialog.locator('#email, input[type="email"]').first()).toBeVisible()
        await expect(dialog.locator('#password, input[type="password"]').first()).toBeVisible()
        await expect(dialog.locator('#display_name, input[name="display_name"]').first()).toBeVisible()
    })

    test('應顯示使用者分頁資訊', async ({ page }) => {
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })
        const pagination = page.getByText(/共.*筆|.*record/)
        await expect(pagination).toBeVisible({ timeout: 10_000 })
    })
})
