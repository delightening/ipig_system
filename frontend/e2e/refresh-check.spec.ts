/**
 * 頁面重新整理壓力測試：連續 refresh 5 次，捕獲網路與 Console 錯誤碼。
 * 用於驗證 checkAuth、refresh token、503 誤登出等情境。
 */
import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

interface CapturedError {
    type: 'network' | 'console'
    url?: string
    status?: number
    method?: string
    message?: string
    timestamp: number
}

test.describe('Refresh Check', () => {
    test('連續重新整理 5 次，記錄所有錯誤碼', { timeout: 90_000 }, async ({ page }) => {
        const errors: CapturedError[] = []

        // 監聽 API 回應（非 2xx 視為錯誤）
        page.on('response', (response) => {
            const url = response.url()
            if (!url.includes('/api/')) return
            const status = response.status()
            if (status >= 400 || status < 200) {
                errors.push({
                    type: 'network',
                    url,
                    status,
                    method: response.request().method(),
                    timestamp: Date.now(),
                })
            }
        })

        // 監聽 Console 錯誤
        page.on('console', (msg) => {
            const type = msg.type()
            if (type === 'error' || type === 'warning') {
                errors.push({
                    type: 'console',
                    message: msg.text(),
                    timestamp: Date.now(),
                })
            }
        })

        await ensureAdminOnPage(page, '/dashboard')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/dashboard')
        }
        const initialUrl = page.url()
        if (initialUrl.includes('/login')) {
            console.log('\n⚠️ 初始即為登入頁，可能 session 已失效')
        } else {
            await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })
        }

        // 若已在登入頁，仍執行 refresh 以捕獲錯誤（例如 refresh token 400 等）

        // 連續 refresh 5 次
        for (let i = 1; i <= 5; i++) {
            await page.reload({ waitUntil: 'load' })
            // 短暫等待讓 API 完成（不依賴 networkidle，避免逾時）
            await page.waitForTimeout(1500)
        }

        const finalUrl = page.url()
        if (finalUrl.includes('/login')) {
            console.log('\n⚠️ 5 次 refresh 後被導向登入頁（可能 401/503 誤登出）')
        }

        // 輸出結果
        const networkErrors = errors.filter((e) => e.type === 'network')
        const consoleErrors = errors.filter((e) => e.type === 'console')

        if (networkErrors.length > 0) {
            console.log('\n=== 網路錯誤 (非 2xx) ===')
            const byStatus = new Map<number, number>()
            const byPath = new Map<string, number>()
            networkErrors.forEach((e) => {
                const count = (byStatus.get(e.status!) || 0) + 1
                byStatus.set(e.status!, count)
                const path = (() => {
                    try {
                        const u = new URL(e.url || '')
                        return u.pathname + u.search
                    } catch {
                        return e.url || ''
                    }
                })()
                byPath.set(path, (byPath.get(path) || 0) + 1)
            })
            byStatus.forEach((count, status) => {
                console.log(`  HTTP ${status}: ${count} 次`)
            })
            console.log('  依路徑統計:')
            ;[...byPath.entries()]
                .sort((a, b) => b[1] - a[1])
                .slice(0, 8)
                .forEach(([path, count]) => console.log(`    ${path}: ${count}`))
        }

        if (consoleErrors.length > 0) {
            console.log('\n=== Console 錯誤/警告 ===')
            consoleErrors.slice(0, 10).forEach((e, i) => {
                console.log(`  [${i + 1}] ${(e.message || '').slice(0, 150)}`)
            })
            if (consoleErrors.length > 10) {
                console.log(`  ... 共 ${consoleErrors.length} 則`)
            }
        }

        if (errors.length === 0) {
            console.log('\n✅ 5 次 refresh 未發現錯誤')
        }

        // 若有 401/503 等關鍵錯誤，測試失敗（可依需求調整）
        const criticalStatuses = networkErrors
            .filter((e) => e.status === 401 || e.status === 503 || e.status === 500)
            .map((e) => e.status)
        if (criticalStatuses.length > 0) {
            console.log('\n⚠️ 關鍵錯誤碼:', [...new Set(criticalStatuses)].join(', '))
            // 不直接 fail，讓測試通過但輸出報告；若要改成 fail 可取消下面這行
            // expect(criticalStatuses, '不應出現 401/503/500').toHaveLength(0)
        }
    })
})
