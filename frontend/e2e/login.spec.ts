import { test, expect } from '@playwright/test'

/**
 * 登入流程 E2E 測試
 *
 * 前置條件：系統已啟動且可存取
 */
test.describe('登入流程', () => {
    test('應顯示登入頁面', async ({ page }) => {
        await page.goto('/login')

        // 確認登入表單存在
        await expect(page.getByRole('heading', { level: 1 })).toBeVisible()
        await expect(page.locator('input[type="email"], input[name="email"]')).toBeVisible()
        await expect(page.locator('input[type="password"]')).toBeVisible()
    })

    test('空白表單送出應顯示錯誤', async ({ page }) => {
        await page.goto('/login')

        // 點擊登入按鈕（不填寫任何欄位）
        await page.getByRole('button', { name: /登入|login|sign in/i }).click()

        // 應出現驗證錯誤提示
        await expect(page.locator('[role="alert"], .error, [data-state="open"]')).toBeVisible({
            timeout: 5000,
        })
    })

    test('錯誤的密碼應顯示錯誤訊息', async ({ page }) => {
        await page.goto('/login')

        // 填入錯誤的帳號密碼
        await page.locator('input[type="email"], input[name="email"]').fill('wrong@example.com')
        await page.locator('input[type="password"]').fill('wrongpassword')
        await page.getByRole('button', { name: /登入|login|sign in/i }).click()

        // 應出現錯誤提示
        await expect(page.locator('[role="alert"], .error, [data-state="open"]')).toBeVisible({
            timeout: 10000,
        })
    })
})
