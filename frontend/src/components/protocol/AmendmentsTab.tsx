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
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2, Plus, FileEdit, Send, Eye, CheckCircle2, History } from 'lucide-react'
import { HistoricalAmendmentDialog } from './HistoricalAmendmentDialog'
import { StructuredChangeEditor, type ChangeDetailRow } from './StructuredChangeEditor'
import { formatDateTime } from '@/lib/utils'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
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
    /** 是否為匯入計劃（protocol.imported_at 非 null）：決定是否顯示「補登歷史變更」入口（P6） */
    isImported?: boolean
    /** 目前使用者是否為本計劃的負責人 SD（protocol.study_director_user_id === user.id） */
    isStudyDirector?: boolean
    /** 後端權威：是否可建立/更新/提交此計畫的變更申請（= access::can_write_amendment：
     * admin / PI，不含 SD；外部 PI 計畫另含 SD 核准的生效中代理人）。供修正案建立·編輯·
     * 送審按鈕 gating。 */
    canWriteAmendment?: boolean
    /** 當前使用者是否以 PI 代理人身分取得上面的 canWriteAmendment（而非本人是 PI/admin）。
     * 純顯示徽章，不影響任何授權判斷——避免讓人誤以為是 PI 本人操作。 */
    isPiDelegate?: boolean
}

