import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Users, Loader2 } from 'lucide-react'
import api from '@/lib/api'

interface StaffAttendanceStats {
    user_id: string
    display_name: string
    attendance_days: number
    late_count: number
    leave_days: number
    overtime_hours: number
}

export function StaffAttendanceWidget() {
    const { data, isLoading, error } = useQuery({
        queryKey: ['staff-attendance-stats'],
        queryFn: async () => {
            // 取得本月工作人員出勤統計
            const now = new Date()
            const startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split('T')[0]
            const endDate = now.toISOString().split('T')[0]
            const res = await api.get<{ data: StaffAttendanceStats[] }>(
                `/hr/attendance/stats?start_date=${startDate}&end_date=${endDate}`
            )
            return res.data.data
        },
    })

    if (isLoading) {
        return (
            <Card className="col-span-2">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Users className="h-4 w-4 text-indigo-500" />
                        工作人員出勤表
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
            <Card className="col-span-2">
                <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Users className="h-4 w-4 text-indigo-500" />
                        工作人員出勤表
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-sm text-muted-foreground">載入失敗</p>
                </CardContent>
            </Card>
        )
    }

    const currentMonth = new Date().toLocaleDateString('zh-TW', { year: 'numeric', month: 'long' })

    return (
        <Card className="col-span-2">
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <Users className="h-4 w-4 text-indigo-500" />
                    工作人員出勤表
                </CardTitle>
                <CardDescription>{currentMonth} 出勤統計</CardDescription>
            </CardHeader>
            <CardContent>
                {data && data.length > 0 ? (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>姓名</TableHead>
                                <TableHead className="text-right">出勤天數</TableHead>
                                <TableHead className="text-right">遲到次數</TableHead>
                                <TableHead className="text-right">請假天數</TableHead>
                                <TableHead className="text-right">加班時數</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {data.map((staff) => (
                                <TableRow key={staff.user_id}>
                                    <TableCell className="font-medium">{staff.display_name}</TableCell>
                                    <TableCell className="text-right">{staff.attendance_days}</TableCell>
                                    <TableCell className="text-right">
                                        <span className={staff.late_count > 0 ? 'text-yellow-600' : ''}>
                                            {staff.late_count}
                                        </span>
                                    </TableCell>
                                    <TableCell className="text-right">{staff.leave_days}</TableCell>
                                    <TableCell className="text-right">
                                        <span className={staff.overtime_hours > 0 ? 'text-blue-600' : ''}>
                                            {staff.overtime_hours}
                                        </span>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                ) : (
                    <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
                        <Users className="h-8 w-8 mb-2" />
                        <p className="text-sm">本月尚無出勤記錄</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
