import React, { useState } from 'react'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    AmendmentListItem,
    amendmentStatusNames,
    amendmentStatusColors,
    amendmentTypeNames,
    CreateAmendmentRequest,
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
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Loader2, Plus, FileEdit, Send, Eye } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Link } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'
import { useTranslation } from 'react-i18next'

interface AmendmentsTabProps {
    protocolId: string
    protocolStatus?: string
}

export function AmendmentsTab({ protocolId, protocolStatus }: AmendmentsTabProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { user } = useAuthStore()

    const dialogs = useDialogSet(['create'] as const)
    const [amendmentTitle, setAmendmentTitle] = useState('')
    const [amendmentDescription, setAmendmentDescription] = useState('')
    const [selectedChangeItems, setSelectedChangeItems] = useState<string[]>([])

    // 取得變更申請列表
    const { data: amendments, isLoading } = useQuery({
        queryKey: ['protocol-amendments', protocolId],
        queryFn: async () => {
            const response = await api.get<AmendmentListItem[]>(`/protocols/${protocolId}/amendments`)
            return response.data
        },
        enabled: !!protocolId,
    })

    const { sortedData: sortedAmendments, sort, toggleSort } = useTableSort(amendments)

    // 建立變更申請
    const createAmendmentMutation = useMutation({
        mutationFn: async (req: CreateAmendmentRequest) => {
            return api.post('/amendments', req)
        },
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('protocols.amendments.createSuccess') })
            queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
            resetForm()
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('protocols.amendments.createFailed')),
                variant: 'destructive',
            })
        },
    })

    // 提交變更申請
    const submitAmendmentMutation = useMutation({
        mutationFn: async (amendmentId: string) => {
            return api.post(`/amendments/${amendmentId}/submit`)
        },
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('protocols.amendments.submitSuccess') })
            queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('protocols.amendments.submitFailed')),
                variant: 'destructive',
            })
        },
    })

    const resetForm = () => {
        dialogs.close('create')
        setAmendmentTitle('')
        setAmendmentDescription('')
        setSelectedChangeItems([])
    }

    const handleCreateAmendment = () => {
        if (!amendmentTitle.trim()) {
            toast({ title: t('common.error'), description: t('protocols.amendments.validation.titleRequired'), variant: 'destructive' })
            return
        }
        if (selectedChangeItems.length === 0) {
            toast({ title: t('common.error'), description: t('protocols.amendments.validation.changeItemsRequired'), variant: 'destructive' })
            return
        }

        createAmendmentMutation.mutate({
            protocol_id: protocolId,
            title: amendmentTitle.trim(),
            description: amendmentDescription.trim() || undefined,
            change_items: selectedChangeItems,
        })
    }

    const handleToggleChangeItem = (value: string) => {
        setSelectedChangeItems(prev =>
            prev.includes(value)
                ? prev.filter(item => item !== value)
                : [...prev, value]
        )
    }

    // 檢查是否可以建立變更申請（計畫已核准）
    const canCreateAmendment =
        protocolStatus === 'APPROVED' ||
        protocolStatus === 'APPROVED_WITH_CONDITIONS'

    // 檢查是否為 PI 或相關角色
    const isPIorCoEditor = user?.roles?.some(r =>
        ['PI', 'EXPERIMENT_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
    )

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>{t('protocols.amendments.title')}</CardTitle>
                    <CardDescription>{t('protocols.amendments.description')}</CardDescription>
                </div>
                {canCreateAmendment && isPIorCoEditor && (
                    <Button onClick={() => dialogs.open('create')}>
                        <Plus className="mr-2 h-4 w-4" />
                        {t('protocols.amendments.create')}
                    </Button>
                )}
            </CardHeader>
            <CardContent>
                <div className="@container">

                    {/* Table view: container ≥ 600px */}
                    <div className="hidden @[600px]:block overflow-x-auto">
                        <Table className="w-full" style={{ minWidth: 580 }}>
                            <TableHeader>
                                <TableRow className="bg-muted/50 hover:bg-muted/50">
                                    <SortableTableHead style={{ width: 95 }} sortKey="amendment_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.amendments.table.amendmentNo')}</SortableTableHead>
                                    <TableHead style={{ minWidth: 140 }}>{t('protocols.amendments.table.title')}</TableHead>
                                    <SortableTableHead style={{ width: 85 }} sortKey="amendment_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.amendments.table.type')}</SortableTableHead>
                                    <SortableTableHead style={{ width: 85 }} sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.amendments.table.status')}</SortableTableHead>
                                    <SortableTableHead style={{ width: 160 }} sortKey="submitted_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[740px]:table-cell">{t('protocols.amendments.table.submittedAt')}</SortableTableHead>
                                    <TableHead style={{ width: 175 }} className="text-right">{t('protocols.amendments.table.actions')}</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {isLoading ? (
                                    <TableRow>
                                        <TableCell colSpan={6} className="p-0">
                                            <TableSkeleton rows={5} cols={6} />
                                        </TableCell>
                                    </TableRow>
                                ) : sortedAmendments && sortedAmendments.length > 0 ? (
                                    sortedAmendments.map((amendment) => (
                                        <TableRow key={amendment.id}>
                                            <TableCell style={{ width: 95 }} className="font-medium whitespace-nowrap">{amendment.amendment_no}</TableCell>
                                            <TableCell style={{ minWidth: 140 }} className="whitespace-normal break-words">{amendment.title}</TableCell>
                                            <TableCell style={{ width: 85 }}>
                                                <Badge variant="outline">
                                                    {amendmentTypeNames[amendment.amendment_type]}
                                                </Badge>
                                            </TableCell>
                                            <TableCell style={{ width: 85 }}>
                                                <Badge variant={amendmentStatusColors[amendment.status]}>
                                                    {amendmentStatusNames[amendment.status]}
                                                </Badge>
                                            </TableCell>
                                            <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[740px]:table-cell">
                                                {amendment.submitted_at ? formatDateTime(amendment.submitted_at) : '-'}
                                            </TableCell>
                                            <TableCell style={{ width: 175 }}>
                                                <div className="flex items-center justify-end gap-1">
                                                    {amendment.status === 'DRAFT' && isPIorCoEditor && (
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            onClick={() => submitAmendmentMutation.mutate(amendment.id)}
                                                            disabled={submitAmendmentMutation.isPending}
                                                        >
                                                            {submitAmendmentMutation.isPending ? (
                                                                <Loader2 className="h-4 w-4 animate-spin" />
                                                            ) : (
                                                                <>
                                                                    <Send className="mr-1 h-4 w-4" />
                                                                    {t('protocols.amendments.submit')}
                                                                </>
                                                            )}
                                                        </Button>
                                                    )}
                                                    <Button variant="ghost" size="sm" asChild>
                                                        <Link to={`/protocols/${protocolId}?tab=amendments&amendmentId=${amendment.id}`}>
                                                            <Eye className="mr-1 h-4 w-4" />
                                                            {t('protocols.amendments.view')}
                                                        </Link>
                                                    </Button>
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    ))
                                ) : (
                                    <TableEmptyRow colSpan={6} icon={FileEdit} title={t('protocols.amendments.noAmendments')} />
                                )}
                            </TableBody>
                        </Table>
                    </div>

                    {/* Card view: container < 600px */}
                    <div className="@[600px]:hidden space-y-3 py-1">
                        {isLoading ? (
                            <TableSkeleton rows={3} cols={1} />
                        ) : sortedAmendments && sortedAmendments.length > 0 ? (
                            sortedAmendments.map((amendment) => (
                                <div key={amendment.id} className="rounded-lg border bg-card p-3 space-y-2">
                                    <div className="flex items-center justify-between gap-2">
                                        <span className="text-sm font-semibold text-foreground">{amendment.amendment_no}</span>
                                        <Badge variant={amendmentStatusColors[amendment.status]}>
                                            {amendmentStatusNames[amendment.status]}
                                        </Badge>
                                    </div>
                                    <p className="text-sm text-muted-foreground leading-snug break-words">{amendment.title}</p>
                                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                                        <Badge variant="outline">
                                            {amendmentTypeNames[amendment.amendment_type]}
                                        </Badge>
                                        <div className="flex items-center gap-1">
                                            {amendment.status === 'DRAFT' && isPIorCoEditor && (
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    onClick={() => submitAmendmentMutation.mutate(amendment.id)}
                                                    disabled={submitAmendmentMutation.isPending}
                                                >
                                                    {submitAmendmentMutation.isPending ? (
                                                        <Loader2 className="h-4 w-4 animate-spin" />
                                                    ) : (
                                                        <>
                                                            <Send className="mr-1 h-4 w-4" />
                                                            {t('protocols.amendments.submit')}
                                                        </>
                                                    )}
                                                </Button>
                                            )}
                                            <Button variant="ghost" size="sm" asChild>
                                                <Link to={`/protocols/${protocolId}?tab=amendments&amendmentId=${amendment.id}`}>
                                                    <Eye className="mr-1 h-4 w-4" />
                                                    {t('protocols.amendments.view')}
                                                </Link>
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            ))
                        ) : (
                            <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                                <FileEdit className="h-8 w-8" />
                                <p className="text-sm">{t('protocols.amendments.noAmendments')}</p>
                            </div>
                        )}
                    </div>

                </div>
            </CardContent>

            {/* 建立變更申請對話框 */}
            <Dialog open={dialogs.isOpen('create')} onOpenChange={dialogs.setOpen('create')}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle>{t('protocols.amendments.dialog.title')}</DialogTitle>
                        <DialogDescription>
                            {t('protocols.amendments.dialog.description')}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label htmlFor="amendment-title">{t('protocols.amendments.dialog.amendmentTitle')}</Label>
                            <Input
                                id="amendment-title"
                                value={amendmentTitle}
                                onChange={(e) => setAmendmentTitle(e.target.value)}
                                placeholder={t('protocols.amendments.dialog.titlePlaceholder')}
                            />
                        </div>

                        <div className="space-y-2">
                            <Label>{t('protocols.amendments.dialog.changeItems')}</Label>
                            <div className="grid grid-cols-2 gap-2 mt-2">
                                {AMENDMENT_CHANGE_ITEM_OPTIONS.map((option) => (
                                    <div key={option.value} className="flex items-center space-x-2">
                                        <Checkbox
                                            id={`change-item-${option.value}`}
                                            checked={selectedChangeItems.includes(option.value)}
                                            onCheckedChange={() => handleToggleChangeItem(option.value)}
                                        />
                                        <label
                                            htmlFor={`change-item-${option.value}`}
                                            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                                        >
                                            {t(`amendments.changeItemLabels.${option.value}`, { defaultValue: option.label })}
                                        </label>
                                    </div>
                                ))}
                            </div>
                            {selectedChangeItems.length > 0 && (
                                <p className="text-xs text-muted-foreground mt-1">
                                    {t('protocols.amendments.dialog.selectedCount', { count: selectedChangeItems.length })}
                                </p>
                            )}
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="amendment-description">{t('protocols.amendments.dialog.amendmentDescription')}</Label>
                            <Textarea
                                id="amendment-description"
                                value={amendmentDescription}
                                onChange={(e) => setAmendmentDescription(e.target.value)}
                                placeholder={t('protocols.amendments.dialog.descriptionPlaceholder')}
                                rows={4}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={resetForm}>
                            {t('protocols.amendments.dialog.cancel')}
                        </Button>
                        <Button
                            onClick={handleCreateAmendment}
                            disabled={createAmendmentMutation.isPending}
                        >
                            {createAmendmentMutation.isPending && (
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            )}
                            {t('protocols.amendments.dialog.create')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </Card>
    )
}
