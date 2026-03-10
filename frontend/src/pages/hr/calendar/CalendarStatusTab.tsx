/**
 * 日曆「連線狀態」分頁元件
 * 顯示 Google Calendar 連接狀態、同步統計、自動同步設定
 */
import { useState, useEffect } from 'react'
import {
    AlertTriangle,
    Calendar,
    CheckCircle,
    Clock,
    Link2,
    Save,
    Unlink,
    XCircle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import type { CalendarSyncStatus, CalendarConfig, UpdateCalendarConfig } from '@/types/hr'
import { formatDateTime } from '@/lib/utils'

interface CalendarStatusTabProps {
    syncStatus: CalendarSyncStatus | undefined
    loadingStatus: boolean
    onDisconnect: () => void
    disconnectPending: boolean
    onShowConnectDialog: () => void
    calendarConfig: CalendarConfig | undefined
    loadingConfig: boolean
    onUpdateConfig: (data: UpdateCalendarConfig) => void
    updateConfigPending: boolean
}

/** 最後同步狀態標籤 */
function getLastSyncBadge(status: string | null) {
    switch (status) {
        case 'success':
            return <Badge className="bg-green-500">成功</Badge>
        case 'partial':
            return <Badge className="bg-yellow-500">部分完成</Badge>
        case 'failed':
            return <Badge variant="destructive">失敗</Badge>
        default:
            return null
    }
}

/** 將 "HH:MM:SS" 轉為 <input type="time"> 的值 "HH:MM" */
function toTimeInput(val: string | null): string {
    if (!val) return ''
    return val.slice(0, 5) // "HH:MM"
}

/** 將 "HH:MM" 轉為後端格式 "HH:MM:SS"，空字串轉為 null */
function fromTimeInput(val: string): string | null {
    return val ? `${val}:00` : null
}

export function CalendarStatusTab({
    syncStatus,
    loadingStatus,
    onDisconnect,
    disconnectPending,
    onShowConnectDialog,
    calendarConfig,
    loadingConfig,
    onUpdateConfig,
    updateConfigPending,
}: CalendarStatusTabProps) {
    // 本地設定表單狀態
    const [syncEnabled, setSyncEnabled] = useState(false)
    const [morningTime, setMorningTime] = useState('')
    const [eveningTime, setEveningTime] = useState('')
    const [isDirty, setIsDirty] = useState(false)

    // 載入遠端設定後同步至本地狀態
    useEffect(() => {
        if (!calendarConfig) return
        setSyncEnabled(calendarConfig.sync_enabled)
        setMorningTime(toTimeInput(calendarConfig.sync_schedule_morning))
        setEveningTime(toTimeInput(calendarConfig.sync_schedule_evening))
        setIsDirty(false)
    }, [calendarConfig])

    const handleSave = () => {
        onUpdateConfig({
            sync_enabled: syncEnabled,
            sync_schedule_morning: fromTimeInput(morningTime),
            sync_schedule_evening: fromTimeInput(eveningTime),
        })
        setIsDirty(false)
    }

    if (loadingStatus) {
        return <div className="text-center py-8">載入中...</div>
    }

    if (syncStatus?.is_configured) {
        return (
            <div className="space-y-6">
                {/* 連線標頭 */}
                <div className="flex items-center gap-4">
                    <div className="h-12 w-12 rounded-full bg-green-100 flex items-center justify-center flex-shrink-0">
                        <CheckCircle className="h-6 w-6 text-green-600" />
                    </div>
                    <div className="flex-1 min-w-0">
                        <div className="font-medium">已連接</div>
                        <div className="text-sm text-muted-foreground truncate">
                            {syncStatus.calendar_id}
                        </div>
                    </div>
                    <Button
                        variant="destructive"
                        onClick={onDisconnect}
                        disabled={disconnectPending}
                        className="flex-shrink-0"
                    >
                        <Unlink className="h-4 w-4 mr-2" />
                        斷開連接
                    </Button>
                </div>

                {/* 近期錯誤警告 */}
                {syncStatus.recent_errors > 0 && (
                    <div className="flex items-center gap-3 rounded-lg border border-destructive/30 bg-destructive/10 p-3">
                        <AlertTriangle className="h-4 w-4 text-destructive flex-shrink-0" />
                        <div className="text-sm text-destructive">
                            過去 24 小時內有 {syncStatus.recent_errors} 個同步錯誤，請至「同步歷史」查看詳情
                        </div>
                    </div>
                )}

                {/* 統計卡片 */}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <Card>
                        <CardHeader className="pb-2">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <CheckCircle className="h-4 w-4 text-muted-foreground" />
                                同步狀態
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            {syncStatus.sync_enabled ? (
                                <Badge className="bg-green-500">自動同步啟用</Badge>
                            ) : (
                                <Badge variant="secondary">自動同步停用</Badge>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="pb-2">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <Clock className="h-4 w-4 text-muted-foreground" />
                                最後同步
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-1">
                            <div className="text-sm">
                                {syncStatus.last_sync_at
                                    ? formatDateTime(syncStatus.last_sync_at)
                                    : '尚未同步'}
                            </div>
                            {syncStatus.last_sync_status && (
                                <div>{getLastSyncBadge(syncStatus.last_sync_status)}</div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="pb-2">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <Clock className="h-4 w-4 text-muted-foreground" />
                                待同步事件
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{syncStatus.pending_syncs}</div>
                            {syncStatus.next_sync_at && (
                                <div className="text-xs text-muted-foreground mt-1">
                                    下次同步：{formatDateTime(syncStatus.next_sync_at)}
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="pb-2">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <XCircle className="h-4 w-4 text-muted-foreground" />
                                待處理衝突
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className={`text-2xl font-bold ${syncStatus.pending_conflicts > 0 ? 'text-orange-500' : ''}`}>
                                {syncStatus.pending_conflicts}
                            </div>
                            {syncStatus.pending_conflicts > 0 && (
                                <div className="text-xs text-orange-500 mt-1">需要手動處理</div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                {/* 自動同步設定 */}
                <div className="border rounded-lg p-4 space-y-4">
                    <div className="font-medium text-sm">自動同步設定</div>

                    {loadingConfig ? (
                        <div className="text-sm text-muted-foreground">載入設定中...</div>
                    ) : (
                        <>
                            {/* 啟用自動同步 */}
                            <div className="flex items-center justify-between">
                                <div>
                                    <Label htmlFor="sync-enabled" className="text-sm font-medium">
                                        啟用自動同步
                                    </Label>
                                    <div className="text-xs text-muted-foreground mt-0.5">
                                        依排程自動將請假事件同步至 Google Calendar
                                    </div>
                                </div>
                                <Switch
                                    id="sync-enabled"
                                    checked={syncEnabled}
                                    onCheckedChange={(v) => { setSyncEnabled(v); setIsDirty(true) }}
                                />
                            </div>

                            {/* 同步時間設定 */}
                            {syncEnabled && (
                                <div className="grid gap-4 sm:grid-cols-2 pt-2 border-t">
                                    <div className="space-y-1.5">
                                        <Label htmlFor="morning-time" className="text-sm">
                                            早上同步時間
                                        </Label>
                                        <input
                                            id="morning-time"
                                            type="time"
                                            value={morningTime}
                                            onChange={(e) => { setMorningTime(e.target.value); setIsDirty(true) }}
                                            className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                        />
                                        <div className="text-xs text-muted-foreground">留空表示不設定早上同步</div>
                                    </div>
                                    <div className="space-y-1.5">
                                        <Label htmlFor="evening-time" className="text-sm">
                                            晚上同步時間
                                        </Label>
                                        <input
                                            id="evening-time"
                                            type="time"
                                            value={eveningTime}
                                            onChange={(e) => { setEveningTime(e.target.value); setIsDirty(true) }}
                                            className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                        />
                                        <div className="text-xs text-muted-foreground">留空表示不設定晚上同步</div>
                                    </div>
                                </div>
                            )}

                            {/* 儲存按鈕 */}
                            <div className="flex justify-end pt-2">
                                <Button
                                    size="sm"
                                    onClick={handleSave}
                                    disabled={!isDirty || updateConfigPending}
                                >
                                    <Save className="h-4 w-4 mr-2" />
                                    {updateConfigPending ? '儲存中...' : '儲存設定'}
                                </Button>
                            </div>
                        </>
                    )}
                </div>
            </div>
        )
    }

    return (
        <div className="text-center py-8 space-y-4">
            <Calendar className="h-16 w-16 mx-auto text-muted-foreground" />
            <div>
                <div className="font-medium">尚未連接 Google Calendar</div>
                <div className="text-sm text-muted-foreground">
                    連接後可自動同步核准的請假事件到共用日曆
                </div>
            </div>
            <Button onClick={onShowConnectDialog}>
                <Link2 className="h-4 w-4 mr-2" />
                連接 Google Calendar
            </Button>
        </div>
    )
}
