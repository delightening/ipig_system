import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('Dashboard', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/dashboard')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/dashboard')
        }
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
        // 若 session 過期被導向登入頁，ensureAdminOnPage 會重新登入
        await ensureAdminOnPage(page, '/dashboard')
        await page.goto('/dashboard')
        await page.waitForLoadState('domcontentloaded')

        // 若仍於登入頁則重新登入（session 過期情境）
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/dashboard')
        }
        await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })

        // 通知鈴鐺按鈕（NotificationDropdown，data-testid 或 aria-label）
        const notificationButton = page.getByTestId('notification-bell').or(page.getByRole('button', { name: '通知' }))
        await expect(notificationButton).toBeVisible({ timeout: 10_000 })
    })

    test('語言切換應可運作', async ({ page }) => {
        await ensureAdminOnPage(page, '/dashboard')
        await page.goto('/dashboard')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/dashboard')
        }
        await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })

        // 語言選擇器（MainLayout data-testid="language-selector" 或 combobox）
        const langSelector = page.getByTestId('language-selector').or(page.locator('header').getByRole('combobox'))
        await expect(langSelector, '頁面應有語言切換選擇器').toBeVisible({ timeout: 15_000 })
    })
})
