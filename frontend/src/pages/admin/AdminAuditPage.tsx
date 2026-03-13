import { useState } from 'react'
import { Activity, AlertTriangle, LogIn, RefreshCw, Shield, Users } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { SecurityAlert } from '@/types/hr'
import { useAuditData } from './hooks/useAuditData'
import type { AuditLog } from './types/audit'
import { AuditDashboardTab } from './components/AuditDashboardTab'
import { AuditActivitiesTab } from './components/AuditActivitiesTab'
import { AuditLoginsTab } from './components/AuditLoginsTab'
import { AuditSessionsTab } from './components/AuditSessionsTab'
import { AuditAlertsTab } from './components/AuditAlertsTab'
import { AuditLogDetailDialog } from './components/AuditLogDetailDialog'
import { AuditAlertDetailDialog } from './components/AuditAlertDetailDialog'

export function AdminAuditPage() {
    const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null)
    const [selectedAlert, setSelectedAlert] = useState<SecurityAlert | null>(null)

    const audit = useAuditData()

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">安全審計</h1>
                    <p className="text-muted-foreground">監控系統活動與安全事件</p>
                </div>
                <Button variant="outline" onClick={audit.refreshAll}>
                    <RefreshCw className="h-4 w-4 mr-2" />
                    重新整理
                </Button>
            </div>

            <Tabs value={audit.activeTab} onValueChange={audit.setActiveTab}>
                <TabsList className="flex flex-wrap w-full sm:grid sm:grid-cols-5">
                    <TabsTrigger value="dashboard" className="flex items-center gap-1.5 text-xs sm:text-sm sm:gap-2">
                        <Shield className="h-4 w-4 hidden sm:inline-block" />
                        總覽
                    </TabsTrigger>
                    <TabsTrigger value="activities" className="flex items-center gap-1.5 text-xs sm:text-sm sm:gap-2">
                        <Activity className="h-4 w-4 hidden sm:inline-block" />
                        活動記錄
                    </TabsTrigger>
                    <TabsTrigger value="logins" className="flex items-center gap-1.5 text-xs sm:text-sm sm:gap-2">
                        <LogIn className="h-4 w-4 hidden sm:inline-block" />
                        登入事件
                    </TabsTrigger>
                    <TabsTrigger value="sessions" className="flex items-center gap-1.5 text-xs sm:text-sm sm:gap-2">
                        <Users className="h-4 w-4 hidden sm:inline-block" />
                        活躍 Sessions
                    </TabsTrigger>
                    <TabsTrigger value="alerts" className="flex items-center gap-1.5 text-xs sm:text-sm sm:gap-2">
                        <AlertTriangle className="h-4 w-4 hidden sm:inline-block" />
                        安全警報
                        {audit.dashboardStats && audit.dashboardStats.open_alerts > 0 && (
                            <Badge variant="destructive" className="ml-0.5 sm:ml-1 text-[10px] sm:text-xs px-1 sm:px-1.5">
                                {audit.dashboardStats.open_alerts}
                            </Badge>
                        )}
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="dashboard" className="space-y-4">
                    <AuditDashboardTab stats={audit.dashboardStats} />
                </TabsContent>

                <TabsContent value="activities" className="space-y-4">
                    <AuditActivitiesTab
                        dateFrom={audit.dateFrom}
                        dateTo={audit.dateTo}
                        onDateFromChange={audit.handleDateFromChange}
                        onDateToChange={audit.handleDateToChange}
                        activityLogs={audit.activityLogs}
                        isLoading={audit.loadingActivities}
                        currentPage={audit.activitiesPage}
                        onPageChange={audit.setActivitiesPage}
                        onSelectLog={setSelectedLog}
                    />
                </TabsContent>

                <TabsContent value="logins" className="space-y-4">
                    <AuditLoginsTab
                        dateFrom={audit.dateFrom}
                        dateTo={audit.dateTo}
                        onDateFromChange={audit.handleDateFromChange}
                        onDateToChange={audit.handleDateToChange}
                        loginEvents={audit.loginEvents}
                        isLoading={audit.loadingLogins}
                        currentPage={audit.loginsPage}
                        onPageChange={audit.setLoginsPage}
                    />
                </TabsContent>

                <TabsContent value="sessions" className="space-y-4">
                    <AuditSessionsTab
                        sessions={audit.sessions}
                        isLoading={audit.loadingSessions}
                        currentPage={audit.sessionsPage}
                        onPageChange={audit.setSessionsPage}
                        forceLogoutMutation={audit.forceLogoutMutation}
                    />
                </TabsContent>

                <TabsContent value="alerts" className="space-y-4">
                    <AuditAlertsTab
                        alerts={audit.alerts}
                        sortedAlerts={audit.sortedAlerts}
                        isLoading={audit.loadingAlerts}
                        currentPage={audit.alertsPage}
                        onPageChange={audit.setAlertsPage}
                        sortConfig={audit.alertSortConfig}
                        onSort={audit.handleAlertSort}
                        onSelectAlert={setSelectedAlert}
                        resolveAlertMutation={audit.resolveAlertMutation}
                    />
                </TabsContent>
            </Tabs>

            <AuditLogDetailDialog
                log={selectedLog}
                open={!!selectedLog}
                onOpenChange={(open) => !open && setSelectedLog(null)}
            />
            <AuditAlertDetailDialog
                alert={selectedAlert}
                open={!!selectedAlert}
                onOpenChange={(open) => !open && setSelectedAlert(null)}
                onResolve={(alertId) => audit.resolveAlertMutation.mutate(alertId)}
                isResolving={audit.resolveAlertMutation.isPending}
            />
        </div>
    )
}

export default AdminAuditPage
