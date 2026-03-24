import React, { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import api, {
    AmendmentListItem,
    amendmentStatusColors,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
            <div>
                <h1 className="text-3xl font-bold tracking-tight">{t('amendments.title')}</h1>
                <p className="text-muted-foreground">
                    {t('amendments.description')}
                </p>
            </div>

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
                            <TableHead>{t('amendments.columns.amendmentNo')}</TableHead>
                            <TableHead>{t('amendments.columns.protocol')}</TableHead>
                            <TableHead>{t('amendments.columns.title')}</TableHead>
                            <TableHead>{t('amendments.columns.changeItems')}</TableHead>
                            <TableHead>{t('amendments.columns.type')}</TableHead>
                            <TableHead>{t('amendments.columns.status')}</TableHead>
                            <TableHead>{t('amendments.columns.submittedAt')}</TableHead>
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
                                ) : filteredAmendments && filteredAmendments.length > 0 ? (
                                    filteredAmendments.map((amendment) => (
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
