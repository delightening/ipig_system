import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { shouldPoll } from '@/lib/query'
import { queryKeys } from '@/lib/queryKeys'
import { notificationTargetPath } from '@/lib/notificationRoute'
import { useAuthStore } from '@/stores/auth'
import type { NotificationItem } from '@/types/notification'
import { Button } from '@/components/ui/button'
import { Loader2, CircleAlert, CheckCircle2, ExternalLink } from 'lucide-react'

/**
 * 驚嘆號入口：**待處理**（需要本人做出動作、且系統能判定完成的事）。
 *
 * 與鈴鐺（{@link ../NotificationDropdown}）的分工：
 * 鈴鐺是「看過就好」，這裡是「非做不可」。判準由後端 `?entry=todo` 決定，
 * 前端只說自己是哪個入口 —— 否則清單與紅點會各算各的而對不起來。
 *
 * **刻意沒有「標為已讀 / 全部已讀」按鈕。** 使用者裁定待辦只能由系統偵測動作
 * 完成後自動消失，不可手動略過；後端也擋掉了直接呼叫 API 的路。
 */
export function ActionRequiredDropdown() {
    const navigate = useNavigate()
    const { t, i18n } = useTranslation()
    const { user, isInitialized } = useAuthStore()

    const [showDropdown, setShowDropdown] = useState(false)
    const dropdownRef = useRef<HTMLDivElement>(null)

    const isLoggedIn = !!user && isInitialized

    const { data: pendingCount } = useQuery({
        queryKey: queryKeys.notifications.actionRequiredCount,
        queryFn: async () => {
            const res = await api.get<{ count: number }>('/notifications/action-required-count')
            return res.data.count
        },
        staleTime: 30_000,
        refetchInterval: () => shouldPoll(60_000),
        enabled: isLoggedIn,
    })

    const { data: items, isLoading } = useQuery({
        queryKey: queryKeys.notifications.actionRequired,
        queryFn: async () => {
            const res = await api.get<{ data: NotificationItem[] }>(
                '/notifications?entry=todo&per_page=10',
            )
            return res.data.data
        },
        enabled: isLoggedIn && showDropdown,
        staleTime: 30_000,
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

    const handleClick = (notification: NotificationItem) => {
        setShowDropdown(false)
        const path = notificationTargetPath(notification)
        if (path) navigate(path)
    }

    /** 待辦積了幾天 —— 待辦看的是「拖多久了」，不是「多久以前發生的」。 */
    const waitedDays = (dateStr: string) =>
        Math.floor((Date.now() - new Date(dateStr).getTime()) / 86_400_000)

    const hasPending = (pendingCount ?? 0) > 0

    return (
        <div className="relative" ref={dropdownRef}>
            <Button
                variant="ghost"
                size="icon"
                className="relative"
                onClick={() => setShowDropdown(!showDropdown)}
                aria-label={t('common.actionRequired')}
                aria-haspopup="menu"
                aria-expanded={showDropdown}
                data-testid="action-required-bell"
            >
                <CircleAlert className="h-5 w-5" />
                {hasPending && (
                    // error-strong 而非 error：白字配 --status-error-solid 只有 3.78:1，
                    // 12px 粗體屬 WCAG「一般文字」需 4.5:1；error-strong 是 4.67:1，過關且更沉。
                    // （DESIGN.md 變更紀錄第 14 項已把 solid + 白字的對比列為待收斂項。）
                    <span
                        className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center rounded-full bg-status-error-strong-solid text-white text-xs font-bold"
                        data-testid="action-required-badge"
                    >
                        {pendingCount! > 99 ? '99+' : pendingCount}
                    </span>
                )}
            </Button>

            {showDropdown && (
                <div className="absolute right-0 top-12 w-[calc(100vw-2rem)] md:w-96 max-w-sm bg-white rounded-lg shadow-xl border z-50 overflow-hidden">
                    <div className="px-4 py-3 border-b bg-muted">
                        <h3 className="font-semibold text-foreground flex items-center gap-2">
                            <CircleAlert className="h-4 w-4 text-status-error-solid" />
                            {t('common.actionRequired')}
                        </h3>
                        {/* 沒有「全部已讀」可按，先講清楚為什麼，免得使用者找按鈕找半天 */}
                        <p className="text-xs text-muted-foreground mt-1">
                            {t('common.actionRequiredHint')}
                        </p>
                    </div>

                    {/* overscroll-contain：捲到底不要把背後的頁面一起帶著捲 */}
                    <div className="max-h-[400px] overflow-y-auto overscroll-contain">
                        {isLoading ? (
                            <div className="px-4 py-8 text-center text-muted-foreground">
                                <Loader2 className="h-8 w-8 mx-auto mb-2 animate-spin" />
                                <p>{t('common.loading')}</p>
                            </div>
                        ) : items && items.length > 0 ? (
                            items.map((item) => {
                                const days = waitedDays(item.created_at)
                                return (
                                    // 用 <button> 而非 <div onClick>：列項的主要動作是導向，
                                    // div 不可聚焦也不吃 Enter/Space，鍵盤與螢幕閱讀器使用者
                                    // 完全打不開任何一則待辦。
                                    <button
                                        key={item.id}
                                        type="button"
                                        onClick={() => handleClick(item)}
                                        className="w-full text-left px-4 py-3 border-b last:border-b-0 cursor-pointer hover:bg-muted focus-visible:bg-muted focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset transition-colors border-l-4 border-l-status-error-solid"
                                    >
                                        <div className="flex items-start gap-3">
                                            <div className="flex-1 min-w-0">
                                                <p className="text-sm font-semibold text-foreground">
                                                    {item.title}
                                                </p>
                                                {item.content && (
                                                    <p className="text-sm text-muted-foreground truncate mt-0.5">
                                                        {item.content}
                                                    </p>
                                                )}
                                                <p className="text-xs text-muted-foreground mt-1">
                                                    {days >= 1
                                                        ? t('common.waitingDays', { count: days })
                                                        : new Date(item.created_at).toLocaleString(
                                                              i18n.language,
                                                              {
                                                                  timeZone: 'Asia/Taipei',
                                                                  month: '2-digit',
                                                                  day: '2-digit',
                                                                  hour: '2-digit',
                                                                  minute: '2-digit',
                                                              },
                                                          )}
                                                </p>
                                            </div>
                                            {notificationTargetPath(item) && (
                                                <ExternalLink className="h-4 w-4 text-muted-foreground shrink-0 mt-1" />
                                            )}
                                        </div>
                                    </button>
                                )
                            })
                        ) : (
                            // 零狀態刻意用完成色而非灰色：「沒有待辦」是好消息，不是空狀態
                            <div className="px-4 py-8 text-center text-muted-foreground">
                                <CheckCircle2 className="h-8 w-8 mx-auto mb-2 text-status-success-solid" />
                                <p>{t('common.noActionRequired')}</p>
                            </div>
                        )}
                    </div>

                    {items && items.length > 0 && (
                        <div className="px-4 py-2 border-t bg-muted">
                            <button
                                onClick={() => {
                                    setShowDropdown(false)
                                    navigate('/notifications?entry=todo')
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
