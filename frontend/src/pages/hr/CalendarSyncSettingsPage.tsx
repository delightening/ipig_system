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
import { useTranslation } from 'react-i18next'
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
    const { t } = useTranslation()
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
                title={t('nav.hrCalendar')}
                description={t('hrPages.calendar.page.description')}
                actions={isConfigured && canViewCalendarConfig ? (
                    <Button
                        size="sm"
                        onClick={() => syncMutation.mutate()}
                        disabled={syncMutation.isPending}
                    >
                        <RefreshCw className={`h-4 w-4 mr-2 ${syncMutation.isPending ? 'animate-spin' : ''}`} />
                        {t('hrPages.calendar.page.syncNow')}
                    </Button>
                ) : undefined}
            />

            {/* 同步相關分頁（Google 事件、同步歷史、衝突、連線）對一般員工是維運雜訊，
                一律收在 isAdmin 後面；員工只留「請假行事曆」。 */}
            <PageTabs
                tabs={[
                    { value: 'leave-calendar', label: t('hrPages.calendar.tabs.leaveCalendar'), icon: CalendarRange, hidden: !canViewLeaveCalendar },
                    { value: 'calendar', label: t('hrPages.calendar.tabs.google'), icon: CalendarDays, hidden: !isAdmin },
                    { value: 'history', label: t('hrPages.calendar.tabs.history'), icon: Clock, hidden: !isAdmin },
                    { value: 'conflicts', label: t('hrPages.calendar.tabs.conflicts'), icon: AlertTriangle, badge: conflicts?.total, hidden: !isAdmin },
                    { value: 'status', label: t('hrPages.calendar.tabs.status'), icon: Settings, hidden: !isAdmin },
                ]}
                defaultTab={canViewLeaveCalendar ? DEFAULT_TAB : 'calendar'}
            >
                {/* 請假行事曆（原生資料） */}
                {canViewLeaveCalendar && (
                    <PageTabContent value="leave-calendar" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle>{t('hrPages.calendar.tabs.leaveCalendar')}</CardTitle>
                                <CardDescription>
                                    {t('hrPages.calendar.leaveCalendarDescription')}
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
                            <CardTitle>{t('hrPages.calendar.connection.title')}</CardTitle>
                            <CardDescription>
                                {t('hrPages.calendar.connection.description')}
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
                            <CardTitle>{t('hrPages.calendar.events.title')}</CardTitle>
                            <CardDescription>
                                {t('hrPages.calendar.events.description')}
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
                            <CardTitle>{t('hrPages.calendar.historyTitle')}</CardTitle>
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
                            <CardTitle>{t('hrPages.calendar.pendingConflicts')}</CardTitle>
                            <CardDescription>
                                {t('hrPages.calendar.conflictsDescription')}
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
                        <DialogTitle>{t('hrPages.calendar.connect.title')}</DialogTitle>
                        <DialogDescription>
                            {t('hrPages.calendar.connect.description')}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label>Calendar ID *</Label>
                            <Input
                                placeholder={t('hrPages.calendar.connect.calendarIdPlaceholder')}
                                value={calendarId}
                                onChange={(e) => setCalendarId(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                {t('hrPages.calendar.connect.calendarIdHint')}
                            </p>
                        </div>
                        <div className="grid gap-2">
                            <Label>{t('hrPages.calendar.connect.authEmail')} *</Label>
                            <Input
                                type="email"
                                placeholder="service-account@example.com"
                                value={authEmail}
                                onChange={(e) => setAuthEmail(e.target.value)}
                            />
                            <p className="text-xs text-muted-foreground">
                                {t('hrPages.calendar.connect.authEmailHint')}
                            </p>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowConnectDialog(false)}>
                            {t('common.cancel')}
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
                            {t('hrPages.calendar.connect.submit')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}

export default CalendarSyncSettingsPage
