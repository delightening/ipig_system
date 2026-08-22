import { useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { cn, uiLocale } from '@/lib/utils'
import { notificationTargetPath } from '@/lib/notificationRoute'
import { queryKeys } from '@/lib/queryKeys'
import type { NotificationItem, NotificationEntry } from '@/types/notification'
import { notificationTypeNames } from '@/types/notification'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
    Bell, CircleAlert, CheckCheck, CheckCircle2, ExternalLink, Loader2,
    ChevronLeft, ChevronRight,
} from 'lucide-react'

const PER_PAGE = 20

/** tab 與其面板的關聯 id（role="tab" 需要 aria-controls 指向 role="tabpanel"） */
const TAB_PANEL_ID = 'notifications-panel'

/** 網址上只認這兩個值，其餘（含沒帶）一律退回鈴鐺 */
const isEntry = (v: string | null): v is NotificationEntry => v === 'bell' || v === 'todo'

export default function NotificationsPage() {
    const navigate = useNavigate()
    const queryClient = useQueryClient()
    const [searchParams, setSearchParams] = useSearchParams()
    const [page, setPage] = useState(1)

    // 分頁狀態放網址：驚嘆號下拉的「查看全部」要能直接連到待處理分頁，
    // 使用者也才能把「我的待辦」這個畫面加書籤 / 貼給別人。
    const entryParam = searchParams.get('entry')
    const entry: NotificationEntry = isEntry(entryParam) ? entryParam : 'bell'
    const isTodo = entry === 'todo'

    const switchEntry = (next: NotificationEntry) => {
        setPage(1)
        setSearchParams(next === 'bell' ? {} : { entry: next })
    }

    const { data, isLoading } = useQuery({
        queryKey: [...queryKeys.notifications.all, 'page', entry, page],
        queryFn: async () => {
            const res = await api.get<{ data: NotificationItem[]; total: number; page: number; per_page: number }>(
                `/notifications?entry=${entry}&page=${page}&per_page=${PER_PAGE}`,
            )
            return res.data
        },
    })

    // 標為已讀後，兩個入口的紅點都要重算：鈴鐺的未讀數變了，
    // 而待處理雖然不受已讀影響，清單仍可能同時被別的動作解除。
    const invalidateAll = () => {
        for (const key of [
            queryKeys.notifications.all,
            queryKeys.notifications.unreadCount,
            queryKeys.notifications.recent,
            queryKeys.notifications.actionRequired,
            queryKeys.notifications.actionRequiredCount,
        ]) {
            queryClient.invalidateQueries({ queryKey: key })
        }
    }

    const markReadMutation = useMutation({
        mutationFn: async (ids: string[]) => api.post('/notifications/read', { notification_ids: ids }),
        onSuccess: invalidateAll,
    })

    const markAllReadMutation = useMutation({
        mutationFn: async () => api.post('/notifications/read-all'),
        onSuccess: () => {
            invalidateAll()
            toast({ title: '已全部標記為已讀' })
        },
    })

    const handleClick = (n: NotificationItem) => {
        // 待辦不因為「看過」而改變狀態，只有系統偵測到動作完成才會消失
        if (!isTodo && !n.is_read) markReadMutation.mutate([n.id])
        const path = notificationTargetPath(n)
        if (path) navigate(path)
    }

    const formatTime = (dateStr: string) => {
        const d = new Date(dateStr)
        return d.toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
    }

    const waitedDays = (dateStr: string) =>
        Math.floor((Date.now() - new Date(dateStr).getTime()) / 86_400_000)

    const notifications = data?.data ?? []
    const total = data?.total ?? 0
    const totalPages = Math.ceil(total / PER_PAGE)
    const unreadCount = notifications.filter(n => !n.is_read).length

    return (
        <div className="p-4 md:p-6 max-w-3xl mx-auto space-y-4">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold flex items-center gap-2">
                        {isTodo ? <CircleAlert className="h-6 w-6" /> : <Bell className="h-6 w-6" />}
                        {isTodo ? '待處理' : '通知中心'}
                    </h1>
                    <p className="text-sm text-muted-foreground mt-1">
                        {isTodo
                            ? total > 0
                                ? `${total} 件待你處理，完成後會自動消失`
                                : '目前沒有待處理事項'
                            : `共 ${total} 筆通知`}
                    </p>
                </div>
                {/* 待處理分頁刻意沒有「全部標記已讀」：待辦不可手動略過 */}
                {!isTodo && unreadCount > 0 && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => markAllReadMutation.mutate()}
                        disabled={markAllReadMutation.isPending}
                    >
                        <CheckCheck className="h-4 w-4 mr-1" />
                        全部標記已讀
                    </Button>
                )}
            </div>

            <div className="flex gap-1 border-b" role="tablist">
                {([
                    { key: 'bell' as const, label: '通知', icon: Bell },
                    { key: 'todo' as const, label: '待處理', icon: CircleAlert },
                ]).map(({ key, label, icon: Icon }) => (
                    <button
                        key={key}
                        role="tab"
                        id={`notif-tab-${key}`}
                        aria-selected={entry === key}
                        aria-controls={TAB_PANEL_ID}
                        onClick={() => switchEntry(key)}
                        className={cn(
                            'flex items-center gap-1.5 px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors',
                            entry === key
                                ? 'border-primary text-foreground'
                                : 'border-transparent text-muted-foreground hover:text-foreground',
                        )}
                    >
                        <Icon className="h-4 w-4" />
                        {label}
                    </button>
                ))}
            </div>

            <div
                id={TAB_PANEL_ID}
                role="tabpanel"
                aria-labelledby={`notif-tab-${entry}`}
                className="border rounded-lg overflow-hidden divide-y"
            >
                {isLoading ? (
                    <div className="p-8 text-center text-muted-foreground">
                        <Loader2 className="h-6 w-6 animate-spin mx-auto mb-2" />
                        載入中...
                    </div>
                ) : notifications.length === 0 ? (
                    <div className="p-8 text-center text-muted-foreground">
                        {isTodo ? (
                            <>
                                {/* 「沒有待辦」是好消息，用完成色而非灰色空狀態 */}
                                <CheckCircle2 className="h-8 w-8 mx-auto mb-2 text-status-success-solid" />
                                目前沒有待處理事項
                            </>
                        ) : (
                            <>
                                <Bell className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                尚無通知
                            </>
                        )}
                    </div>
                ) : notifications.map(n => {
                    // 待處理分頁上每列都是未完成待辦；鈴鐺分頁上 kind='action' 者＝已完成的歷史
                    const isResolvedAction = !isTodo && n.kind === 'action'
                    const days = waitedDays(n.created_at)
                    return (
                    // 用 <button> 而非 <div onClick>：div 不可聚焦也不吃 Enter/Space，
                    // 鍵盤與螢幕閱讀器使用者打不開任何一則通知／待辦
                    <button
                        key={n.id}
                        type="button"
                        onClick={() => handleClick(n)}
                        className={cn(
                            'w-full text-left px-4 py-3 cursor-pointer hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset transition-colors flex items-start gap-3',
                            isTodo
                                ? 'border-l-4 border-l-status-error-solid'
                                : !n.is_read && 'bg-status-info-bg',
                        )}
                    >
                        {!isTodo && (
                            <div className={cn(
                                'w-2 h-2 rounded-full mt-2 shrink-0',
                                // 原本寫成 bg-status-info-bg0（不存在的 class），未讀圓點一直沒顯示
                                !n.is_read ? 'bg-status-info-solid' : 'bg-transparent',
                            )} />
                        )}
                        <div className="flex-1 min-w-0">
                            {isResolvedAction && (
                                <span className="inline-flex items-center gap-1 mb-1 text-xs font-medium px-1.5 py-0.5 rounded bg-status-success-bg text-status-success-text">
                                    <CheckCircle2 className="h-3 w-3" />
                                    已完成
                                </span>
                            )}
                            <p className={cn('text-sm', (isTodo || !n.is_read) && 'font-semibold')}>
                                {n.title}
                            </p>
                            {n.content && (
                                <p className="text-sm text-muted-foreground mt-0.5 line-clamp-2">
                                    {n.content}
                                </p>
                            )}
                            <div className="flex items-center gap-2 mt-1">
                                <span className="text-xs text-muted-foreground">
                                    {/* 待辦看的是「拖多久了」，通知看的是「什麼時候發生的」 */}
                                    {isTodo && days >= 1 ? `已等待 ${days} 天` : formatTime(n.created_at)}
                                </span>
                                <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                    {notificationTypeNames[n.type] ?? n.type}
                                </span>
                            </div>
                        </div>
                        {notificationTargetPath(n) && (
                            <ExternalLink className="h-4 w-4 text-muted-foreground shrink-0 mt-1" />
                        )}
                    </button>
                    )
                })}
            </div>

            {totalPages > 1 && (
                <div className="flex items-center justify-center gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        disabled={page <= 1}
                        onClick={() => setPage(p => p - 1)}
                    >
                        <ChevronLeft className="h-4 w-4" />
                    </Button>
                    <span className="text-sm text-muted-foreground">
                        第 {page} / {totalPages} 頁
                    </span>
                    <Button
                        variant="outline"
                        size="sm"
                        disabled={page >= totalPages}
                        onClick={() => setPage(p => p + 1)}
                    >
                        <ChevronRight className="h-4 w-4" />
                    </Button>
                </div>
            )}
        </div>
    )
}
