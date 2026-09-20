import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { animalFieldCorrectionApi } from '@/lib/api'
import { useAuthHasPermission } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
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
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2, Check, X, FileEdit, CheckCircle2 } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { toast } from '@/components/ui/use-toast'
import { useState } from 'react'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'

// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const FIELD_LABEL_KEYS: Record<string, string> = {
  ear_tag: 'adminGlp.fieldCorrections.field.earTag',
  birth_date: 'adminGlp.fieldCorrections.field.birthDate',
  gender: 'adminGlp.fieldCorrections.field.gender',
  breed: 'adminGlp.fieldCorrections.field.breed',
}

const BREED_LABEL_KEYS: Record<string, string> = {
  miniature: 'adminGlp.fieldCorrections.value.miniPig',
  minipig: 'adminGlp.fieldCorrections.value.miniPig',
  white: 'adminGlp.fieldCorrections.value.whitePig',
  other: 'adminGlp.fieldCorrections.value.other',
}

const formatValue = (field: string, value: string | null, t: TFunction): string => {
  if (!value) return '-'
  if (field === 'birth_date') {
    try {
      const d = new Date(value)
      if (!isNaN(d.getTime())) return d.toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })
    } catch {
      // ignore
    }
  }
  if (field === 'gender') {
    if (value === 'male') return t('adminGlp.fieldCorrections.value.male')
    if (value === 'female') return t('adminGlp.fieldCorrections.value.female')
  }
  if (field === 'breed') {
    if (value === 'LYD' || value === 'lyd') return 'LYD'
    const labelKey = BREED_LABEL_KEYS[value]
    return labelKey ? t(labelKey) : value
  }
  return value
}

export function AnimalFieldCorrectionsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  // R71-8：補前端權限 gate（與後端 require_permission!("animal.field_correction.review") 對齊）
  const hasPermission = useAuthHasPermission()
  const canReview = hasPermission('animal.field_correction.review')
  const { dialogState, confirm } = useConfirmDialog()
  const [rejectDialog, setRejectDialog] = useState<{ id: string; earTag: string } | null>(null)
  const [rejectReason, setRejectReason] = useState('')

  const { data: pending, isLoading } = useQuery({
    queryKey: ['animals-animal-field-corrections-pending'],
    queryFn: async () => {
      const res = await animalFieldCorrectionApi.listPending()
      return res.data
    },
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) =>
      animalFieldCorrectionApi.review(id, { approved: true }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals-animal-field-corrections-pending'] })
      // R71-1：核准已改動物身分欄位，須刷新動物列表(['animals'])與詳情(['animal', id])，
      // 否則畫面仍顯示舊耳號/品種等，與資料庫不一致。
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animal'] })
      toast({ title: t('common.success'), description: t('adminGlp.fieldCorrections.toast.approved') })
    },
    onError: (err) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(err, t('adminGlp.fieldCorrections.toast.approveFailed')),
        variant: 'destructive',
      })
    },
  })

  const rejectMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      animalFieldCorrectionApi.review(id, { approved: false, reject_reason: reason }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals-animal-field-corrections-pending'] })
      setRejectDialog(null)
      setRejectReason('')
      toast({ title: t('common.success'), description: t('adminGlp.fieldCorrections.toast.rejected') })
    },
    onError: (err) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(err, t('adminGlp.fieldCorrections.toast.rejectFailed')),
        variant: 'destructive',
      })
    },
  })

  const { sortedData: sortedPending, sort, toggleSort } = useTableSort(pending)

  // R71-10：批准會直接套用至動物識別欄位（不可逆），加二次確認。
  const handleApprove = async (id: string, earTag: string) => {
    const ok = await confirm({
      title: t('adminGlp.fieldCorrections.confirm.title'),
      description: t('adminGlp.fieldCorrections.confirm.description', { earTag: earTag || '' }),
      confirmLabel: t('adminGlp.fieldCorrections.confirm.label'),
    })
    if (ok) approveMutation.mutate(id)
  }

  // R71-10：高風險動作駁回原因改為必填。
  const handleReject = () => {
    if (!rejectDialog) return
    const reason = rejectReason.trim()
    if (!reason) return
    rejectMutation.mutate({ id: rejectDialog.id, reason })
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('adminGlp.fieldCorrections.title')}
        description={t('adminGlp.fieldCorrections.description')}
      />

      <Card className="overflow-hidden">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileEdit className="h-5 w-5" />
            {t('adminGlp.fieldCorrections.pendingTitle')}
          </CardTitle>
          <CardDescription>
            {t('adminGlp.fieldCorrections.pendingCount', { count: pending?.length ?? 0 })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <SortableTableHead sortKey="animal_ear_tag" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('adminGlp.fieldCorrections.field.earTag')}</SortableTableHead>
                <SortableTableHead sortKey="field_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('adminGlp.fieldCorrections.col.field')}</SortableTableHead>
                <TableHead>{t('adminGlp.fieldCorrections.col.originalValue')}</TableHead>
                <TableHead>{t('adminGlp.fieldCorrections.col.newValue')}</TableHead>
                <TableHead>{t('adminGlp.fieldCorrections.col.reason')}</TableHead>
                <SortableTableHead sortKey="requested_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('adminGlp.fieldCorrections.col.requester')}</SortableTableHead>
                <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('adminGlp.fieldCorrections.col.requestedAt')}</SortableTableHead>
                <TableHead className="text-right">{t('common.actions')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell></TableRow>
              ) : !pending?.length ? (
                <TableEmptyRow colSpan={8} icon={CheckCircle2} title={t('adminGlp.fieldCorrections.empty.title')} description={t('adminGlp.fieldCorrections.empty.description')} />
              ) : (
                (sortedPending ?? pending)?.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell>
                      <Link
                        to={`/animals/${r.animal_id}`}
                        className="text-primary hover:underline font-medium"
                      >
                        {r.animal_ear_tag || '-'}
                      </Link>
                    </TableCell>
                    <TableCell>{FIELD_LABEL_KEYS[r.field_name] ? t(FIELD_LABEL_KEYS[r.field_name]) : r.field_name}</TableCell>
                    <TableCell>{formatValue(r.field_name, r.old_value, t)}</TableCell>
                    <TableCell className="font-medium">{formatValue(r.field_name, r.new_value, t)}</TableCell>
                    <TableCell className="max-w-[200px] whitespace-normal break-words" title={r.reason}>
                      {r.reason}
                    </TableCell>
                    <TableCell>{r.requested_by_name || '-'}</TableCell>
                    <TableCell>{new Date(r.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                    <TableCell className="text-right">
                      {canReview ? (
                        <div className="flex justify-end gap-2">
                          <Button
                            size="sm"
                            className="bg-status-success-text hover:bg-status-success-text/90"
                            onClick={() => handleApprove(r.id, r.animal_ear_tag || '')}
                            disabled={approveMutation.isPending}
                          >
                            {approveMutation.isPending ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <>
                                <Check className="h-4 w-4 mr-1" />
                                {t('adminGlp.fieldCorrections.approve')}
                              </>
                            )}
                          </Button>
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={() => setRejectDialog({ id: r.id, earTag: r.animal_ear_tag || '' })}
                            disabled={rejectMutation.isPending}
                          >
                            <X className="h-4 w-4 mr-1" />
                            {t('adminGlp.fieldCorrections.reject')}
                          </Button>
                        </div>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog
        open={!!rejectDialog}
        onOpenChange={(open) => {
          if (!open) {
            setRejectDialog(null)
            setRejectReason('')
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('adminGlp.fieldCorrections.rejectDialog.title')}</DialogTitle>
            <DialogDescription>
              {t('adminGlp.fieldCorrections.rejectDialog.description', { earTag: rejectDialog?.earTag ?? '' })}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label>{t('adminGlp.fieldCorrections.rejectDialog.reasonLabel')}</Label>
            <Input
              value={rejectReason}
              onChange={(e) => setRejectReason(e.target.value)}
              placeholder={t('adminGlp.fieldCorrections.rejectDialog.reasonPlaceholder')}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRejectDialog(null)}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="destructive"
              onClick={handleReject}
              disabled={rejectMutation.isPending || !rejectReason.trim()}
            >
              {rejectMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('adminGlp.fieldCorrections.rejectDialog.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
