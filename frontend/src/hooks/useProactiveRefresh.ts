import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/stores/auth'
import { isTabIdle } from '@/lib/tabActivity'

/**
 * Proactive Refresh Hook（sliding session A2 + D1）
 *
 * 在 access token 過期前 silent refresh，消除 reactive 401 → /auth/refresh →
 * retry 的 200ms 卡頓。配合後端 R35-15 refresh token rotation 與 R46 reuse
 * detection — 每次 refresh 拿新的 access + refresh token，鏈條一致。
 *
 * 排程策略：在剩餘 TTL 的 80% 處 fire（access TTL 15 min → 12 min 後 refresh，
 * token 仍有 3 min buffer 給 refresh 請求完成）。
 *
 * Bootstrap：頁面 reload 後 accessTokenExpiresAt 為 null（不持久化），
 * 立即觸發一次 refresh 建立 baseline；後續由 store 更新自動串接下一輪 timer。
 *
 * Multi-tab：每個 tab 各自跑 timer，B1 (BroadcastChannel) 會做跨 tab 協調，
 * 此前由 api/client.ts 的 Promise singleton + 後端 R46 race window (5s) 兜底。
 *
 * D1: visibilitychange — 筆電從睡眠醒來 / 切回分頁時，setTimeout 可能因系統
 * suspend 而延後；若 token 已過期或臨近過期，立即主動 refresh 避免下次
 * user 操作落到 reactive 401 flow。
 */

const REFRESH_BUFFER_RATIO = 0.8
const MIN_REFRESH_DELAY_MS = 10_000
const MAX_REFRESH_DELAY_MS = 60 * 60 * 1000
/** D1: 切回分頁時剩餘 TTL 低於此閾值就立即 refresh */
const VISIBILITY_REFRESH_THRESHOLD_MS = 30_000

export function useProactiveRefresh(isAuthenticated: boolean) {
    const accessTokenExpiresAt = useAuthStore((s) => s.accessTokenExpiresAt)
    const refreshSession = useAuthStore((s) => s.refreshSession)
    // 模擬登入使用純 access-only token（後端不簽發 refresh token，見 session.rs impersonate），
    // 主動續期打 /auth/refresh 必得 400。頁面 reload 後 accessTokenExpiresAt 為 null 會讓
    // bootstrap 迴圈式重試，持續擾動 auth 狀態並在 RequirePermission 造成偽「無權限」。
    // 模擬登入時一律跳過主動續期；token 到期由 reactive 401 flow 處理（登出恢復管理員）。
    const isImpersonating = useAuthStore((s) => s.isImpersonating)
    // Bootstrap in-flight guard — accessTokenExpiresAt 從 null → number 期間
    // 任何 rerender 重跑 effect 都會再次觸發 refresh。用 ref-based flag 確保
    // bootstrap 只 fire 一次。來源：CodeRabbit review on PR #428 (Major)。
    const bootstrapRefreshingRef = useRef(false)

    // 主排程：在剩餘 TTL 80% 處 fire（或 bootstrap / immediate）
    useEffect(() => {
        if (!isAuthenticated) return
        if (isImpersonating) return

        if (accessTokenExpiresAt == null) {
            if (isTabIdle()) return
            if (bootstrapRefreshingRef.current) return
            bootstrapRefreshingRef.current = true
            // Bootstrap：頁面 reload / 初次掛載沒有 baseline → 立即 refresh
            // 失敗交給 client.ts interceptor 處理（logout flow）
            void refreshSession().finally(() => {
                bootstrapRefreshingRef.current = false
            })
            return
        }

        const ttlRemaining = accessTokenExpiresAt - Date.now()
        if (ttlRemaining <= MIN_REFRESH_DELAY_MS) {
            void refreshSession()
            return
        }

        const delay = Math.min(
            Math.max(ttlRemaining * REFRESH_BUFFER_RATIO, MIN_REFRESH_DELAY_MS),
            MAX_REFRESH_DELAY_MS,
        )

        const timer = setTimeout(() => {
            if (isTabIdle()) {
                useAuthStore.getState().clearAuthLocal()
                return
            }
            void refreshSession()
        }, delay)

        return () => clearTimeout(timer)
    }, [isAuthenticated, accessTokenExpiresAt, refreshSession, isImpersonating])

    // D1: visibilitychange — tab 回前景時若 token 快過期 / 已過期，先 refresh
    useEffect(() => {
        if (!isAuthenticated) return
        if (isImpersonating) return

        const onVisibilityChange = () => {
            if (document.hidden) return
            if (isTabIdle()) {
                useAuthStore.getState().clearAuthLocal()
                return
            }
            const expiry = useAuthStore.getState().accessTokenExpiresAt
            if (expiry == null) return
            const ttlRemaining = expiry - Date.now()
            if (ttlRemaining < VISIBILITY_REFRESH_THRESHOLD_MS) {
                void refreshSession()
            }
        }

        document.addEventListener('visibilitychange', onVisibilityChange)
        return () => document.removeEventListener('visibilitychange', onVisibilityChange)
    }, [isAuthenticated, refreshSession, isImpersonating])
}
