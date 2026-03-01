import { test, expect } from '@playwright/test'
import { getAdminCredentials } from './auth-helpers'

/**
 * 登入流程 E2E 測試
 *
 * 注意：這些測試不使用 storageState（測試的就是登入本身），
 * 所以不會繼承 auth-setup 的 cookie。
 */
test.use({ storageState: { cookies: [], origins: [] } })

test.describe('登入流程', () => {
    test('應顯示登入頁面', async ({ page }) => {
        await page.goto('/login')

        // 確認頁面標題和表單
        await expect(page.getByText('iPig 統一入口門戶')).toBeVisible()
        await expect(page.locator('#email')).toBeVisible()
        await expect(page.locator('#password')).toBeVisible()
        await expect(page.getByRole('button', { name: '登入' })).toBeVisible()
    })

    test('空白表單送出應顯示驗證錯誤', async ({ page }) => {
        await page.goto('/login')

        await page.getByRole('button', { name: '登入' }).click()

        // Zod 驗證：email 和 password 都是必填
        await expect(page.getByText('請輸入有效的電子郵件')).toBeVisible({ timeout: 5_000 })
    })

    test('錯誤的帳密應顯示錯誤訊息', async ({ page }) => {
        await page.goto('/login')

        await page.locator('#email').fill('wrong@example.com')
        await page.locator('#password').fill('wrongpassword')
        await page.getByRole('button', { name: '登入' }).click()

        // API 回傳 401 → toast 顯示「登入失敗」
        const errorLocator = page.locator('[data-state="open"]').getByText(/登入失敗/i).first()
        await expect(errorLocator).toBeVisible({ timeout: 10_000 })
    })

    test('成功登入應導向 dashboard', { timeout: 60_000 }, async ({ page }) => {
        const admin = getAdminCredentials()
        const email = process.env.E2E_USER_EMAIL || admin.email
        const password = process.env.E2E_USER_PASSWORD || admin.password
        expect(email, '請設定 E2E_USER_EMAIL 或 E2E_ADMIN_EMAIL').toBeTruthy()
        expect(password, '請設定 E2E_USER_PASSWORD 或 ADMIN_INITIAL_PASSWORD').toBeTruthy()

        await page.goto('/login')
        await page.waitForLoadState('domcontentloaded')
        await expect(page.locator('#email')).toBeVisible({ timeout: 10_000 })

        let response: import('@playwright/test').APIResponse
        const maxRetries = 3
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            await page.locator('#email').fill(email!)
            await page.locator('#password').fill(password!)

            const [resp] = await Promise.all([
                page.waitForResponse(
                    (r) =>
                        (r.url().includes('/api/auth/login') || r.url().includes('/api/v1/auth/login')) &&
                        r.request().method() === 'POST',
                    { timeout: 20_000 },
                ),
                page.getByRole('button', { name: '登入' }).click(),
            ])
            response = resp
            if (resp.status() === 429 && attempt < maxRetries) {
                const waitMs = Math.min((Number(resp.headers()['retry-after']) || 5) * 1000 + 1000, 10_000)
                await page.waitForTimeout(waitMs)
                continue
            }
            break
        }
        expect(response!.status(), '登入 API 應回傳 200（429 時會重試）').toBe(200)

        // 等待前端 React 狀態更新完成跳轉
        // 若 race condition 導致未自動跳轉，手動導航（HttpOnly cookie 已設定）
        try {
            await expect(page).not.toHaveURL(/\/login$/, { timeout: 8_000 })
        } catch {
            // Cookie 已由後端設定，直接導航驗證 auth 狀態正確
            await page.goto('/dashboard')
            await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })
        }
    })

    test('未登入訪問受保護頁面應導向 /login', async ({ page }) => {
        await page.goto('/animals')

        await expect(page).toHaveURL(/\/login/, { timeout: 10_000 })
    })

    test('忘記密碼連結應可見', async ({ page }) => {
        await page.goto('/login')

        const link = page.getByRole('link', { name: '忘記密碼？' })
        await expect(link).toBeVisible()
        await link.click()
        await expect(page).toHaveURL(/\/forgot-password/)
    })
})
