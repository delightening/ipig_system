/**
 * Session 監控工具
 *
 * 在測試執行期間監控 session 狀態，幫助識別 session 過期問題
 */
import { BrowserContext } from '@playwright/test'

export class SessionMonitor {
    private sessionStartTime: number = Date.now()
    private testStartTime: number = Date.now()

    /**
     * 標記 session 開始（通常在登入後調用）
     */
    markSessionStart() {
        this.sessionStartTime = Date.now()
        this.log('Session 已建立')
    }

    /**
     * 標記測試開始
     */
    markTestStart(testName: string) {
        this.testStartTime = Date.now()
        this.log(`測試開始: ${testName}`)
    }

    /**
     * 獲取 session 存活時間（秒）
     */
    getSessionAge(): number {
        return Math.floor((Date.now() - this.sessionStartTime) / 1000)
    }

    /**
     * 獲取測試執行時間（秒）
     */
    getTestAge(): number {
        return Math.floor((Date.now() - this.testStartTime) / 1000)
    }

    /**
     * 檢查 session 是否接近過期
     *
     * @param ttlMinutes - JWT TTL（分鐘）
     * @param warningThreshold - 警告閾值（0-1，預設 0.8 = 80%）
     * @returns true 如果 session 已使用超過 TTL 的警告閾值
     */
    isSessionNearExpiry(ttlMinutes: number = 15, warningThreshold: number = 0.8): boolean {
        const ageSeconds = this.getSessionAge()
        const ttlSeconds = ttlMinutes * 60
        return ageSeconds > ttlSeconds * warningThreshold
    }

    /**
     * 獲取 session 剩餘時間（秒）
     *
     * @param ttlMinutes - JWT TTL（分鐘）
     * @returns 剩餘秒數（負數表示已過期）
     */
    getSessionRemaining(ttlMinutes: number = 15): number {
        const ageSeconds = this.getSessionAge()
        const ttlSeconds = ttlMinutes * 60
        return ttlSeconds - ageSeconds
    }

    /**
     * 記錄 session 狀態到 console
     *
     * @param context - BrowserContext
     * @param label - 標籤（用於識別記錄點）
     */
    async logSessionState(context: BrowserContext, label: string = '', ttlMinutes: number = 15) {
        const cookies = await context.cookies()
        const accessToken = cookies.find((c) => c.name === 'access_token')

        const sessionAge = this.getSessionAge()
        const remaining = this.getSessionRemaining(ttlMinutes)
        const isNearExpiry = this.isSessionNearExpiry(ttlMinutes)

        console.log(`
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[Session Monitor] ${label || '狀態檢查'}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⏱️  時間資訊
  Session Age: ${sessionAge}s
  Remaining: ${remaining}s ${remaining < 0 ? '(已過期！)' : ''}
  Near Expiry: ${isNearExpiry ? '⚠️  YES' : '✅ NO'}

🔐 Token 資訊
  Has Token: ${accessToken ? '✅ YES' : '❌ NO'}
  Token Value: ${accessToken ? accessToken.value.substring(0, 20) + '...' : 'N/A'}
  Token Expires: ${accessToken?.expires ? new Date(accessToken.expires * 1000).toISOString() : 'N/A'}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
`)
    }

    /**
     * 簡短日誌（用於追蹤）
     */
    private log(message: string) {
        const timestamp = new Date().toISOString().substring(11, 23) // HH:mm:ss.SSS
        console.log(`[SessionMonitor ${timestamp}] ${message}`)
    }

    /**
     * 警告日誌（用於重要提示）
     */
    warn(message: string) {
        const timestamp = new Date().toISOString().substring(11, 23)
        console.warn(`[SessionMonitor ${timestamp}] ⚠️  ${message}`)
    }

    /**
     * 獲取格式化的 session 摘要
     */
    getSummary(ttlMinutes: number = 15): string {
        const age = this.getSessionAge()
        const remaining = this.getSessionRemaining(ttlMinutes)
        const isNearExpiry = this.isSessionNearExpiry(ttlMinutes)

        return [
            `Session Age: ${age}s`,
            `Remaining: ${remaining}s${remaining < 0 ? ' (已過期)' : ''}`,
            `Near Expiry: ${isNearExpiry ? 'YES ⚠️' : 'NO'}`,
        ].join(' | ')
    }

    /**
     * 檢查是否需要 refresh
     *
     * @param ttlMinutes - JWT TTL（分鐘）
     * @param refreshThreshold - Refresh 閾值（0-1，預設 0.5 = 50%）
     * @returns true 如果應該 refresh token
     */
    shouldRefresh(ttlMinutes: number = 15, refreshThreshold: number = 0.5): boolean {
        const ageSeconds = this.getSessionAge()
        const ttlSeconds = ttlMinutes * 60
        return ageSeconds > ttlSeconds * refreshThreshold
    }

    /**
     * 重置監控器（例如在重新登入後）
     */
    reset() {
        this.sessionStartTime = Date.now()
        this.testStartTime = Date.now()
        this.log('Session Monitor 已重置')
    }
}

/**
 * 全局 SessionMonitor 實例（可選）
 *
 * 用於跨測試追蹤 session 狀態（如果使用 worker 級別的 shared context）
 */
export const globalSessionMonitor = new SessionMonitor()
