import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import api from '@/lib/api'
import { shouldPoll } from '@/lib/query'
import { useAuthStore } from '@/stores/auth'
import type { NotificationItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
  Loader2,
  Bell,
  CheckCheck,
  ExternalLink,
} from 'lucide-react'

export function NotificationDropdown() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { t, i18n } = useTranslation()
  const { user } = useAuthStore()

  const [showDropdown, setShowDropdown] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  const isLoggedIn = !!user

  const { data: unreadCount } = useQuery({
    queryKey: ['notifications-unread-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/notifications/unread-count')
      return res.data.count
    },
    staleTime: 30_000,
    refetchInterval: () => shouldPoll(60_000),
    enabled: isLoggedIn,
  })

  const { data: notificationsData, isLoading: isLoadingNotifications } = useQuery({
    queryKey: ['notifications-recent'],
    queryFn: async () => {
      const res = await api.get<{ data: NotificationItem[] }>('/notifications?per_page=10')
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
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleNotificationClick = (notification: NotificationItem) => {
    if (!notification.is_read) {
      markReadMutation.mutate([notification.id])
    }
    setShowDropdown(false)

    if (notification.related_entity_type) {
      switch (notification.related_entity_type) {
        case 'protocol':
          navigate(`/protocols/${notification.related_entity_id}`)
          break
        case 'document':
          navigate(`/documents/${notification.related_entity_id}`)
          break
        case 'animal':
          navigate(`/animals/${notification.related_entity_id}`)
          break
        case 'amendment':
          navigate(`/protocols/amendments/${notification.related_entity_id}`)
          break
        case 'leave_request':
          navigate('/hr/leaves')
          break
        case 'overtime_record':
        case 'overtime':
          navigate('/hr/overtime')
          break
        case 'euthanasia_order':
        case 'euthanasia_appeal':
          if (notification.related_entity_id) {
            navigate(`/animals/${notification.related_entity_id}`)
          }
          break
        case 'expiry_warning':
          navigate('/inventory?filter=expiry_warning')
          break
        case 'low_stock':
          navigate('/inventory')
          break
        case 'equipment':
          navigate('/equipment')
          break
        case 'report':
          navigate('/admin/settings')
          break
      }
    }
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
        data-testid="notification-bell"
      >
        <Bell className="h-5 w-5" />
        {unreadCount && unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center rounded-full bg-red-500 text-white text-xs font-bold">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </Button>

      {showDropdown && (
        <div className="absolute right-0 top-12 w-[calc(100vw-2rem)] md:w-96 max-w-sm bg-white rounded-lg shadow-xl border z-50 overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b bg-slate-50">
            <h3 className="font-semibold text-slate-900">{t('common.notifications')}</h3>
            {unreadCount && unreadCount > 0 && (
              <button
                onClick={() => markAllReadMutation.mutate()}
                className="text-sm text-blue-600 hover:text-blue-800 flex items-center gap-1"
                disabled={markAllReadMutation.isPending}
              >
                <CheckCheck className="h-4 w-4" />
                {t('common.markAllRead')}
              </button>
            )}
          </div>

          <div className="max-h-[400px] overflow-y-auto">
            {isLoadingNotifications ? (
              <div className="px-4 py-8 text-center text-slate-500">
                <Loader2 className="h-8 w-8 mx-auto mb-2 text-slate-300 animate-spin" />
                <p>{t('common.loading')}</p>
              </div>
            ) : notificationsData && notificationsData.length > 0 ? (
              notificationsData.map((notification) => (
                <div
                  key={notification.id}
                  onClick={() => handleNotificationClick(notification)}
                  className={cn(
                    "px-4 py-3 border-b last:border-b-0 cursor-pointer hover:bg-slate-50 transition-colors",
                    !notification.is_read && "bg-blue-50"
                  )}
                >
                  <div className="flex items-start gap-3">
                    <div className={cn(
                      "w-2 h-2 rounded-full mt-2 shrink-0",
                      !notification.is_read ? "bg-blue-500" : "bg-transparent"
                    )} />
                    <div className="flex-1 min-w-0">
                      <p className={cn(
                        "text-sm truncate",
                        !notification.is_read ? "font-semibold text-slate-900" : "text-slate-700"
                      )}>
                        {notification.title}
                      </p>
                      {notification.content && (
                        <p className="text-sm text-slate-500 truncate mt-0.5">
                          {notification.content}
                        </p>
                      )}
                      <p className="text-xs text-slate-400 mt-1">
                        {formatNotificationTime(notification.created_at)}
                      </p>
                    </div>
                    {notification.related_entity_type && (
                      <ExternalLink className="h-4 w-4 text-slate-400 shrink-0 mt-1" />
                    )}
                  </div>
                </div>
              ))
            ) : (
              <div className="px-4 py-8 text-center text-slate-500">
                <Bell className="h-8 w-8 mx-auto mb-2 text-slate-300" />
                <p>{t('common.noNotifications')}</p>
              </div>
            )}
          </div>

          {notificationsData && notificationsData.length > 0 && (
            <div className="px-4 py-2 border-t bg-slate-50">
              <button
                onClick={() => {
                  setShowDropdown(false)
                  navigate('/admin/settings')
                }}
                className="text-sm text-blue-600 hover:text-blue-800 w-full text-center"
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
