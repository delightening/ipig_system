/**
 * E2E 診斷工具
 *
 * 用於測試失敗時收集診斷資訊，幫助排查問題
 */
import { Page, BrowserContext } from '@playwright/test'

export class E2EDiagnostics {
    /**
     * 檢查當前是否在登入頁
     */
    static async isOnLoginPage(page: Page): Promise<boolean> {
        return page.url().includes('/login')
    }

    /**
     * 檢查 access_token cookie 是否存在
     */
    static async hasAccessToken(page: Page): Promise<boolean> {
        const cookies = await page.context().cookies()
        return cookies.some((c) => c.name === 'access_token' && c.value.length > 0)
    }

    /**
     * 獲取 access_token cookie 的詳細資訊
     */
    static async getAccessTokenInfo(page: Page): Promise<{
        exists: boolean
        value?: string
        expires?: number
        expiresAt?: string
    }> {
        const cookies = await page.context().cookies()
        const accessToken = cookies.find((c) => c.name === 'access_token')

        if (!accessToken) {
            return { exists: false }
        }

        return {
            exists: true,
            value: accessToken.value.substring(0, 20) + '...', // 只顯示前 20 字元
            expires: accessToken.expires,
            expiresAt: accessToken.expires
                ? new Date(accessToken.expires * 1000).toISOString()
                : 'session',
        }
    }

    /**
     * 檢查 session 狀態（綜合檢查）
     */
    static async checkSessionStatus(page: Page): Promise<{
        isOnLoginPage: boolean
        hasAccessToken: boolean
        currentUrl: string
        accessTokenInfo: Awaited<ReturnType<typeof E2EDiagnostics.getAccessTokenInfo>>
    }> {
        return {
            isOnLoginPage: await this.isOnLoginPage(page),
            hasAccessToken: await this.hasAccessToken(page),
            currentUrl: page.url(),
            accessTokenInfo: await this.getAccessTokenInfo(page),
        }
    }

    /**
     * 記錄診斷資訊到 console（測試失敗時使用）
     */
    static async logDiagnostics(page: Page, testName: string) {
        const status = await this.checkSessionStatus(page)
        const timestamp = new Date().toISOString()

        console.error(`
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[E2E Diagnostics] ${testName}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 基本資訊
  Current URL: ${status.currentUrl}
  On Login Page: ${status.isOnLoginPage ? '❌ YES (問題！)' : '✅ NO'}
  Timestamp: ${timestamp}

🔐 Session 狀態
  Has Access Token: ${status.hasAccessToken ? '✅ YES' : '❌ NO (問題！)'}
  Token Value: ${status.accessTokenInfo.value || 'N/A'}
  Token Expires: ${status.accessTokenInfo.expiresAt || 'N/A'}

💡 建議
${this.getSuggestions(status)}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
`)
    }

    /**
     * 根據診斷狀態提供建議
     */
    private static getSuggestions(status: Awaited<ReturnType<typeof E2EDiagnostics.checkSessionStatus>>): string {
        const suggestions: string[] = []

        if (status.isOnLoginPage) {
            suggestions.push('  • Session 失效，被導向登入頁')
            suggestions.push('  • 檢查 JWT_EXPIRATION_MINUTES 是否足夠（建議 >= 15）')
            suggestions.push('  • 執行：npx tsx e2e/scripts/verify-config.ts')
        }

        if (!status.hasAccessToken) {
            suggestions.push('  • access_token cookie 不存在')
            suggestions.push('  • 可能是 Cookie 設定問題（COOKIE_SECURE、COOKIE_DOMAIN）')
            suggestions.push('  • 檢查 .env 中 COOKIE_SECURE=false')
        }

        if (status.accessTokenInfo.exists && status.accessTokenInfo.expires) {
            const now = Date.now()
            const expiresAt = status.accessTokenInfo.expires * 1000
            const remainingMs = expiresAt - now

            if (remainingMs < 0) {
                suggestions.push('  • Token 已過期')
            } else if (remainingMs < 60_000) {
                suggestions.push(`  • Token 即將過期（剩餘 ${Math.floor(remainingMs / 1000)} 秒）`)
            }
        }

        if (suggestions.length === 0) {
            suggestions.push('  • 狀態看起來正常，請檢查測試邏輯或後端日誌')
        }

        return suggestions.join('\n')
    }

    /**
     * 在測試失敗時自動截圖並記錄狀態
     *
     * 使用方式：
     * ```typescript
     * test.afterEach(async ({ page }, testInfo) => {
     *   if (testInfo.status === 'failed') {
     *     await E2EDiagnostics.captureFailureState(page, testInfo.title)
     *   }
     * })
     * ```
     */
    static async captureFailureState(page: Page, testName: string) {
        try {
            await this.logDiagnostics(page, testName)
            // Playwright 會自動截圖（screenshot: 'only-on-failure'），這裡額外記錄狀態
        } catch (error) {
            console.error('[E2EDiagnostics] 無法收集診斷資訊:', error)
        }
    }

    /**
     * 檢查 session 是否健康
     *
     * @returns true 如果 session 健康，false 如果有問題
     */
    static async isSessionHealthy(page: Page): Promise<boolean> {
        const status = await this.checkSessionStatus(page)

        // 健康條件：
        // 1. 不在登入頁
        // 2. 有 access_token
        return !status.isOnLoginPage && status.hasAccessToken
    }

    /**
     * 等待 session 就緒（用於測試開始前的檢查）
     *
     * @param page - Page 物件
     * @param timeoutMs - 超時時間（預設 10 秒）
     * @throws 如果 session 不健康
     */
    static async waitForSessionReady(page: Page, timeoutMs = 10_000): Promise<void> {
        const startTime = Date.now()

        while (Date.now() - startTime < timeoutMs) {
            const isHealthy = await this.isSessionHealthy(page)

            if (isHealthy) {
                return
            }

            // 每 500ms 檢查一次
            await page.waitForTimeout(500)
        }

        // 超時，記錄診斷資訊並拋出錯誤
        await this.logDiagnostics(page, 'waitForSessionReady 超時')
        throw new Error(`Session 未就緒（超時 ${timeoutMs}ms）`)
    }

    /**
     * 記錄所有 cookies（用於深度調試）
     */
    static async logAllCookies(context: BrowserContext) {
        const cookies = await context.cookies()

        console.log('\n📋 All Cookies:')
        cookies.forEach((cookie) => {
            console.log(`  ${cookie.name}:`)
            console.log(`    Value: ${cookie.value.substring(0, 50)}${cookie.value.length > 50 ? '...' : ''}`)
            console.log(`    Domain: ${cookie.domain}`)
            console.log(`    Path: ${cookie.path}`)
            console.log(`    Secure: ${cookie.secure}`)
            console.log(`    HttpOnly: ${cookie.httpOnly}`)
            console.log(
                `    Expires: ${cookie.expires ? new Date(cookie.expires * 1000).toISOString() : 'session'}`,
            )
            console.log(`    SameSite: ${cookie.sameSite || 'None'}`)
        })
        console.log()
    }
}
