/**
 * Worker 級別共用 admin context：只登入一次，該 worker 內所有 test 共用同一 context（同 session）。
 * 包含健壯的 session 檢查、自動 refresh 和重試機制。
 */
import base, { expect } from '@playwright/test'
import type { BrowserContext } from 'playwright'
import { performLogin, getAdminCredentials } from '../auth-helpers'
import { globalSessionMonitor } from '../helpers/session-monitor'

/**
 * 檢查 session 是否過期（通過 cookie）
 */
async function isSessionExpired(context: BrowserContext): Promise<boolean> {
    // ⚠️ CRITICAL FIX: 必須指定 URL 才能獲取 cookies！
    const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'
    const cookies = await context.cookies(baseURL)
    const accessToken = cookies.find((c) => c.name === 'access_token')

    if (!accessToken) {
        return true // 沒有 token = 已過期
    }

    // 檢查 cookie 過期時間
    if (accessToken.expires) {
        const expiresAt = accessToken.expires * 1000 // 轉為毫秒
        const now = Date.now()

        if (expiresAt <= now) {
            return true
        }
    }

    return false
}

/**
 * 嘗試 refresh token（通過訪問 /api/me）
 */
async function tryRefreshToken(context: BrowserContext): Promise<boolean> {
    const page = await context.newPage()
    try {
        const response = await page.request.get('/api/me')

        if (response.status() === 200) {
            console.log('[admin-context] Token refresh 成功（/api/me 返回 200）')
            return true
        } else if (response.status() === 401) {
            console.warn('[admin-context] Token refresh 失敗（401），需要重新登入')
            return false
        }

        return false
    } catch (error) {
        console.error('[admin-context] Token refresh 失敗:', error)
        return false
    } finally {
        await page.close()
    }
}

/**
 * 確保已登入（健壯版本，包含多種檢查）
 */
async function ensureLoggedIn(context: BrowserContext, maxRetries = 3): Promise<void> {
    const page = await context.newPage()

    try {
        for (let retry = 0; retry < maxRetries; retry++) {
            // 1. 檢查 session 是否過期（通過 cookie）
            const expired = await isSessionExpired(context)
            if (expired) {
                console.log('[admin-context] Session 已過期，重新登入')
                const { email, password } = getAdminCredentials()
                if (password) {
                    await performLogin(page, email, password)
                    globalSessionMonitor.markSessionStart() // 標記 session 開始
                    return // 登入成功，結束
                }
            } else {
                // Cookie 有效，session 確認有效
                return
            }
        }

        // 達到最大重試次數
        throw new Error(`ensureLoggedIn 失敗（重試 ${maxRetries} 次）`)
    } finally {
        await page.close()
    }
}

export const test = base.extend<{}, { sharedAdminContext: BrowserContext }>({
    sharedAdminContext: [
        async ({ browser }, use) => {
            const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'
            const ctx = await browser.newContext({
                baseURL,
                storageState: { cookies: [], origins: [] },
            })
            await ensureLoggedIn(ctx)
            await use(ctx)
            await ctx.close()
        },
        { scope: 'worker' },
    ],
    page: [
        async ({ sharedAdminContext }, use, testInfo) => {
            const page = await sharedAdminContext.newPage()

            // 在測試開始前檢查 session 狀態
            const sessionAge = globalSessionMonitor.getSessionAge()
            const jwtTtlMinutes = parseInt(process.env.JWT_EXPIRATION_MINUTES || '15', 10)

            if (sessionAge > 0) {
                // 不是第一個測試，檢查 session
                const remaining = globalSessionMonitor.getSessionRemaining(jwtTtlMinutes)

                if (remaining < 60) {
                    // 剩餘 < 1 分鐘，主動 refresh
                    console.warn(
                        `[admin-context] Session 即將過期（剩餘 ${remaining}s），嘗試 refresh`,
                    )
                    const refreshed = await tryRefreshToken(sharedAdminContext)

                    if (!refreshed) {
                        // Refresh 失敗，重新登入
                        await ensureLoggedIn(sharedAdminContext)
                    }
                }
            }

            // 使用 page
            await use(page)

            // 清理
            await page.close()
        },
        { scope: 'test' },
    ],
})

export { expect }
