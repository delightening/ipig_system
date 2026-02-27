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
 * Session 有效性緩存（減少不必要的檢查）
 */
interface SessionCache {
    isValid: boolean
    checkedAt: number
    expiresAt: number
}

let sessionCache: SessionCache | null = null
const CACHE_TTL = 30_000 // 30 秒緩存

/**
 * 檢查 session 是否過期（通過 cookie）
 * 增加容錯機制和緩存
 */
async function isSessionExpired(context: BrowserContext): Promise<boolean> {
    const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'
    
    // 檢查緩存
    if (sessionCache && Date.now() < sessionCache.checkedAt + CACHE_TTL) {
        if (Date.now() < sessionCache.expiresAt) {
            return !sessionCache.isValid
        }
    }

    try {
        const cookies = await context.cookies(baseURL)
        const accessToken = cookies.find((c) => c.name === 'access_token')

        if (!accessToken) {
            sessionCache = {
                isValid: false,
                checkedAt: Date.now(),
                expiresAt: Date.now() + CACHE_TTL,
            }
            return true
        }

        if (accessToken.expires) {
            const expiresAt = accessToken.expires * 1000
            const now = Date.now()
            const isValid = expiresAt > now + 60_000 // 至少還有 1 分鐘才過期
            
            // 更新緩存
            sessionCache = {
                isValid,
                checkedAt: Date.now(),
                expiresAt: Math.min(expiresAt, Date.now() + CACHE_TTL),
            }
            
            return !isValid
        }

        // 沒有 expires 資訊，假設有效但緩存時間較短
        sessionCache = {
            isValid: true,
            checkedAt: Date.now(),
            expiresAt: Date.now() + 10_000, // 10 秒緩存
        }
        return false
    } catch (error) {
        // Cookie 讀取失敗，使用緩存或假設過期
        console.warn(`[admin-context] Cookie 讀取失敗: ${error}`)
        if (sessionCache && Date.now() < sessionCache.checkedAt + CACHE_TTL) {
            return !sessionCache.isValid
        }
        // 無緩存時，保守假設已過期
        return true
    }
}

/**
 * 防抖鎖定（避免多個測試同時觸發登入）
 */
let loginInProgress: Promise<void> | null = null
const LOGIN_DEBOUNCE_MS = 2_000 // 2 秒防抖窗口

/**
 * 確保已登入（僅在 session 過期時重新登入）
 * 增加防抖機制避免同時觸發多個登入請求
 */
async function ensureLoggedIn(context: BrowserContext, maxRetries = 3): Promise<void> {
    // 防抖：如果已有登入進行中，等待它完成
    if (loginInProgress) {
        console.log('[admin-context] 登入進行中，等待完成...')
        try {
            await loginInProgress
            // 登入完成後再次檢查 session
            const expired = await isSessionExpired(context)
            if (!expired) {
                return
            }
        } catch (error) {
            console.warn(`[admin-context] 等待中的登入失敗: ${error}`)
            // 繼續執行自己的登入流程
        }
    }

    // 執行登入（使用防抖鎖定）
    const loginPromise = (async () => {
        const page = await context.newPage()

        try {
            for (let retry = 0; retry < maxRetries; retry++) {
                const expired = await isSessionExpired(context)
                if (!expired) {
                    // 清除緩存，標記為有效
                    sessionCache = {
                        isValid: true,
                        checkedAt: Date.now(),
                        expiresAt: Date.now() + CACHE_TTL,
                    }
                    return
                }

                console.log(`[admin-context] Session 已過期，重新登入（attempt ${retry + 1}/${maxRetries}）`)
                const { email, password } = getAdminCredentials()
                if (password) {
                    await performLogin(page, email, password)
                    globalSessionMonitor.markSessionStart()
                    // 清除緩存並標記為有效
                    sessionCache = {
                        isValid: true,
                        checkedAt: Date.now(),
                        expiresAt: Date.now() + CACHE_TTL,
                    }
                    // 儲存更新後的 storageState 供後續使用
                    await context.storageState({ path: adminStatePath })
                    return
                }
            }

            throw new Error(`ensureLoggedIn 失敗（重試 ${maxRetries} 次）`)
        } finally {
            await page.close()
            // 清除防抖鎖定（延遲清除，避免短時間內重複觸發）
            setTimeout(() => {
                if (loginInProgress === loginPromise) {
                    loginInProgress = null
                }
            }, LOGIN_DEBOUNCE_MS)
        }
    })()

    loginInProgress = loginPromise
    await loginPromise
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
                // 初始化緩存
                sessionCache = {
                    isValid: true,
                    checkedAt: Date.now(),
                    expiresAt: Date.now() + CACHE_TTL,
                }
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
