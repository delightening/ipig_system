import { test as setup, expect } from '@playwright/test'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const authDir = path.join(__dirname, '.auth')

/**
 * 透過瀏覽器執行登入，處理 429 rate limit 重試。
 * 成功後確保頁面已離開 /login。
 */
async function performLogin(
    page: import('@playwright/test').Page,
    email: string,
    password: string,
    maxRetries = 3,
) {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        await page.goto('/login')
        await page.waitForLoadState('networkidle')
        await page.locator('#email').fill(email)
        await page.locator('#password').fill(password)

        const [response] = await Promise.all([
            page.waitForResponse(
                (resp) =>
                    resp.url().includes('/api/auth/login') && resp.request().method() === 'POST',
                { timeout: 15_000 },
            ),
            page.getByRole('button', { name: '登入' }).click(),
        ])

        if (response.status() === 429) {
            // Rate limited — 等待後重試
            const retryAfter = Number(response.headers()['retry-after']) || 5
            const waitMs = retryAfter * 1000 + 1000 // 多等 1 秒緩衝
            if (attempt < maxRetries) {
                await page.waitForTimeout(waitMs)
                continue
            }
            throw new Error(`Login rate limited (429) after ${maxRetries} retries`)
        }

        if (response.status() !== 200) {
            throw new Error(`Login API returned ${response.status()}`)
        }

        // 等待前端 React 狀態更新與 Router 跳轉
        try {
            await expect(page).toHaveURL(/\/(dashboard|my-projects|force-change)/, {
                timeout: 8_000,
            })
        } catch {
            // Race condition：Cookie 已設定但前端未跳轉，手動導航
            await page.goto('/dashboard')
            await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })
        }

        return // 登入成功
    }
}

/**
 * Auth Setup：執行一次登入，將 cookie/storageState 存檔
 * 後續所有測試直接載入此 state，不需重複登入。
 *
 * 需要環境變數：
 *   E2E_USER_EMAIL    一般測試帳號
 *   E2E_USER_PASSWORD 一般測試密碼
 */
setup('authenticate as user', async ({ page }) => {
    const email = process.env.E2E_USER_EMAIL
    const password = process.env.E2E_USER_PASSWORD

    if (!email || !password) {
        throw new Error(
            'E2E_USER_EMAIL and E2E_USER_PASSWORD must be set.\n' +
                'Create a test user: cargo run --bin create_test_user <email> <password> <name>',
        )
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await page.context().storageState({ path: path.join(authDir, 'user.json') })
})

setup('authenticate as admin', async ({ page }) => {
    const email = process.env.E2E_ADMIN_EMAIL
    const password = process.env.E2E_ADMIN_PASSWORD

    if (!email || !password) {
        setup.skip()
        return
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await page.context().storageState({ path: path.join(authDir, 'admin.json') })
})
