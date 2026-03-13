import { AlertTriangle, Clock, LogIn, Users } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { AuditDashboardStats } from '@/types/hr'

interface AuditDashboardTabProps {
    stats: AuditDashboardStats | undefined
}

export function AuditDashboardTab({ stats }: AuditDashboardTabProps) {
    return (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">今日活躍用戶</CardTitle>
                    <Users className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">{stats?.active_users_today ?? 0}</div>
                    <p className="text-xs text-muted-foreground">
                        本週: {stats?.active_users_week ?? 0} / 本月: {stats?.active_users_month ?? 0}
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">今日登入次數</CardTitle>
                    <LogIn className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">{stats?.total_logins_today ?? 0}</div>
                    <p className="text-xs text-muted-foreground">
                        失敗: {stats?.failed_logins_today ?? 0} 次
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">活躍 Sessions</CardTitle>
                    <Clock className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">{stats?.active_sessions ?? 0}</div>
                    <p className="text-xs text-muted-foreground">目前線上</p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">未解決警報</CardTitle>
                    <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">{stats?.open_alerts ?? 0}</div>
                    <p className="text-xs text-muted-foreground">
                        嚴重: {stats?.critical_alerts ?? 0}
                    </p>
                </CardContent>
            </Card>
        </div>
    )
}
