import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Calendar, Info, Loader2 } from 'lucide-react'
import api from '@/lib/api'

interface BalanceSummary {
    annual_leave_total: number
    annual_leave_used: number
    annual_leave_remaining: number
    comp_time_total: number
    comp_time_used: number
    comp_time_remaining: number
}

interface LeaveBalance {
    leave_type: 'annual' | 'comp'
    total: number
    used: number
    remaining: number
    unit: 'day' | 'hour'
}

export function LeaveBalanceWidget() {
    const { t } = useTranslation()
    const { data, isLoading, error } = useQuery({
        queryKey: ['my-leave-balances'],
        queryFn: async () => {
            const res = await api.get<BalanceSummary>('/hr/balances/summary')
            const summary = res.data

            const balances: LeaveBalance[] = [
                {
                    leave_type: 'annual',
                    total: summary.annual_leave_total,
                    used: summary.annual_leave_used,
                    remaining: summary.annual_leave_remaining,
                    unit: 'day'
                },
                {
                    leave_type: 'comp',
                    total: summary.comp_time_total,
                    used: summary.comp_time_used,
                    remaining: summary.comp_time_remaining,
                    unit: 'hour'
                }
            ]
            return balances
        },
    })

    if (isLoading) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-orange-500" />
                        {t('dashboard.widgets.names.leave_balance')}
                    </CardTitle>
                </CardHeader>
                <CardContent className="flex items-center justify-center py-4">
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card className="h-full">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-orange-500" />
                        {t('dashboard.widgets.names.leave_balance')}
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card className="h-full">
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-orange-500" />
                    {t('dashboard.widgets.names.leave_balance')}
                </CardTitle>
                <CardDescription className="text-xs">{t('dashboard.widgets.hr.balanceDescription')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {data?.map((balance) => {
                    const percentage = balance.total > 0 ? Math.min(100, Math.max(0, (balance.used / balance.total) * 100)) : 0
                    const isAnnual = balance.leave_type === 'annual'

                    return (
                        <div key={balance.leave_type} className="space-y-1">
                            <div className="flex justify-between items-center text-sm">
                                <span className="font-medium">
                                    {isAnnual ? t('dashboard.widgets.hr.annualLeave') : t('dashboard.widgets.hr.compLeave')}
                                </span>
                                <span className="text-muted-foreground text-xs">
                                    {t('dashboard.widgets.hr.usedTotal', { used: balance.used, total: balance.total })}
                                </span>
                            </div>
                            <div className="space-y-2">
                                <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                    <div
                                        className="h-full bg-orange-500 transition-all"
                                        style={{ width: `${percentage}%` }}
                                    />
                                </div>
                                <div className="flex justify-start">
                                    <span className="text-[10px] font-semibold py-0.5 px-2 rounded-full text-orange-600 bg-orange-100 border border-orange-200">
                                        {balance.remaining} {balance.unit === 'day' ? t('dashboard.widgets.common.days') : t('dashboard.widgets.common.hours')}
                                    </span>
                                </div>
                            </div>
                        </div>
                    )
                })}
                {(!data || data.length === 0) && (
                    <div className="flex flex-col items-center justify-center py-4 text-muted-foreground">
                        <Info className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-xs">{t('dashboard.widgets.common.noData')}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
