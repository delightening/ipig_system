import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Activity, AlertTriangle, Ban, LogIn, RefreshCw, Shield, ShieldAlert, Users } from 'lucide-react'

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
import { IpBlocklistTab } from './components/IpBlocklistTab'
import { SecurityEventsTab } from './components/SecurityEventsTab'
import { AuditLogDetailDialog } from './components/AuditLogDetailDialog'
import { AuditAlertDetailDialog } from './components/AuditAlertDetailDialog'

export function AdminAuditPage() {
    const { t } = useTranslation()
    const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null)
    const [selectedAlert, setSelectedAlert] = useState<SecurityAlert | null>(null)

    const audit = useAuditData()

    return (
        <div className="space-y-6">
            <PageHeader
                title={t('nav.adminSecurityAudit')}
                description={t('adminUsers.audit.page.description')}
                actions={
                    <Button size="sm" variant="outline" onClick={audit.refreshAll}>
                        <RefreshCw className="h-4 w-4 mr-2" />
                        {t('adminUsers.audit.page.refresh')}
                    </Button>
                }
            />

            <PageTabs
                tabs={[
                    { value: 'dashboard', label: t('adminUsers.audit.tab.overview'), icon: Shield },
                    { value: 'activities', label: t('adminUsers.audit.tab.activities'), icon: Activity },
                    { value: 'logins', label: t('adminUsers.audit.tab.logins'), icon: LogIn },
                    { value: 'sessions', label: t('adminUsers.audit.tab.sessions'), icon: Users },
                    { value: 'alerts', label: t('adminUsers.audit.tab.alerts'), icon: AlertTriangle, badge: audit.dashboardStats?.open_alerts },
                    { value: 'security-events', label: t('adminUsers.audit.tab.securityEvents'), icon: ShieldAlert },
                    { value: 'ip-blocklist', label: t('adminUsers.audit.tab.ipBlocklist'), icon: Ban },
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
                        eventTypeFilter={audit.loginEventType}
                        onEventTypeChange={audit.handleLoginEventTypeChange}
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
                        bulkResolveAlertsMutation={audit.bulkResolveAlertsMutation}
                        selectedAlertIds={audit.selectedAlertIds}
                        onAlertSelect={audit.handleAlertSelect}
                        onSelectAllAlerts={audit.handleSelectAllAlerts}
                        search={audit.alertSearch}
                        onSearchChange={audit.handleAlertSearchChange}
                        statusFilter={audit.alertStatusFilter}
                        onStatusFilterChange={audit.handleAlertStatusFilterChange}
                    />
                </PageTabContent>

                <PageTabContent value="security-events" className="space-y-4">
                    <SecurityEventsTab
                        dateFrom={audit.dateFrom}
                        dateTo={audit.dateTo}
                        onDateFromChange={audit.handleDateFromChange}
                        onDateToChange={audit.handleDateToChange}
                        securityEvents={audit.securityEvents}
                        isLoading={audit.loadingSecurityEvents}
                        currentPage={audit.securityEventsPage}
                        onPageChange={audit.setSecurityEventsPage}
                        eventTypeFilter={audit.securityEventType}
                        onEventTypeChange={audit.handleSecurityEventTypeChange}
                    />
                </PageTabContent>

                <PageTabContent value="ip-blocklist" className="space-y-4">
                    <IpBlocklistTab />
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
