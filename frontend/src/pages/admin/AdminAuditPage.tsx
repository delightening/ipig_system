import { useState } from 'react'
import { Activity, AlertTriangle, LogIn, RefreshCw, Shield, Users } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
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
            <PageHeader
                title="安全審計"
                description="監控系統活動與安全事件"
                actions={
                    <Button variant="outline" onClick={audit.refreshAll}>
                        <RefreshCw className="h-4 w-4 mr-2" />
                        重新整理
                    </Button>
                }
            />

            <PageTabs
                tabs={[
                    { value: 'dashboard', label: '總覽', icon: Shield },
                    { value: 'activities', label: '活動記錄', icon: Activity },
                    { value: 'logins', label: '登入事件', icon: LogIn },
                    { value: 'sessions', label: '活躍 Sessions', icon: Users },
                    { value: 'alerts', label: '安全警報', icon: AlertTriangle, badge: audit.dashboardStats?.open_alerts },
                ]}
                defaultTab="dashboard"
            >
                <PageTabContent value="dashboard" className="space-y-4">
                    <AuditDashboardTab stats={audit.dashboardStats} />
                </PageTabContent>

                <PageTabContent value="activities" className="space-y-4">
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
                </PageTabContent>

                <PageTabContent value="logins" className="space-y-4">
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
                </PageTabContent>

                <PageTabContent value="sessions" className="space-y-4">
                    <AuditSessionsTab
                        sessions={audit.sessions}
                        isLoading={audit.loadingSessions}
                        currentPage={audit.sessionsPage}
                        onPageChange={audit.setSessionsPage}
                        forceLogoutMutation={audit.forceLogoutMutation}
                    />
                </PageTabContent>

                <PageTabContent value="alerts" className="space-y-4">
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
                        search={audit.alertSearch}
                        onSearchChange={audit.handleAlertSearchChange}
                        statusFilter={audit.alertStatusFilter}
                        onStatusFilterChange={audit.handleAlertStatusFilterChange}
                    />
                </PageTabContent>
            </PageTabs>

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
