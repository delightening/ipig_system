import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Calendar, Clock, Loader2 } from 'lucide-react'
import api from '@/lib/api'

interface LeaveBalanceSummary {
    annual_leave_total: number
    annual_leave_used: number
    annual_leave_remaining: number
    comp_time_total: number
    comp_time_used: number
    comp_time_remaining: number
    expiring_soon_days: number
    expiring_soon_hours: number
}

export function LeaveBalanceWidget() {
    const { data, isLoading, error } = useQuery({
        queryKey: ['hr-balance-summary'],
        queryFn: async () => {
            const res = await api.get<LeaveBalanceSummary>('/hr/balances/summary')
            return res.data
        },
    })

    if (isLoading) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-blue-500" />
                        請假餘額
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="flex items-center justify-center py-4">
                        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    </div>
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-blue-500" />
                        請假餘額
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card>
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-blue-500" />
                    請假餘額
                </CardTitle>
                <CardDescription>您的假期餘額摘要</CardDescription>
            </CardHeader>
            <CardContent>
                <div className="space-y-3">
                    {/* 特休 */}
                    <div className="flex justify-between items-center">
                        <span className="text-sm text-muted-foreground">特休</span>
                        <div className="text-right">
                            <span className="text-lg font-semibold text-blue-600">
                                {data?.annual_leave_remaining ?? 0}
                            </span>
                            <span className="text-sm text-muted-foreground ml-1">天</span>
                            <span className="text-xs text-muted-foreground ml-2">
                                (已用 {data?.annual_leave_used ?? 0} / 共 {data?.annual_leave_total ?? 0})
                            </span>
                        </div>
                    </div>

                    {/* 補休 */}
                    <div className="flex justify-between items-center">
                        <span className="text-sm text-muted-foreground">補休</span>
                        <div className="text-right">
                            <span className="text-lg font-semibold text-green-600">
                                {data?.comp_time_remaining ?? 0}
                            </span>
                            <span className="text-sm text-muted-foreground ml-1">小時</span>
                            <span className="text-xs text-muted-foreground ml-2">
                                (已用 {data?.comp_time_used ?? 0} / 共 {data?.comp_time_total ?? 0})
                            </span>
                        </div>
                    </div>

                    {/* 即將到期提醒 */}
                    {((data?.expiring_soon_days ?? 0) > 0 || (data?.expiring_soon_hours ?? 0) > 0) && (
                        <div className="mt-2 p-2 bg-yellow-50 rounded-lg border border-yellow-200">
                            <div className="flex items-center gap-2 text-yellow-700 text-xs">
                                <Clock className="h-3 w-3" />
                                <span>
                                    即將到期：
                                    {(data?.expiring_soon_days ?? 0) > 0 && `特休 ${data?.expiring_soon_days} 天`}
                                    {(data?.expiring_soon_days ?? 0) > 0 && (data?.expiring_soon_hours ?? 0) > 0 && '、'}
                                    {(data?.expiring_soon_hours ?? 0) > 0 && `補休 ${data?.expiring_soon_hours} 小時`}
                                </span>
                            </div>
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
