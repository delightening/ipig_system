import { Calendar, Clock } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { parseDecimal } from '@/lib/utils'
import type { BalanceSummary } from '@/types/hr'

interface LeaveBalanceSummaryProps {
    balanceSummary: BalanceSummary | undefined
}

export function LeaveBalanceSummary({ balanceSummary }: LeaveBalanceSummaryProps) {
    return (
        <div className="grid gap-4 md:grid-cols-4">
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">特休剩餘</CardTitle>
                    <Calendar className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">
                        {balanceSummary?.annual_leave_remaining ?? 0} 天
                    </div>
                    <p className="text-xs text-muted-foreground">
                        已使用 {balanceSummary?.annual_leave_used ?? 0} /{' '}
                        {balanceSummary?.annual_leave_total ?? 0} 天
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">補休剩餘</CardTitle>
                    <Clock className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">
                        {parseDecimal(balanceSummary?.comp_time_remaining).toFixed(1)} 小時
                    </div>
                    <p className="text-xs text-muted-foreground">
                        已使用 {parseDecimal(balanceSummary?.comp_time_used).toFixed(1)} 小時
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">即將到期（特休）</CardTitle>
                    <Clock className="h-4 w-4 text-status-warning-text" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold text-status-warning-text">
                        {balanceSummary?.expiring_soon_days ?? 0} 天
                    </div>
                    <p className="text-xs text-muted-foreground">30 天內到期</p>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between pb-2">
                    <CardTitle className="text-sm font-medium">即將到期（補休）</CardTitle>
                    <Clock className="h-4 w-4 text-status-warning-text" />
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold text-status-warning-text">
                        {parseDecimal(balanceSummary?.expiring_soon_hours).toFixed(1)} 小時
                    </div>
                    <p className="text-xs text-muted-foreground">30 天內到期</p>
                </CardContent>
            </Card>
        </div>
    )
}
