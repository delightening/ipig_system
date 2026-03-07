/**
 * Google Calendar 同步設定頁面
 * 整合「日曆視圖」「同步歷史」「衝突處理」「連線狀態」四個分頁
 *
 * 重構後：邏輯由 useCalendarSync、useCalendarEvents Hook 管理，
 * 各分頁 UI 拆分為獨立子元件，此檔案僅負責佈局與組合。
 */
import { useState } from 'react'
import {
    AlertTriangle,
    CalendarDays,
    Clock,
    RefreshCw,
    Settings,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
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
import { SyncHistoryTab } from './calendar/SyncHistoryTab'
import { ConflictsTab } from './calendar/ConflictsTab'

export function CalendarSyncSettingsPage() {
    const [activeTab, setActiveTab] = useState('calendar')

    const {
        syncStatus,
        loadingStatus,
        syncHistory,
        loadingHistory,
        conflicts,
        loadingConflicts,
        isAdmin,
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

    const {
        fullCalendarEvents,
        isLoading: loadingEvents,
        isFetching: fetchingEvents,
        handleDatesSet,
    } = useCalendarEvents(
        syncStatus?.is_configured === true,
        activeTab === 'calendar',
    )

    return (
        <div className="space-y-6">
            {/* 頁面標題與同步按鈕 */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">Google Calendar</h1>
                    <p className="text-muted-foreground">設定與管理請假日曆</p>
                </div>
                {syncStatus?.is_configured && (
                    <Button
                        onClick={() => syncMutation.mutate()}
                        disabled={syncMutation.isPending}
                    >
                        <RefreshCw className={`h-4 w-4 mr-2 ${syncMutation.isPending ? 'animate-spin' : ''}`} />
                        立即同步
                    </Button>
                )}
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="calendar" className="flex items-center gap-2">
                        <CalendarDays className="h-4 w-4" />
                        日曆
                    </TabsTrigger>
                    <TabsTrigger value="history" className="flex items-center gap-2">
                        <Clock className="h-4 w-4" />
                        同步歷史
                    </TabsTrigger>
                    {isAdmin && (
                        <TabsTrigger value="conflicts" className="flex items-center gap-2">
                            <AlertTriangle className="h-4 w-4" />
                            衝突處理
                            {conflicts && conflicts.total > 0 && (
                                <Badge variant="destructive" className="ml-1">
                                    {conflicts.total}
                                </Badge>
                            )}
                        </TabsTrigger>
                    )}
                    {isAdmin && (
                        <TabsTrigger value="status" className="flex items-center gap-2">
                            <Settings className="h-4 w-4" />
                            連線狀態
                        </TabsTrigger>
                    )}
                </TabsList>

                {/* 連線狀態 */}
                <TabsContent value="status" className="space-y-4">
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
                </TabsContent>

                {/* 日曆視圖 */}
                <TabsContent value="calendar" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Google Calendar 事件</CardTitle>
                            <CardDescription>
                                從已連接的 Google Calendar 讀取的事件
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <CalendarEventsTab
                                isConfigured={syncStatus?.is_configured === true}
                                isAdmin={isAdmin}
                                loadingStatus={loadingStatus}
                                isLoading={loadingEvents}
                                isFetching={fetchingEvents}
                                fullCalendarEvents={fullCalendarEvents}
                                onDatesSet={handleDatesSet}
                            />
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* 同步歷史 */}
                <TabsContent value="history" className="space-y-4">
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
                </TabsContent>

                {/* 衝突處理 */}
                <TabsContent value="conflicts" className="space-y-4">
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
                </TabsContent>
            </Tabs>

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
