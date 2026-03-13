import { Send, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
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
import { OVERTIME_TYPE_NAMES, formatDate } from '../constants'
import { OvertimeStatusBadge } from './OvertimeStatusBadge'

interface MyOvertimeTableRowProps {
    overtime: OvertimeWithUser
    onSubmit: (id: string) => void
    onDelete: (id: string) => void
    isSubmitting: boolean
    isDeleting: boolean
}

function MyOvertimeTableRow({
    overtime,
    onSubmit,
    onDelete,
    isSubmitting,
    isDeleting,
}: MyOvertimeTableRowProps) {
    return (
        <TableRow>
            <TableCell className="whitespace-nowrap">
                {formatDate(overtime.overtime_date)}
            </TableCell>
            <TableCell>
                {overtime.start_time} ~ {overtime.end_time}
            </TableCell>
            <TableCell>
                {OVERTIME_TYPE_NAMES[overtime.overtime_type] || overtime.overtime_type}
            </TableCell>
            <TableCell>{parseDecimal(overtime.hours).toFixed(1)} 小時</TableCell>
            <TableCell className="text-green-600 font-medium">
                {parseDecimal(overtime.comp_time_hours).toFixed(1)} 小時
            </TableCell>
            <TableCell className="max-w-[150px] truncate">{overtime.reason}</TableCell>
            <TableCell><OvertimeStatusBadge status={overtime.status} /></TableCell>
            <TableCell>
                <div className="flex gap-2">
                    {overtime.status === 'draft' && (
                        <>
                            <Button
                                variant="default"
                                size="sm"
                                onClick={() => onSubmit(overtime.id)}
                                disabled={isSubmitting}
                            >
                                <Send className="h-4 w-4 mr-1" />
                                送審
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onDelete(overtime.id)}
                                disabled={isDeleting}
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                        </>
                    )}
                </div>
            </TableCell>
        </TableRow>
    )
}

interface MyOvertimeTabContentProps {
    overtimeData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onSubmit: (id: string) => void
    onDelete: (id: string) => void
    isSubmitting: boolean
    isDeleting: boolean
}

export function MyOvertimeTabContent({
    overtimeData,
    isLoading,
    onSubmit,
    onDelete,
    isSubmitting,
    isDeleting,
}: MyOvertimeTabContentProps) {
    return (
        <TabsContent value="my-overtime" className="space-y-4">
            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>日期</TableHead>
                            <TableHead>時間</TableHead>
                            <TableHead>類型</TableHead>
                            <TableHead>加班時數</TableHead>
                            <TableHead>補休</TableHead>
                            <TableHead>事由</TableHead>
                            <TableHead>狀態</TableHead>
                            <TableHead>操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={8} className="text-left py-8">
                                    載入中...
                                </TableCell>
                            </TableRow>
                        ) : overtimeData?.data?.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={8} className="text-left py-8 text-muted-foreground">
                                    沒有加班記錄
                                </TableCell>
                            </TableRow>
                        ) : (
                            overtimeData?.data?.map((ot) => (
                                <MyOvertimeTableRow
                                    key={ot.id}
                                    overtime={ot}
                                    onSubmit={onSubmit}
                                    onDelete={onDelete}
                                    isSubmitting={isSubmitting}
                                    isDeleting={isDeleting}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
            </Card>
        </TabsContent>
    )
}
