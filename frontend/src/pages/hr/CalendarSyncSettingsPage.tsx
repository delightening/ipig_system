/**
 * Google Calendar 同步設定頁面
 * 整合「日曆視圖」「同步歷史」「衝突處理」「連線狀態」四個分頁
 *
 * 重構後：邏輯由 useCalendarSync、useCalendarEvents Hook 管理，
 * 各分頁 UI 拆分為獨立子元件，此檔案僅負責佈局與組合。
 */
import {
    AlertTriangle,
    CalendarDays,
    CalendarRange,
    Clock,
    RefreshCw,
    Settings,
} from 'lucide-react'
import { useSearchParams } from 'react-router-dom'
import { useAuthIsAdmin } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { useCalendarSync } from '@/hooks/useCalendarSync'
import { useCalendarEvents } from '@/hooks/useCalendarEvents'
import { CalendarStatusTab } from './calendar/CalendarStatusTab'
import { CalendarEventsTab } from './calendar/CalendarEventsTab'
import { LeaveCalendarTab } from './calendar/LeaveCalendarTab'
import { SyncHistoryTab } from './calendar/SyncHistoryTab'
import { ConflictsTab } from './calendar/ConflictsTab'

/** 一般員工的落點分頁；沒有請假行事曆權限時退回 Google 事件檢視 */
const DEFAULT_TAB = 'leave-calendar'

/** 僅管理員可見的分頁；其餘身分即使手動改網址也不得進入 */
const ADMIN_ONLY_TABS = ['calendar', 'history', 'conflicts', 'status'] as const

