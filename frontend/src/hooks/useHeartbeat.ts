import { useEffect, useRef, useCallback } from 'react'
import api from '@/lib/api'

/**
 * Heartbeat Hook
 * 監聽使用者在頁面上的活動（滑鼠、鍵盤、點擊、滾動、觸控），
 * 每 60 秒發送一次 heartbeat 到後端更新 session 的最後活動時間與 IP。
 */
export function useHeartbeat(isAuthenticated: boolean) {
    const hasActivityRef = useRef(false)
    const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

    // 記錄使用者活動
    const markActivity = useCallback(() => {
        hasActivityRef.current = true
    }, [])

    useEffect(() => {
        if (!isAuthenticated) return

        // 監聽使用者活動事件
        const events = ['mousemove', 'keydown', 'click', 'scroll', 'touchstart'] as const
        events.forEach((event) => {
            window.addEventListener(event, markActivity, { passive: true })
        })

        // 每 60 秒檢查是否有活動，若有則發送 heartbeat
        intervalRef.current = setInterval(async () => {
            if (hasActivityRef.current) {
                hasActivityRef.current = false
                try {
                    await api.post('/auth/heartbeat')
                } catch {
                    // 靜默失敗，不影響使用者操作
                }
            }
        }, 60_000)

        // 首次載入立即發送一次
        api.post('/auth/heartbeat').catch(() => { })

        return () => {
            events.forEach((event) => {
                window.removeEventListener(event, markActivity)
            })
            if (intervalRef.current) {
                clearInterval(intervalRef.current)
                intervalRef.current = null
            }
        }
    }, [isAuthenticated, markActivity])
}