export function AmendmentsTab({ protocolId, protocolStatus, isImported, isStudyDirector, canWriteAmendment, isPiDelegate }: AmendmentsTabProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { hasPermission } = useAuthStore()
    // R30-25c：標記為 EFFECTIVE 的權限與 change_status 同層（IACUC_STAFF/CHAIR/admin）
    const canMarkEffective = hasPermission('aup.protocol.change_status')

    // R30-25c follow-up：用專案統一 useConfirmDialog 取代 window.confirm
    const { dialogState: confirmState, confirm } = useConfirmDialog()

    const dialogs = useDialogSet(['create', 'historical'] as const)
    // P6：補登歷史變更入口 — 限匯入計劃（isImported）且使用者為本計劃負責人(SD)/管理者；
    // 與後端 access::can_backfill_historical_amendment 對齊（SD 比對而非 EXPERIMENT_STAFF 角色）。
    const canBackfill =
        !!isImported &&
        (hasPermission('aup.protocol.import_approved') || !!isStudyDirector)
    const [amendmentTitle, setAmendmentTitle] = useState('')
    const [amendmentDescription, setAmendmentDescription] = useState('')
    const [selectedChangeItems, setSelectedChangeItems] = useState<string[]>([])
    // R71-12：結構化變更明細（變更目的 + 多列 項次/改動前/改動後），存入 changes_content
    const [changePurpose, setChangePurpose] = useState('')
    const [changeDetails, setChangeDetails] = useState<ChangeDetailRow[]>([])

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

    // R30-25c：標記 amendment 為 EFFECTIVE（GLP §58 正式生效）
    const markEffectiveMutation = useMutation({
        mutationFn: async (amendmentId: string) => {
            return api.post(`/amendments/${amendmentId}/effective`, {})
        },
        onSuccess: () => {
            toast({
                title: t('common.success'),
                description: t('protocols.amendments.markEffectiveSuccess', { defaultValue: '已標記為生效' }),
            })
            queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(
                    error,
                    t('protocols.amendments.markEffectiveFailed', { defaultValue: '標記生效失敗' }),
                ),
                variant: 'destructive',
            })
        },
    })

    const handleMarkEffective = async (amendmentId: string, amendmentNo: string) => {
        const ok = await confirm({
            title: t('protocols.amendments.markEffectiveConfirmTitle', {
                defaultValue: '標記為生效',
            }),
            description: t('protocols.amendments.markEffectiveConfirm', {
                defaultValue: '確定將 {{no}} 標記為生效？此操作會記錄到稽核軌跡且不可逆。',
                no: amendmentNo,
            }),
            variant: 'destructive',
            confirmLabel: t('protocols.amendments.markEffective', { defaultValue: '標記生效' }),
        })
        if (ok) markEffectiveMutation.mutate(amendmentId)
    }

    const resetForm = () => {
        dialogs.close('create')
        setAmendmentTitle('')
        setAmendmentDescription('')
        setSelectedChangeItems([])
        setChangePurpose('')
        setChangeDetails([])
    }

    const addChangeDetail = () =>
        setChangeDetails((rows) => [...rows, { id: crypto.randomUUID(), section: '', before: '', after: '' }])
    const removeChangeDetail = (idx: number) =>
        setChangeDetails((rows) => rows.filter((_, i) => i !== idx))
    const updateChangeDetail = (idx: number, field: 'section' | 'before' | 'after', value: string) =>
        setChangeDetails((rows) => rows.map((r, i) => (i === idx ? { ...r, [field]: value } : r)))

    const handleCreateAmendment = () => {
        if (!amendmentTitle.trim()) {
            toast({ title: t('common.error'), description: t('protocols.amendments.validation.titleRequired'), variant: 'destructive' })
            return
        }
        if (selectedChangeItems.length === 0) {
            toast({ title: t('common.error'), description: t('protocols.amendments.validation.changeItemsRequired'), variant: 'destructive' })
            return
        }

        // R71-12：只保留有填項次的列；目的或明細任一有內容才送 changes_content
        const items = changeDetails
            .filter((r) => r.section.trim() || r.before.trim() || r.after.trim())
            .map((r) => ({ section: r.section.trim(), before: r.before.trim(), after: r.after.trim() }))
        const purpose = changePurpose.trim()
        const changes_content =
            purpose || items.length > 0
                ? { ...(purpose ? { purpose } : {}), ...(items.length > 0 ? { items } : {}) }
                : undefined

        createAmendmentMutation.mutate({
            protocol_id: protocolId,
            title: amendmentTitle.trim(),
            description: amendmentDescription.trim() || undefined,
            change_items: selectedChangeItems,
            changes_content,
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

    // 與後端 access::can_write_amendment 對齊（admin / PI，不含 SD）：原本用角色陣列
    // + isStudyDirector 硬判，會讓 SD、EXPERIMENT_STAFF 看到必定 403 的按鈕。
    const canManageAmendment = !!canWriteAmendment

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>{t('protocols.amendments.title')}</CardTitle>
                    <CardDescription>{t('protocols.amendments.description')}</CardDescription>
                </div>
                <div className="flex items-center gap-2">
                    {canBackfill && (
                        <Button variant="outline" onClick={() => dialogs.open('historical')}>
                            <History className="mr-2 h-4 w-4" />
                            補登歷史變更
                        </Button>
                    )}
                    {canCreateAmendment && canManageAmendment && (
                        <>
                            {isPiDelegate && (
                                <Badge variant="outline">以 PI 代理人身分操作</Badge>
                            )}
                            <Button onClick={() => dialogs.open('create')}>
                                <Plus className="mr-2 h-4 w-4" />
                                {t('protocols.amendments.create')}
                            </Button>
                        </>
                    )}
                </div>
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
                                            <TableCell style={{ width: 95 }} className="font-medium whitespace-nowrap">
                                                {amendment.amendment_no}
                                                {amendment.is_historical && (
                                                    <Badge variant="outline" className="ml-1 text-[10px] align-middle">補登</Badge>
                                                )}
                                            </TableCell>
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
                                                {amendment.effective_from && (
                                                    <div className="text-[10px] text-muted-foreground mt-1 leading-tight">
                                                        {t('protocols.amendments.effectiveSince', {
                                                            defaultValue: '生效於 {{date}}',
                                                            date: formatDateTime(amendment.effective_from),
                                                        })}
                                                    </div>
                                                )}
                                            </TableCell>
                                            <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[740px]:table-cell">
                                                {amendment.submitted_at ? formatDateTime(amendment.submitted_at) : '-'}
                                            </TableCell>
                                            <TableCell style={{ width: 175 }}>
                                                <div className="flex items-center justify-end gap-1">
                                                    {amendment.status === 'DRAFT' && canManageAmendment && (
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
                                                    {(amendment.status === 'APPROVED' || amendment.status === 'ADMIN_APPROVED') &&
                                                        canMarkEffective && (
                                                            <Button
                                                                variant="outline"
                                                                size="sm"
                                                                onClick={() => handleMarkEffective(amendment.id, amendment.amendment_no)}
                                                                disabled={markEffectiveMutation.isPending}
                                                            >
                                                                {markEffectiveMutation.isPending ? (
                                                                    <Loader2 className="h-4 w-4 animate-spin" />
                                                                ) : (
                                                                    <>
                                                                        <CheckCircle2 className="mr-1 h-4 w-4" />
                                                                        {t('protocols.amendments.markEffective', { defaultValue: '標記生效' })}
                                                                    </>
                                                                )}
                                                            </Button>
                                                        )}
                                                    <Button variant="ghost" size="sm" asChild>
                                                        <Link to={`/protocols/amendments/${amendment.id}`}>
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
                                        <span className="text-sm font-semibold text-foreground">
                                            {amendment.amendment_no}
                                            {amendment.is_historical && (
                                                <Badge variant="outline" className="ml-1 text-[10px] align-middle">補登</Badge>
                                            )}
                                        </span>
                                        <Badge variant={amendmentStatusColors[amendment.status]}>
                                            {amendmentStatusNames[amendment.status]}
                                        </Badge>
                                    </div>
                                    <p className="text-sm text-muted-foreground leading-snug break-words">{amendment.title}</p>
                                    {amendment.effective_from && (
                                        <p className="text-xs text-muted-foreground">
                                            {t('protocols.amendments.effectiveSince', {
                                                defaultValue: '生效於 {{date}}',
                                                date: formatDateTime(amendment.effective_from),
                                            })}
                                        </p>
                                    )}
                                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                                        <Badge variant="outline">
                                            {amendmentTypeNames[amendment.amendment_type]}
                                        </Badge>
                                        <div className="flex items-center gap-1">
                                            {amendment.status === 'DRAFT' && canManageAmendment && (
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
                                            {(amendment.status === 'APPROVED' || amendment.status === 'ADMIN_APPROVED') &&
                                                canMarkEffective && (
                                                    <Button
                                                        variant="outline"
                                                        size="sm"
                                                        onClick={() => handleMarkEffective(amendment.id, amendment.amendment_no)}
                                                        disabled={markEffectiveMutation.isPending}
                                                    >
                                                        {markEffectiveMutation.isPending ? (
                                                            <Loader2 className="h-4 w-4 animate-spin" />
                                                        ) : (
                                                            <>
                                                                <CheckCircle2 className="mr-1 h-4 w-4" />
                                                                {t('protocols.amendments.markEffective', { defaultValue: '標記生效' })}
                                                            </>
                                                        )}
                                                    </Button>
                                                )}
                                            <Button variant="ghost" size="sm" asChild>
                                                <Link to={`/protocols/amendments/${amendment.id}`}>
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
                <DialogContent>
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

                        {/* R71-12：結構化變更明細 — 變更目的 + 多列 項次/改動前/改動後 */}
                        <StructuredChangeEditor
                            purpose={changePurpose}
                            onPurposeChange={setChangePurpose}
                            rows={changeDetails}
                            onAddRow={addChangeDetail}
                            onRemoveRow={removeChangeDetail}
                            onUpdateRow={updateChangeDetail}
                        />
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

            {/* P6：補登歷史變更對話框（匯入計劃 + SD/管理者） */}
            {canBackfill && (
                <HistoricalAmendmentDialog
                    protocolId={protocolId}
                    open={dialogs.isOpen('historical')}
                    onOpenChange={dialogs.setOpen('historical')}
                    onSuccess={() =>
                        queryClient.invalidateQueries({ queryKey: ['protocol-amendments', protocolId] })
                    }
                />
            )}

            {/* R30-25c follow-up：標記生效確認對話框（取代 window.confirm） */}
            <ConfirmDialog state={confirmState} />
        </Card>
    )
}
