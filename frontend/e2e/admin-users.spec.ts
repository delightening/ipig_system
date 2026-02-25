import { test, expect } from '@playwright/test'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'

/**
 * 管理員：使用者管理 E2E 測試
 *
 * 前置條件：需要 admin 帳號。若未設定 E2E_ADMIN_EMAIL，整組跳過。
 */

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const adminAuthFile = path.join(__dirname, '.auth', 'admin.json')

test.describe('使用者管理（Admin）', () => {
    // 使用 admin storageState
    test.beforeEach(async () => {
        if (!fs.existsSync(adminAuthFile) || !process.env.E2E_ADMIN_EMAIL) {
            test.skip()
        }
    })

    test.use({
        storageState: fs.existsSync(path.join(path.dirname(fileURLToPath(import.meta.url)), '.auth', 'admin.json'))
            ? path.join(path.dirname(fileURLToPath(import.meta.url)), '.auth', 'admin.json')
            : { cookies: [], origins: [] },
    })

    test('應能存取使用者管理頁面', async ({ page }) => {
        await page.goto('/admin/users')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })

        await expect(page.getByText('使用者管理')).toBeVisible()
    })

    test('應顯示使用者列表', async ({ page }) => {
        await page.goto('/admin/users')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })

        const table = page.locator('table')
        await expect(table).toBeVisible({ timeout: 10_000 })

        const rows = table.locator('tbody tr')
        await expect(rows.first()).toBeVisible({ timeout: 10_000 })
    })

    test('應有新增使用者按鈕', async ({ page }) => {
        await page.goto('/admin/users')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })

        const createBtn = page.getByRole('button', { name: '新增使用者' })
        await expect(createBtn).toBeVisible()
    })

    test('新增使用者對話框應可開啟', async ({ page }) => {
        await page.goto('/admin/users')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })

        await page.getByRole('button', { name: '新增使用者' }).click()

        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 5_000 })

        await expect(dialog.locator('#email, input[type="email"]').first()).toBeVisible()
        await expect(dialog.locator('#password, input[type="password"]').first()).toBeVisible()
        await expect(dialog.locator('#display_name, input[name="display_name"]').first()).toBeVisible()
    })

    test('應顯示使用者分頁資訊', async ({ page }) => {
        await page.goto('/admin/users')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({ timeout: 15_000 })

        const pagination = page.getByText(/共.*筆/)
        await expect(pagination).toBeVisible({ timeout: 10_000 })
    })
})
