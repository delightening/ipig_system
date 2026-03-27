// 安全警報 polling hook
// 每 30 秒輪詢後端，偵測新警報並顯示 toast 通知

import { useEffect, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import api from '@/lib/api'
import { shouldPoll } from '@/lib/query'
import { toast } from '@/components/ui/use-toast'

interface SecurityAlert {
  id: string
  alert_type: string
  severity: string
  title: string
  description: string | null
  created_at: string
}

/** Polling 間隔（毫秒） */
const POLL_INTERVAL_MS = 30_000

/**
 * 安全警報 polling hook
 * 僅在使用者為管理員時啟用。偵測到新警報自動顯示 toast。
 */
export function useSecurityAlerts() {
  const { user } = useAuthStore()
  const isAdmin = user?.roles?.includes('ADMIN') || user?.roles?.includes('admin')

  // 追蹤已顯示 toast 的警報 ID，避免重複通知
  const shownIdsRef = useRef<Set<string>>(new Set())
  // 追蹤是否為首次載入（首次不彈 toast）
  const isFirstFetchRef = useRef(true)

  const { data: alerts } = useQuery<SecurityAlert[]>({
    queryKey: ['security-alerts-recent'],
    queryFn: async () => {
      // 查詢最近 60 秒的警報
      const after = new Date(Date.now() - 60_000).toISOString()
      const res = await api.get<SecurityAlert[]>('/admin/audit/alerts/recent', {
        params: { after },
      })
      return res.data
    },
    enabled: !!isAdmin,
    refetchInterval: () => shouldPoll(POLL_INTERVAL_MS),
    staleTime: POLL_INTERVAL_MS,
    retry: false,
  })

  useEffect(() => {
    if (!alerts || alerts.length === 0) return

    // 首次載入：記錄現有警報 ID，不彈 toast
    if (isFirstFetchRef.current) {
      isFirstFetchRef.current = false
      for (const alert of alerts) {
        shownIdsRef.current.add(alert.id)
      }
      return
    }

    // 後續輪詢：只對未顯示過的新警報彈 toast
    for (const alert of alerts) {
      if (shownIdsRef.current.has(alert.id)) continue
      shownIdsRef.current.add(alert.id)

      const variant = alert.severity === 'critical' ? 'destructive' as const : 'default' as const
      toast({
        title: `🔔 ${alert.title}`,
        description: alert.description ?? undefined,
        variant,
        duration: alert.severity === 'critical' ? 15000 : 8000,
      })
    }

    // 防止 Set 無限增長：只保留最近的 ID
    if (shownIdsRef.current.size > 200) {
      const ids = Array.from(shownIdsRef.current)
      shownIdsRef.current = new Set(ids.slice(-100))
    }
  }, [alerts])
}
