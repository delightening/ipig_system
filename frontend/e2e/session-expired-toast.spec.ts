/**
 * Sliding session overhaul — F7 acceptance test.
 *
 * 確保 401 → /login?reason=session_expired 跳轉後，LoginPage 顯示中性灰色 toast，
 * 而非紅色 destructive 警告（避免被誤認為錯誤）。
 *
 * 真實「等 8h idle」場景不在 e2e 範圍（CI 不可能等 8 小時）；改在 backend 整合
 * 測試 `api_session_heartbeat.rs` 驗證 sliding session 機制本體。
 */
import { test, expect } from './fixtures/coverage'

test.use({ storageState: { cookies: [], origins: [] } })

test.describe('Session expired toast (F7)', () => {
    test('?reason=session_expired 顯示中性灰色 toast 且 URL 被清掉', async ({ page }) => {
        await page.goto('/login?reason=session_expired')
        await page.waitForLoadState('domcontentloaded')

        // Toast 顯示
        const toastTitle = page.getByText('登入時效已到期').first()
        await expect(toastTitle).toBeVisible({ timeout: 5_000 })

        // Toast 不是 destructive (紅色) — toast root 不應帶 bg-destructive class
        const toastRoot = page.locator('[data-state="open"]').filter({ hasText: '登入時效已到期' }).first()
        await expect(toastRoot).not.toHaveClass(/bg-destructive/)
        await expect(toastRoot).not.toHaveClass(/\bdestructive\b/)

        // URL param 被清掉，避免重整後又彈一次
        await expect(page).toHaveURL((url) => !url.search.includes('reason=session_expired'), {
            timeout: 5_000,
        })
    })

    test('沒有 reason param 時不顯示 toast', async ({ page }) => {
        await page.goto('/login')
        await page.waitForLoadState('domcontentloaded')

        // 給 3 秒看會不會彈出，理論上不會
        const toastTitle = page.getByText('登入時效已到期')
        await expect(toastTitle).toHaveCount(0, { timeout: 3_000 })
    })
})
