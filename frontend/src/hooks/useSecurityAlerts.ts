// 安全警報即時訂閱 hook（SSE）
// 訂閱後端 SSE 端點，接收安全警報事件並顯示 toast 通知

import { useEffect, useRef, useCallback } from 'react'
import { useAuthStore } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'

interface AlertEvent {
    alert_type: string
    severity: string
    title: string
    description: string
}

/** 最大重試次數 */
const MAX_RETRIES = 5
/** 基礎重連延遲（毫秒） */
const BASE_DELAY_MS = 2000

/**
 * SSE 安全警報訂閱 hook
 * 僅在使用者為管理員時啟用。收到警報自動顯示 toast。
 * 連線斷開時使用指數退避自動重連（最多 MAX_RETRIES 次）。
 */
export function useSecurityAlerts() {
    const { user } = useAuthStore()
    const eventSourceRef = useRef<EventSource | null>(null)
    const retryCountRef = useRef(0)
    const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
    const isAdmin = user?.roles?.includes('ADMIN') || user?.roles?.includes('admin')

    const handleAlert = useCallback((event: MessageEvent) => {
        try {
            const alert: AlertEvent = JSON.parse(event.data)
            const variant = alert.severity === 'critical' ? 'destructive' as const : 'default' as const
            toast({
                title: `🔔 ${alert.title}`,
                description: alert.description,
                variant,
                duration: alert.severity === 'critical' ? 15000 : 8000,
            })
        } catch {
            // 忽略解析失敗
        }
    }, [])

    const connect = useCallback(() => {
        // 關閉既有連線
        if (eventSourceRef.current) {
            eventSourceRef.current.close()
            eventSourceRef.current = null
        }

        const url = '/api/admin/audit/alerts/sse'
        const es = new EventSource(url, { withCredentials: true })
        eventSourceRef.current = es

        es.addEventListener('security_alert', handleAlert)

        es.onopen = () => {
            // 連線成功，重置重試計數器
            retryCountRef.current = 0
        }

        es.onerror = () => {
            // SSE 連線中斷或逾時（如 524）時關閉後指數退避重連
            es.close()
            eventSourceRef.current = null

            if (retryCountRef.current < MAX_RETRIES) {
                const delay = BASE_DELAY_MS * Math.pow(2, retryCountRef.current)
                retryCountRef.current += 1
                retryTimerRef.current = setTimeout(connect, delay)
            }
            // 超過最大重試次數後靜默放棄，不刷 console
        }
    }, [handleAlert])

    useEffect(() => {
        // 僅管理員訂閱
        if (!isAdmin) return

        connect()

        return () => {
            // 清理：關閉 EventSource + 取消重連 timer
            if (retryTimerRef.current) {
                clearTimeout(retryTimerRef.current)
                retryTimerRef.current = null
            }
            if (eventSourceRef.current) {
                eventSourceRef.current.close()
                eventSourceRef.current = null
            }
            retryCountRef.current = 0
        }
    }, [isAdmin, connect])
}
