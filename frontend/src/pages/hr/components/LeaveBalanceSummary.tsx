import { Calendar, Clock } from 'lucide-react'

import { StatsCard } from '@/components/ui/stats-card'
import { parseDecimal } from '@/lib/utils'
import type { BalanceSummary } from '@/types/hr'

interface LeaveBalanceSummaryProps {
    balanceSummary: BalanceSummary | undefined
}

export function LeaveBalanceSummary({ balanceSummary }: LeaveBalanceSummaryProps) {
    return (
        <div className="grid gap-4 md:grid-cols-4">
            <StatsCard
                icon={Calendar}
                label="特休剩餘"
                value={`${balanceSummary?.annual_leave_remaining ?? 0} 天`}
                description={`已使用 ${balanceSummary?.annual_leave_used ?? 0} / ${balanceSummary?.annual_leave_total ?? 0} 天`}
            />
            <StatsCard
                icon={Clock}
                label="補休剩餘"
                value={`${parseDecimal(balanceSummary?.comp_time_remaining).toFixed(1)} 小時`}
                description={`已使用 ${parseDecimal(balanceSummary?.comp_time_used).toFixed(1)} 小時`}
            />
            <StatsCard
                icon={Clock}
                label="即將到期（特休）"
                value={`${balanceSummary?.expiring_soon_days ?? 0} 天`}
                description="30 天內到期"
                iconClassName="text-status-warning-text"
                valueClassName="text-status-warning-text"
            />
            <StatsCard
                icon={Clock}
                label="即將到期（補休）"
                value={`${parseDecimal(balanceSummary?.expiring_soon_hours).toFixed(1)} 小時`}
                description="30 天內到期"
                iconClassName="text-status-warning-text"
                valueClassName="text-status-warning-text"
            />
        </div>
    )
}
