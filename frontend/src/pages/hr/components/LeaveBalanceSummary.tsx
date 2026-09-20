import { useTranslation } from 'react-i18next'
import { Calendar, Clock } from 'lucide-react'

import { StatsCard } from '@/components/ui/stats-card'
import { parseDecimal } from '@/lib/utils'
import type { BalanceSummary } from '@/types/hr'

interface LeaveBalanceSummaryProps {
    balanceSummary: BalanceSummary | undefined
}

export function LeaveBalanceSummary({ balanceSummary }: LeaveBalanceSummaryProps) {
    const { t } = useTranslation()
    return (
        <div className="grid gap-4 md:grid-cols-4">
            <StatsCard
                icon={Calendar}
                label={t('hrPages.leaves.balance.annualRemaining')}
                value={t('hrPages.shared.daysValue', { days: balanceSummary?.annual_leave_remaining ?? 0 })}
                description={t('hrPages.leaves.balance.annualUsed', {
                    used: balanceSummary?.annual_leave_used ?? 0,
                    total: balanceSummary?.annual_leave_total ?? 0,
                })}
            />
            <StatsCard
                icon={Clock}
                label={t('hrPages.leaves.balance.compRemaining')}
                value={t('hrPages.shared.hoursValue', { hours: parseDecimal(balanceSummary?.comp_time_remaining).toFixed(1) })}
                description={t('hrPages.leaves.balance.compUsed', { hours: parseDecimal(balanceSummary?.comp_time_used).toFixed(1) })}
            />
            <StatsCard
                icon={Clock}
                label={t('hrPages.leaves.balance.expiringAnnual')}
                value={t('hrPages.shared.daysValue', { days: balanceSummary?.expiring_soon_days ?? 0 })}
                description={t('hrPages.leaves.balance.within30Days')}
                iconClassName="text-status-warning-text"
                valueClassName="text-status-warning-text"
            />
            <StatsCard
                icon={Clock}
                label={t('hrPages.leaves.balance.expiringComp')}
                value={t('hrPages.shared.hoursValue', { hours: parseDecimal(balanceSummary?.expiring_soon_hours).toFixed(1) })}
                description={t('hrPages.leaves.balance.within30Days')}
                iconClassName="text-status-warning-text"
                valueClassName="text-status-warning-text"
            />
        </div>
    )
}
