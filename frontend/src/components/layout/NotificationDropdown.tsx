import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import api from '@/lib/api'
import { shouldPoll } from '@/lib/query'
import { queryKeys } from '@/lib/queryKeys'
import { notificationTargetPath } from '@/lib/notificationRoute'
import { useAuthStore } from '@/stores/auth'
import type { NotificationItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
  Loader2,
  Bell,
  CheckCheck,
  CheckCircle2,
  ExternalLink,
} from 'lucide-react'

/**
 * 鈴鐺入口：**通知**（看過就好，不需要動作）。
 *
 * 未完成待辦已分家到 {@link ./ActionRequiredDropdown}，這裡不再出現，
 * 免得同一件事在畫面上被數兩次。但**已完成**的待辦仍留在這裡當歷史 ——
 * 那是「我當初處理過哪些事」，所以入口判準是 `?entry=bell` 而非 `kind=info`。
 */
export function NotificationDropdown() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { t, i18n } = useTranslation()
  const { user, isInitialized } = useAuthStore()

  const [showDropdown, setShowDropdown] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  const isLoggedIn = !!user && isInitialized

  const { data: unreadCount } = useQuery({
    queryKey: queryKeys.notifications.unreadCount,
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/notifications/unread-count')
      return res.data.count
    },
    staleTime: 30_000,
    refetchInterval: () => shouldPoll(60_000),
    enabled: isLoggedIn,
  })

  const { data: notificationsData, isLoading: isLoadingNotifications } = useQuery({
    queryKey: queryKeys.notifications.recent,
    queryFn: async () => {
      const res = await api.get<{ data: NotificationItem[] }>(
        '/notifications?entry=bell&per_page=10',
      )
      return res.data.data
    },
    enabled: isLoggedIn && showDropdown,
    staleTime: 30_000,
  })

  const markReadMutation = useMutation({
    mutationFn: async (ids: string[]) => {
      return api.post('/notifications/read', { notification_ids: ids })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications-unread-count'] })
      queryClient.invalidateQueries({ queryKey: ['notifications-recent'] })
    },
  })

  const markAllReadMutation = useMutation({
    mutationFn: async () => {
      return api.post('/notifications/read-all')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications-unread-count'] })
      queryClient.invalidateQueries({ queryKey: ['notifications-recent'] })
      toast({ title: t('common.success'), description: t('common.saved') })
    },
  })

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false)
      }
    }
    // 只靠「點外面」關閉的話，鍵盤使用者被困在展開的選單裡沒有出路
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setShowDropdown(false)
    }
    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleEscape)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [])

  const handleNotificationClick = (notification: NotificationItem) => {
    if (!notification.is_read) {
      markReadMutation.mutate([notification.id])
    }
    setShowDropdown(false)

    const path = notificationTargetPath(notification)
    if (path) navigate(path)
  }

  const formatNotificationTime = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)

    if (minutes < 1) return t('common.justNow')
    if (minutes < 60) return t('common.minutesAgo', { count: minutes })
    if (hours < 24) return t('common.hoursAgo', { count: hours })
    if (days < 7) return t('common.daysAgo', { count: days })
    return date.toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei' })
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <Button
        variant="ghost"
        size="icon"
        className="relative"
        onClick={() => setShowDropdown(!showDropdown)}
        aria-label="通知"
        aria-haspopup="menu"
        aria-expanded={showDropdown}
        data-testid="notification-bell"
      >
        <Bell className="h-5 w-5" />
        {/* 灰而非紅：用顏色分輕重 —— 紅色留給隔壁「非做不可」的待處理，
            這裡只是「還沒看過」。兩邊都紅等於回到「什麼都很急」。 */}
        {unreadCount && unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center rounded-full bg-status-neutral-solid text-white text-xs font-bold">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </Button>

      {showDropdown && (
        <div className="absolute right-0 top-12 w-[calc(100vw-2rem)] md:w-96 max-w-sm bg-white rounded-lg shadow-xl border z-50 overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b bg-muted">
            <h3 className="font-semibold text-foreground">{t('common.notifications')}</h3>
            {unreadCount && unreadCount > 0 && (
              <button
                onClick={() => markAllReadMutation.mutate()}
                className="text-sm text-status-info-text hover:text-status-info-text flex items-center gap-1"
                disabled={markAllReadMutation.isPending}
              >
                <CheckCheck className="h-4 w-4" />
                {t('common.markAllRead')}
              </button>
            )}
          </div>

          {/* overscroll-contain：下拉清單捲到底不要把背後的頁面一起帶著捲
              （useScrollChaining 尊重此標準屬性，不接力被標記的容器） */}
          <div className="max-h-[400px] overflow-y-auto overscroll-contain">
            {isLoadingNotifications ? (
              <div className="px-4 py-8 text-center text-muted-foreground">
                <Loader2 className="h-8 w-8 mx-auto mb-2 text-muted-foreground animate-spin" />
                <p>{t('common.loading')}</p>
              </div>
            ) : notificationsData && notificationsData.length > 0 ? (
              notificationsData.map((notification) => {
                // 已完成的待辦：留在這裡當「我當初處理過哪些事」的歷史。
                // 未完成的不會出現（?entry=bell 已在後端排除），所以不再有置頂樣式。
                const isResolvedAction = notification.kind === 'action'
                return (
                // 用 <button> 而非 <div onClick>：div 不可聚焦也不吃 Enter/Space，
                // 鍵盤與螢幕閱讀器使用者打不開任何一則通知
                <button
                  key={notification.id}
                  type="button"
                  onClick={() => handleNotificationClick(notification)}
                  className={cn(
                    "w-full text-left px-4 py-3 border-b last:border-b-0 cursor-pointer hover:bg-muted focus-visible:bg-muted focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset transition-colors",
                    !notification.is_read && "bg-status-info-bg"
                  )}
                >
                  <div className="flex items-start gap-3">
                    <div className={cn(
                      "w-2 h-2 rounded-full mt-2 shrink-0",
                      // 原本寫成 bg-status-info-bg0（不存在的 class），未讀圓點一直沒顯示
                      !notification.is_read ? "bg-status-info-solid" : "bg-transparent"
                    )} />
                    <div className="flex-1 min-w-0">
                      {isResolvedAction && (
                        <span className="inline-flex items-center gap-1 mb-1 text-xs font-medium px-1.5 py-0.5 rounded bg-status-success-bg text-status-success-text">
                          <CheckCircle2 className="h-3 w-3" />
                          {t('common.actionCompleted')}
                        </span>
                      )}
                      <p className={cn(
                        "text-sm truncate",
                        !notification.is_read ? "font-semibold text-foreground" : "text-foreground"
                      )}>
                        {notification.title}
                      </p>
                      {notification.content && (
                        <p className="text-sm text-muted-foreground truncate mt-0.5">
                          {notification.content}
                        </p>
                      )}
                      <p className="text-xs text-muted-foreground mt-1">
                        {formatNotificationTime(notification.created_at)}
                      </p>
                    </div>
                    {notificationTargetPath(notification) && (
                      <ExternalLink className="h-4 w-4 text-muted-foreground shrink-0 mt-1" />
                    )}
                  </div>
                </button>
                )
              })
            ) : (
              <div className="px-4 py-8 text-center text-muted-foreground">
                <Bell className="h-8 w-8 mx-auto mb-2 text-muted-foreground" />
                <p>{t('common.noNotifications')}</p>
              </div>
            )}
          </div>

          {notificationsData && notificationsData.length > 0 && (
            <div className="px-4 py-2 border-t bg-muted">
              <button
                onClick={() => {
                  setShowDropdown(false)
                  navigate('/notifications')
                }}
                className="text-sm text-status-info-text hover:underline w-full text-center"
              >
                {t('common.viewAll')}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
