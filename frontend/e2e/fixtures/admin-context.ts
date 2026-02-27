/**
 * Worker 級別共用 admin context：
 * 優先載入 auth.setup 儲存的 admin.json storageState，
 * 只在 storageState 過期時才重新登入，大幅減少 API 呼叫次數。
 */
import base, { expect } from '@playwright/test'
import type { BrowserContext } from 'playwright'
import { performLogin, getAdminCredentials } from '../auth-helpers'
import { globalSessionMonitor } from '../helpers/session-monitor'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const adminStatePath = path.join(__dirname, '..', '.auth', 'admin.json')

/**
 * 檢查 session 是否過期（通過 cookie）
 */
async function isSessionExpired(context: BrowserContext): Promise<boolean> {
    const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'
    const cookies = await context.cookies(baseURL)
    const accessToken = cookies.find((c) => c.name === 'access_token')

    if (!accessToken) {
        return true
    }

    if (accessToken.expires) {
        const expiresAt = accessToken.expires * 1000
        const now = Date.now()
        if (expiresAt <= now) {
            return true
        }
    }

    return false
}

/**
 * 確保已登入（僅在 session 過期時重新登入）
 */
async function ensureLoggedIn(context: BrowserContext, maxRetries = 3): Promise<void> {
    const page = await context.newPage()

    try {
        for (let retry = 0; retry < maxRetries; retry++) {
            const expired = await isSessionExpired(context)
            if (!expired) {
                return
            }

            console.log(`[admin-context] Session 已過期，重新登入（attempt ${retry + 1}）`)
            const { email, password } = getAdminCredentials()
            if (password) {
                await performLogin(page, email, password)
                globalSessionMonitor.markSessionStart()
                // 儲存更新後的 storageState 供後續使用
                await context.storageState({ path: adminStatePath })
                return
            }
        }

        throw new Error(`ensureLoggedIn 失敗（重試 ${maxRetries} 次）`)
    } finally {
        await page.close()
    }
}

export const test = base.extend<{}, { sharedAdminContext: BrowserContext }>({
    sharedAdminContext: [
        async ({ browser }, use) => {
            const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'

            // 優先載入 auth.setup 儲存的 admin storageState
            const hasState = fs.existsSync(adminStatePath)
            const storageState = hasState
                ? adminStatePath
                : { cookies: [] as any[], origins: [] as any[] }

            const ctx = await browser.newContext({ baseURL, storageState })

            // 驗證載入的 session 是否仍有效
            const expired = await isSessionExpired(ctx)
            if (expired) {
                console.log('[admin-context] StorageState 已過期，重新登入')
                await ensureLoggedIn(ctx)
            } else {
                console.log('[admin-context] 使用已儲存的 admin storageState（免登入）')
                globalSessionMonitor.markSessionStart()
            }

            await use(ctx)
            await ctx.close()
        },
        { scope: 'worker' },
    ],
    page: [
        async ({ sharedAdminContext }, use) => {
            const page = await sharedAdminContext.newPage()

            // 檢查 session 是否即將過期
            const sessionAge = globalSessionMonitor.getSessionAge()
            const jwtTtlMinutes = parseInt(process.env.JWT_EXPIRATION_MINUTES || '60', 10)

            if (sessionAge > 0) {
                const remaining = globalSessionMonitor.getSessionRemaining(jwtTtlMinutes)
                if (remaining < 60) {
                    console.warn(
                        `[admin-context] Session 即將過期（剩餘 ${remaining}s），重新登入`,
                    )
                    await ensureLoggedIn(sharedAdminContext)
                }
            }

            await use(page)
            await page.close()
        },
        { scope: 'test' },
    ],
})

export { expect }
