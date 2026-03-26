import React, { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import api, {
    AmendmentListItem,
    amendmentStatusColors,
} from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Badge } from '@/components/ui/badge'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
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
import { Loader2, FileEdit, Eye } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'

export function MyAmendmentsPage() {
    const { t } = useTranslation()
    const [statusFilter, setStatusFilter] = useState<string>('ALL')

    const statusFilterOptions = useMemo(() => [
        { value: 'ALL', label: t('amendments.allStatus') },
        { value: 'DRAFT', label: t('amendments.status.DRAFT') },
        { value: 'SUBMITTED', label: t('amendments.status.SUBMITTED') },
        { value: 'CLASSIFIED', label: t('amendments.status.CLASSIFIED') },
        { value: 'UNDER_REVIEW', label: t('amendments.status.UNDER_REVIEW') },
        { value: 'REVISION_REQUIRED', label: t('amendments.status.REVISION_REQUIRED') },
        { value: 'APPROVED', label: t('amendments.status.APPROVED') },
        { value: 'REJECTED', label: t('amendments.status.REJECTED') },
    ], [t])

    // 取得使用者的變更申請列表
    const { data: amendments, isLoading } = useQuery({
        queryKey: ['my-amendments'],
        queryFn: async () => {
            const response = await api.get<AmendmentListItem[]>('/amendments')
            return response.data
        },
    })

    // 篩選後的列表
    const filteredAmendments = amendments?.filter(amendment => {
        if (statusFilter === 'ALL') return true
        return amendment.status === statusFilter
    })

    const { sortedData: sortedAmendments, sort, toggleSort } = useTableSort(filteredAmendments)

    // 取得變更項目的顯示標籤
    const getChangeItemLabels = (changeItems?: string[]) => {
        if (!changeItems || changeItems.length === 0) return '-'
        return changeItems
            .map(item => {
                // 後端可能回傳小寫（如 animal_count），翻譯 key 為大寫（如 ANIMAL_COUNT）
                const key = item.toUpperCase()
                const translated = t(`amendments.changeItemLabels.${key}`, { defaultValue: '' })
                return translated || item
            })
            .join('、')
    }

    return (
        <div className="space-y-6">
            <PageHeader
                title={t('amendments.title')}
                description={t('amendments.description')}
            />

            <div className="space-y-4">
                <div className="flex flex-wrap gap-4 items-end">
                    <Select value={statusFilter} onValueChange={setStatusFilter}>
                        <SelectTrigger className="w-40">
                            <SelectValue placeholder={t('common.allStatus')} />
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
            </div>

            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead sortKey="amendment_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.amendmentNo')}
                            </SortableTableHead>
                            <SortableTableHead sortKey="protocol_iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.protocol')}
                            </SortableTableHead>
                            <SortableTableHead sortKey="title" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.title')}
                            </SortableTableHead>
                            <TableHead>{t('amendments.columns.changeItems')}</TableHead>
                            <SortableTableHead sortKey="amendment_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.type')}
                            </SortableTableHead>
                            <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.status')}
                            </SortableTableHead>
                            <SortableTableHead sortKey="submitted_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                {t('amendments.columns.submittedAt')}
                            </SortableTableHead>
                            <TableHead className="text-right">{t('amendments.columns.actions')}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                                {isLoading ? (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8">
                                            <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                                        </TableCell>
                                    </TableRow>
                                ) : sortedAmendments && sortedAmendments.length > 0 ? (
                                    sortedAmendments.map((amendment) => (
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
                                                    {t(`amendments.types.${amendment.amendment_type}`)}
                                                </Badge>
                                            </TableCell>
                                            <TableCell>
                                                <Badge variant={amendmentStatusColors[amendment.status]}>
                                                    {t(`amendments.status.${amendment.status}`)}
                                                </Badge>
                                            </TableCell>
                                            <TableCell>
                                                {amendment.submitted_at
                                                    ? formatDateTime(amendment.submitted_at)
                                                    : '-'}
                                            </TableCell>
                                            <TableCell>
                                                <div className="flex items-center justify-end gap-1">
                                                    <Button variant="ghost" size="sm" asChild>
                                                        <Link to={`/protocols/${amendment.protocol_id}?tab=amendments`}>
                                                            <Eye className="mr-1 h-4 w-4" />
                                                            {t('common.view')}
                                                        </Link>
                                                    </Button>
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    ))
                                ) : (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8">
                                            <FileEdit className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                                            <p className="text-muted-foreground font-medium">
                                                {statusFilter === 'ALL' ? t('amendments.noData') : t('amendments.noDataFiltered')}
                                            </p>
                                            <p className="text-sm text-muted-foreground mt-1">
                                                {statusFilter === 'ALL'
                                                    ? t('amendments.emptyHint')
                                                    : t('amendments.filterHint')}
                                            </p>
                                        </TableCell>
                                    </TableRow>
                                )}
                    </TableBody>
                </Table>
            </div>
        </div>
    )
}
