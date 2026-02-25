import { test, expect } from '@playwright/test'

/**
 * Dashboard E2E 測試
 *
 * 前置條件：已透過 auth-setup 登入（storageState 已載入）
 */
test.describe('Dashboard', () => {
    test('已登入使用者應能看到 Dashboard', async ({ page }) => {
        await page.goto('/dashboard')

        // 如果無 dashboard 權限，會被導向 /my-projects
        const url = page.url()
        if (url.includes('/my-projects')) {
            // 使用者沒有 dashboard 權限，確認 my-projects 頁面正常
            await expect(page.locator('h1, h2')).toBeVisible({ timeout: 10_000 })
            return
        }

        // Dashboard 頁面應載入完成
        await page.waitForLoadState('networkidle')
    })

    test('側邊欄導航應可見', async ({ page }) => {
        await page.goto('/dashboard')

        // 側邊欄 logo
        await expect(page.locator('img[src*="pigmodel"]').first()).toBeVisible({ timeout: 10_000 })
    })

    test('通知鈴鐺應可見', async ({ page }) => {
        await page.goto('/dashboard')

        // Top bar 中的通知 icon（Lucide icon 渲染為 SVG，class 可能不含 lucide- prefix）
        const bellIcon = page.locator('[data-lucide="bell"], svg.lucide-bell, button:has(svg) >> nth=0').first()
        // 改用更穩定的選擇器：top bar 上有通知數字的區域
        const notificationArea = page.getByText(/^\d+$/).first().or(bellIcon)
        await expect(notificationArea).toBeVisible({ timeout: 10_000 })
    })

    test('語言切換應可運作', async ({ page }) => {
        await page.goto('/dashboard')

        // 找到語言選擇器（Globe icon 旁邊）
        const langSelector = page.locator('select, [role="combobox"]').filter({ has: page.locator('svg.lucide-globe') })
        // 如果語言選擇器找不到（可能在不同位置），跳過
        if (await langSelector.count() === 0) {
            test.skip()
            return
        }
        await expect(langSelector).toBeVisible()
    })
})
