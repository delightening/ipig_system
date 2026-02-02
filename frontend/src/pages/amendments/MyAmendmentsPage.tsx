import React, { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import api, {
    AmendmentListItem,
    AmendmentStatus,
    amendmentStatusNames,
    amendmentStatusColors,
    amendmentTypeNames,
    AMENDMENT_CHANGE_ITEM_OPTIONS,
} from '@/lib/api'
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Loader2, FileEdit, Eye, Filter } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'

const statusFilterOptions: { value: string; label: string }[] = [
    { value: 'ALL', label: '全部狀態' },
    { value: 'DRAFT', label: '草稿' },
    { value: 'SUBMITTED', label: '已提交' },
    { value: 'CLASSIFIED', label: '已分類' },
    { value: 'UNDER_REVIEW', label: '審查中' },
    { value: 'REVISION_REQUIRED', label: '需修訂' },
    { value: 'APPROVED', label: '已核准' },
    { value: 'REJECTED', label: '已否決' },
]

export function MyAmendmentsPage() {
    const [statusFilter, setStatusFilter] = useState<string>('ALL')

    // 取得使用者的變更申請列表
    const { data: amendments, isLoading } = useQuery({
        queryKey: ['my-amendments'],
        queryFn: async () => {
            const response = await api.get<AmendmentListItem[]>('/amendments/my')
            return response.data
        },
    })

    // 篩選後的列表
    const filteredAmendments = amendments?.filter(amendment => {
        if (statusFilter === 'ALL') return true
        return amendment.status === statusFilter
    })

    // 取得變更項目的顯示標籤
    const getChangeItemLabels = (changeItems?: string[]) => {
        if (!changeItems || changeItems.length === 0) return '-'
        return changeItems
            .map(item => AMENDMENT_CHANGE_ITEM_OPTIONS.find(opt => opt.value === item)?.label || item)
            .join('、')
    }

    return (
        <div className="space-y-6">
            <div>
                <h1 className="text-3xl font-bold">我的變更申請</h1>
                <p className="text-muted-foreground mt-2">
                    查看和管理您提交的計畫書變更申請
                </p>
            </div>

            <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                    <div>
                        <CardTitle>變更申請列表</CardTitle>
                        <CardDescription>
                            {amendments?.length || 0} 筆變更申請
                            {statusFilter !== 'ALL' && ` (顯示 ${filteredAmendments?.length || 0} 筆)`}
                        </CardDescription>
                    </div>
                    <div className="flex items-center gap-2">
                        <Filter className="h-4 w-4 text-muted-foreground" />
                        <Select value={statusFilter} onValueChange={setStatusFilter}>
                            <SelectTrigger className="w-[150px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {statusFilterOptions.map(option => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </CardHeader>
                <CardContent>
                    {isLoading ? (
                        <div className="flex items-center justify-center py-12">
                            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                        </div>
                    ) : filteredAmendments && filteredAmendments.length > 0 ? (
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>變更編號</TableHead>
                                    <TableHead>計畫書</TableHead>
                                    <TableHead>標題</TableHead>
                                    <TableHead>變更項目</TableHead>
                                    <TableHead>類型</TableHead>
                                    <TableHead>狀態</TableHead>
                                    <TableHead>提交時間</TableHead>
                                    <TableHead>操作</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {filteredAmendments.map((amendment) => (
                                    <TableRow key={amendment.id}>
                                        <TableCell className="font-medium">
                                            {amendment.amendment_no}
                                        </TableCell>
                                        <TableCell>
                                            <div>
                                                <div className="font-medium">{amendment.protocol_iacuc_no || '-'}</div>
                                                <div className="text-sm text-muted-foreground truncate max-w-[200px]">
                                                    {amendment.protocol_title}
                                                </div>
                                            </div>
                                        </TableCell>
                                        <TableCell>{amendment.title}</TableCell>
                                        <TableCell>
                                            <span className="text-sm text-muted-foreground">
                                                {getChangeItemLabels(amendment.change_items)}
                                            </span>
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant="outline">
                                                {amendmentTypeNames[amendment.amendment_type]}
                                            </Badge>
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant={amendmentStatusColors[amendment.status]}>
                                                {amendmentStatusNames[amendment.status]}
                                            </Badge>
                                        </TableCell>
                                        <TableCell>
                                            {amendment.submitted_at
                                                ? formatDateTime(amendment.submitted_at)
                                                : '-'}
                                        </TableCell>
                                        <TableCell>
                                            <Button variant="ghost" size="sm" asChild>
                                                <Link to={`/protocols/${amendment.protocol_id}?tab=amendments`}>
                                                    <Eye className="mr-1 h-4 w-4" />
                                                    查看
                                                </Link>
                                            </Button>
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    ) : (
                        <div className="text-center py-12 text-muted-foreground">
                            <FileEdit className="h-12 w-12 mx-auto mb-4" />
                            <h3 className="text-lg font-semibold mb-2">
                                {statusFilter === 'ALL' ? '尚無變更申請' : '無符合條件的變更申請'}
                            </h3>
                            <p className="text-sm">
                                {statusFilter === 'ALL'
                                    ? '您可以在已核准的計畫書中提出變更申請'
                                    : '請嘗試選擇其他狀態篩選條件'}
                            </p>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    )
}
