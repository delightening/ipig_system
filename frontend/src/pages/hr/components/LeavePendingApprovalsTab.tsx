import { CheckCircle, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { formatDate } from '@/lib/utils'
import { LEAVE_TYPE_NAMES } from '@/types/hr'
import type { LeaveRequestWithUser } from '@/types/hr'
import { formatLeaveHours } from '../constants'

interface LeavePendingApprovalsTabProps {
    leaves: LeaveRequestWithUser[] | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    approvePending: boolean
    rejectPending: boolean
}

export function LeavePendingApprovalsTab({
    leaves,
    isLoading,
    onApprove,
    onReject,
    approvePending,
    rejectPending,
}: LeavePendingApprovalsTabProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>待審核請假</CardTitle>
                <CardDescription>您需要審核的請假申請</CardDescription>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>申請人</TableHead>
                            <TableHead>假別</TableHead>
                            <TableHead>日期</TableHead>
                            <TableHead>時數</TableHead>
                            <TableHead>事由</TableHead>
                            <TableHead className="text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center py-8">
                                    載入中...
                                </TableCell>
                            </TableRow>
                        ) : leaves?.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                    沒有待審核的請假
                                </TableCell>
                            </TableRow>
                        ) : (
                            leaves?.map((leave) => (
                                <TableRow key={leave.id}>
                                    <TableCell>
                                        <div>
                                            <div className="font-medium">{leave.user_name}</div>
                                            <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                                        </div>
                                    </TableCell>
                                    <TableCell>{LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type}</TableCell>
                                    <TableCell className="whitespace-nowrap">
                                        {formatDate(leave.start_date)}
                                        {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                    </TableCell>
                                    <TableCell>{formatLeaveHours(leave)}</TableCell>
                                    <TableCell className="max-w-[200px] truncate">{leave.reason}</TableCell>
                                    <TableCell>
                                        <div className="flex items-center justify-end gap-1">
                                            <Button
                                                variant="default"
                                                size="sm"
                                                onClick={() => onApprove(leave.id)}
                                                disabled={approvePending}
                                            >
                                                <CheckCircle className="h-4 w-4 mr-1" />
                                                核准
                                            </Button>
                                            <Button
                                                variant="destructive"
                                                size="sm"
                                                onClick={() => onReject(leave.id, '不符合規定')}
                                                disabled={rejectPending}
                                            >
                                                <XCircle className="h-4 w-4 mr-1" />
                                                駁回
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
