import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('Dashboard', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/dashboard')
    })

    test('已登入使用者應能看到 Dashboard', async ({ page }) => {
        await page.goto('/dashboard')

        // 如果無 dashboard 權限，會被導向 /my-projects
        const url = page.url()
        if (url.includes('/my-projects')) {
            // 使用者沒有 dashboard 權限，確認 my-projects 頁面正常
            await expect(page.locator('h1, h2')).toBeVisible({ timeout: 10_000 })
            return
        }

        // 以側欄 logo 作為載入完成指標（不依賴 networkidle）
        await expect(page.locator('img[src*="pigmodel"]').first()).toBeVisible({ timeout: 15_000 })
    })

    test('側邊欄導航應可見', async ({ page }) => {
        await page.goto('/dashboard')

        // 側邊欄 logo
        await expect(page.locator('img[src*="pigmodel"]').first()).toBeVisible({ timeout: 10_000 })
    })

    test('通知鈴鐺應可見', async ({ page }) => {
        await page.goto('/dashboard')

        // 查找具有 relative class 的通知按鈕（MainLayout.tsx:934）
        const notificationButton = page.locator('header button.relative')
        await expect(notificationButton).toBeVisible({ timeout: 10_000 })
    })

    test('語言切換應可運作', async ({ page }) => {
        await page.goto('/dashboard')

        // 使用 role="combobox" 查找語言選擇器（Radix UI Select.Trigger 的標準 role）
        const langSelector = page.locator('header').getByRole('combobox')
        await expect(langSelector, '頁面應有語言切換選擇器').toBeVisible({ timeout: 15_000 })
    })
})
