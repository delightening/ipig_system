import { Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
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
import { formatLeaveHours, getLeaveStatusVariant } from '../constants'

interface MyLeavesTabContentProps {
    leaves: LeaveRequestWithUser[] | undefined
    isLoading: boolean
    onSubmit: (id: string) => void
    onCancel: (id: string) => void
    submitPending: boolean
    cancelPending: boolean
}

export function MyLeavesTabContent({
    leaves,
    isLoading,
    onSubmit,
    onCancel,
    submitPending,
    cancelPending,
}: MyLeavesTabContentProps) {
    return (
        <Card>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>假別</TableHead>
                        <TableHead>日期</TableHead>
                        <TableHead>時數</TableHead>
                        <TableHead>事由</TableHead>
                        <TableHead>狀態</TableHead>
                        <TableHead>操作</TableHead>
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
                                沒有請假記錄
                            </TableCell>
                        </TableRow>
                    ) : (
                        leaves?.map((leave) => {
                            const status = getLeaveStatusVariant(leave.status)
                            return (
                                <TableRow key={leave.id}>
                                    <TableCell>{LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type}</TableCell>
                                    <TableCell className="whitespace-nowrap">
                                        {formatDate(leave.start_date)}
                                        {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                    </TableCell>
                                    <TableCell>{formatLeaveHours(leave)}</TableCell>
                                    <TableCell className="max-w-[200px] truncate">{leave.reason}</TableCell>
                                    <TableCell>
                                        <Badge variant={status.variant} className={status.className}>
                                            {status.label}
                                        </Badge>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-2">
                                            {leave.status === 'DRAFT' && (
                                                <>
                                                    <Button
                                                        variant="default"
                                                        size="sm"
                                                        onClick={() => onSubmit(leave.id)}
                                                        disabled={submitPending}
                                                    >
                                                        <Send className="h-4 w-4 mr-1" />
                                                        送審
                                                    </Button>
                                                    <Button
                                                        variant="destructive"
                                                        size="sm"
                                                        onClick={() => onCancel(leave.id)}
                                                        disabled={cancelPending}
                                                    >
                                                        <Trash2 className="h-4 w-4" />
                                                    </Button>
                                                </>
                                            )}
                                            {(leave.status.startsWith('PENDING') || leave.status === 'APPROVED') && (
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    onClick={() => onCancel(leave.id)}
                                                    disabled={cancelPending}
                                                >
                                                    取消
                                                </Button>
                                            )}
                                        </div>
                                    </TableCell>
                                </TableRow>
                            )
                        })
                    )}
                </TableBody>
            </Table>
        </Card>
    )
}
