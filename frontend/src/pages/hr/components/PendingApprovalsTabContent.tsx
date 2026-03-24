import { CheckCircle, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { TabsContent } from '@/components/ui/tabs'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { parseDecimal } from '@/lib/utils'
import type { OvertimeWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { formatDate } from '../constants'

interface PendingApprovalRowProps {
    overtime: OvertimeWithUser
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    isApproving: boolean
    isRejecting: boolean
}

function PendingApprovalRow({
    overtime,
    onApprove,
    onReject,
    isApproving,
    isRejecting,
}: PendingApprovalRowProps) {
    return (
        <TableRow>
            <TableCell>
                <div>
                    <div className="font-medium">{overtime.user_name}</div>
                    <div className="text-sm text-muted-foreground">{overtime.user_email}</div>
                </div>
            </TableCell>
            <TableCell className="whitespace-nowrap">
                {formatDate(overtime.overtime_date)}
            </TableCell>
            <TableCell>
                {overtime.start_time} ~ {overtime.end_time}
            </TableCell>
            <TableCell>{parseDecimal(overtime.hours).toFixed(1)} 小時</TableCell>
            <TableCell className="max-w-[200px] truncate">{overtime.reason}</TableCell>
            <TableCell>
                <div className="flex items-center justify-end gap-1">
                    <Button
                        variant="default"
                        size="sm"
                        onClick={() => onApprove(overtime.id)}
                        disabled={isApproving}
                    >
                        <CheckCircle className="h-4 w-4 mr-1" />
                        核准
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => onReject(overtime.id, '不符合規定')}
                        disabled={isRejecting}
                    >
                        <XCircle className="h-4 w-4 mr-1" />
                        駁回
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    )
}

interface PendingApprovalsTabContentProps {
    pendingData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    isApproving: boolean
    isRejecting: boolean
}

export function PendingApprovalsTabContent({
    pendingData,
    isLoading,
    onApprove,
    onReject,
    isApproving,
    isRejecting,
}: PendingApprovalsTabContentProps) {
    return (
        <TabsContent value="approvals" className="space-y-4">
            <Card>
                <CardHeader>
                    <CardTitle>待審核加班</CardTitle>
                    <CardDescription>您需要審核的加班申請</CardDescription>
                </CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>申請人</TableHead>
                                <TableHead>日期</TableHead>
                                <TableHead>時間</TableHead>
                                <TableHead>加班時數</TableHead>
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
                            ) : pendingData?.data?.length === 0 ? (
                                <TableRow>
                                    <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                        沒有待審核的加班
                                    </TableCell>
                                </TableRow>
                            ) : (
                                pendingData?.data?.map((ot) => (
                                    <PendingApprovalRow
                                        key={ot.id}
                                        overtime={ot}
                                        onApprove={onApprove}
                                        onReject={onReject}
                                        isApproving={isApproving}
                                        isRejecting={isRejecting}
                                    />
                                ))
                            )}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </TabsContent>
    )
}
