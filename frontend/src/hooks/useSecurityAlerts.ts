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

/**
 * SSE 安全警報訂閱 hook
 * 僅在使用者為管理員時啟用。收到警報自動顯示 toast。
 */
export function useSecurityAlerts() {
    const { user } = useAuthStore()
    const eventSourceRef = useRef<EventSource | null>(null)
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

    useEffect(() => {
        // 僅管理員訂閱
        if (!isAdmin) return

        // 使用相對路徑，讓 Vite proxy 或 cookie 處理身份認證
        const url = '/api/admin/audit/alerts/sse'
        const es = new EventSource(url, { withCredentials: true })
        eventSourceRef.current = es

        es.addEventListener('security_alert', handleAlert)

        es.onerror = () => {
            // SSE 連線中斷時瀏覽器會自動重連
        }

        return () => {
            es.close()
            eventSourceRef.current = null
        }
    }, [isAdmin, handleAlert])
}