export function CalendarSyncSettingsPage() {
    const [searchParams] = useSearchParams()
    const requestedTab = searchParams.get('tab') ?? DEFAULT_TAB

    // `PageTabs` 的 `hidden` 只是不畫出頁籤按鈕，**不會擋掉內容**——
    // 非管理員手打 `?tab=status` 仍會渲染該面板並觸發它的查詢。
    // 因此網址參數必須先對照授權範圍過濾，再往下傳給 useCalendarSync 與畫面。
    // isAdmin 直接讀 auth store（而非等 useCalendarSync 回傳），否則
    // 「要先有 activeTab 才能呼叫 hook、要先呼叫 hook 才知道 isAdmin」會繞不出來。
    const isAdminUser = useAuthIsAdmin()
    const activeTab =
        !isAdminUser && (ADMIN_ONLY_TABS as readonly string[]).includes(requestedTab)
            ? DEFAULT_TAB
            : requestedTab

    const {
        syncStatus,
        loadingStatus,
        syncHistory,
        loadingHistory,
        conflicts,
        loadingConflicts,
        isAdmin,
        canViewCalendarConfig,
        canViewCalendar,
        canViewLeaveCalendar,
        historyPage,
        setHistoryPage,
        conflictsPage,
        setConflictsPage,
        showConnectDialog,
        setShowConnectDialog,
        calendarId,
        setCalendarId,
        authEmail,
        setAuthEmail,
        connectMutation,
        disconnectMutation,
        syncMutation,
        resolveConflictMutation,
        calendarConfig,
        loadingConfig,
        updateConfigMutation,
    } = useCalendarSync(activeTab)

    // admin 依 syncStatus 判斷是否已連接；non-admin 無法取得 status，直接嘗試撈事件（backend 未設定時回傳空陣列）
    const isConfigured = canViewCalendarConfig
        ? (syncStatus?.is_configured === true)
        : canViewCalendar

    const {
        fullCalendarEvents,
        isLoading: loadingEvents,
        isFetching: fetchingEvents,
        handleDatesSet,
    } = useCalendarEvents(
        isConfigured,
        activeTab === 'calendar',
    )

    return (
        <div className="space-y-6">
            <PageHeader
                title="行事曆"
                description="請假與代理狀況；管理員另可管理 Google Calendar 同步"
                actions={isConfigured && canViewCalendarConfig ? (
                    <Button
                        size="sm"
                        onClick={() => syncMutation.mutate()}
                        disabled={syncMutation.isPending}
                    >
                        <RefreshCw className={`h-4 w-4 mr-2 ${syncMutation.isPending ? 'animate-spin' : ''}`} />
                        立即同步
                    </Button>
                ) : undefined}
            />

            {/* 同步相關分頁（Google 事件、同步歷史、衝突、連線）對一般員工是維運雜訊，
                一律收在 isAdmin 後面；員工只留「請假行事曆」。 */}
            <PageTabs
                tabs={[
                    { value: 'leave-calendar', label: '請假行事曆', icon: CalendarRange, hidden: !canViewLeaveCalendar },
                    { value: 'calendar', label: 'Google 日曆', icon: CalendarDays, hidden: !isAdmin },
                    { value: 'history', label: '同步歷史', icon: Clock, hidden: !isAdmin },
                    { value: 'conflicts', label: '衝突處理', icon: AlertTriangle, badge: conflicts?.total, hidden: !isAdmin },
                    { value: 'status', label: '連線狀態', icon: Settings, hidden: !isAdmin },
                ]}
                defaultTab={canViewLeaveCalendar ? DEFAULT_TAB : 'calendar'}
            >
                {/* 請假行事曆（原生資料） */}
                {canViewLeaveCalendar && (
                    <PageTabContent value="leave-calendar" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle>請假行事曆</CardTitle>
                                <CardDescription>
                                    誰哪幾天不在、由誰代理。含審核中的假單（虛線框）。
                                </CardDescription>
                            </CardHeader>
                            <CardContent>
                                <LeaveCalendarTab isActive={activeTab === 'leave-calendar'} />
                            </CardContent>
                        </Card>
                    </PageTabContent>
                )}

                {/* 連線狀態 */}
                <PageTabContent value="status" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>連線設定</CardTitle>
                            <CardDescription>
                                連接到共用的 Google Calendar 以同步請假事件
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <CalendarStatusTab
                                syncStatus={syncStatus}
                                loadingStatus={loadingStatus}
                                onDisconnect={() => disconnectMutation.mutate()}
                                disconnectPending={disconnectMutation.isPending}
                                onShowConnectDialog={() => setShowConnectDialog(true)}
                                calendarConfig={calendarConfig}
                                loadingConfig={loadingConfig}
                                onUpdateConfig={(data) => updateConfigMutation.mutate(data)}
                                updateConfigPending={updateConfigMutation.isPending}
                            />
                        </CardContent>
                    </Card>
                </PageTabContent>

                {/* 日曆視圖 */}
                <PageTabContent value="calendar" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Google Calendar 事件</CardTitle>
                            <CardDescription>
                                從已連接的 Google Calendar 讀取的事件
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <CalendarEventsTab
                                isConfigured={isConfigured}
                                isAdmin={isAdmin}
                                loadingStatus={loadingStatus}
                                isLoading={loadingEvents}
                                isFetching={fetchingEvents}
                                fullCalendarEvents={fullCalendarEvents}
                                onDatesSet={handleDatesSet}
                            />
                        </CardContent>
                    </Card>
                </PageTabContent>

                {/* 同步歷史 */}
                <PageTabContent value="history" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>同步歷史記錄</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <SyncHistoryTab
                                syncHistory={syncHistory}
                                loadingHistory={loadingHistory}
                                currentPage={historyPage}
                                onPageChange={setHistoryPage}
                            />
                        </CardContent>
                    </Card>
                </PageTabContent>

                {/* 衝突處理 */}
                <PageTabContent value="conflicts" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>待處理衝突</CardTitle>
                            <CardDescription>
                                系統與 Google Calendar 之間的資料不一致需要手動處理
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <ConflictsTab
                                conflicts={conflicts}
                                loadingConflicts={loadingConflicts}
                                onResolve={(params) => resolveConflictMutation.mutate(params)}
                                resolvePending={resolveConflictMutation.isPending}
                                currentPage={conflictsPage}
                                onPageChange={setConflictsPage}
                            />
                        </CardContent>
                    </Card>
                </PageTabContent>
            </PageTabs>

            {/* 連接對話框 */}
            <Dialog open={showConnectDialog} onOpenChange={setShowConnectDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>連接 Google Calendar</DialogTitle>
                        <DialogDescription>
                            輸入共用日曆的 ID 和授權帳戶 Email
                        </DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label>Calendar ID *</Label>
                            <Input
                                placeholder="例如: company-leave@group.calendar.google.com"
                                value={calendarId}
                                onChange={(e) => setCalendarId(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                在 Google Calendar 設定中找到日曆 ID
                            </p>
                        </div>
                        <div className="grid gap-2">
                            <Label>授權 Email *</Label>
                            <Input
                                type="email"
                                placeholder="service-account@example.com"
                                value={authEmail}
                                onChange={(e) => setAuthEmail(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                擁有日曆編輯權限的 Google 帳戶
                            </p>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowConnectDialog(false)}>
                            取消
                        </Button>
                        <Button
                            onClick={() =>
                                connectMutation.mutate({
                                    calendar_id: calendarId,
                                    auth_email: authEmail,
                                })
                            }
                            disabled={!calendarId || !authEmail || connectMutation.isPending}
                        >
                            連接
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}

export default CalendarSyncSettingsPage
