import { expect } from '@playwright/test'
import path from 'path'
import { fileURLToPath } from 'url'
import dotenv from 'dotenv'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
dotenv.config({ path: path.resolve(__dirname, '../../.env') })

/**
 * 透過瀏覽器執行登入，處理 429 rate limit 重試。
 * 成功後確保頁面已離開 /login。
 */
export async function performLogin(
    page: import('@playwright/test').Page,
    email: string,
    password: string,
    maxRetries = 5,
) {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        await page.goto('/login')
        await page.waitForLoadState('networkidle')
        await page.locator('#email').fill(email)
        await page.locator('#password').fill(password)

        const [response] = await Promise.all([
            page.waitForResponse(
                (resp: import('@playwright/test').APIResponse) =>
                    resp.url().includes('/api/auth/login') && resp.request().method() === 'POST',
                { timeout: 15_000 },
            ),
            page.getByRole('button', { name: '登入' }).click(),
        ])

        if (response.status() === 429) {
            const retryAfter = Number(response.headers()['retry-after']) || 60
            const waitMs = Math.min(retryAfter * 1000 + 2000, 65_000) // 最多等 65s
            if (attempt < maxRetries) {
                await page.waitForTimeout(waitMs)
                continue
            }
            throw new Error(`Login rate limited (429) after ${maxRetries} retries`)
        }

        if (response.status() !== 200) {
            const body = await response.text()
            const hint = body.length > 0 ? ` Response: ${body.slice(0, 200)}` : ''
            throw new Error(`Login API returned ${response.status()}.${hint}`)
        }

        try {
            await expect(page).toHaveURL(/\/(dashboard|my-projects|force-change)/, {
                timeout: 8_000,
            })
        } catch {
            await page.goto('/dashboard')
            await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })
        }

        return
    }
}

/** 從環境變數或 .env 取得 admin 帳密 */
export function getAdminCredentials() {
    const email = process.env.E2E_ADMIN_EMAIL || 'admin@ipig.local'
    const password = process.env.E2E_ADMIN_PASSWORD || process.env.ADMIN_INITIAL_PASSWORD
    return { email, password }
}

/**
 * 先導向 path，若被導向 /login（session 過期）則重新登入後再導向 path。
 * 用於 beforeEach，讓同一 context 在 session 過期時能自動恢復。
 */
export async function ensureAdminOnPage(
    page: import('@playwright/test').Page,
    path: string,
): Promise<void> {
    await page.goto(path)
    await page.waitForLoadState('domcontentloaded')
    if (page.url().includes('/login')) {
        const { email, password } = getAdminCredentials()
        if (password) {
            await performLogin(page, email, password)
            await page.goto(path)
            await page.waitForLoadState('domcontentloaded')
        }
    }
}
