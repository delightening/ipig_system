import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listManagementReviews,
  createManagementReview,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Plus, ClipboardList } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const STATUS_OPTIONS = [
  { value: 'planned', labelKey: 'adminGlp.managementReview.status.planned' },
  { value: 'in_progress', labelKey: 'adminGlp.shared.statusLabel.inProgress' },
  { value: 'completed', labelKey: 'adminGlp.shared.statusLabel.completed' },
  { value: 'closed', labelKey: 'adminGlp.shared.statusLabel.closed' },
]

const STATUS_VARIANTS: Record<string, 'default' | 'secondary' | 'destructive' | 'success' | 'warning'> = {
  planned: 'secondary',
  in_progress: 'warning',
  completed: 'success',
  closed: 'secondary',
}

const INITIAL_FORM = { title: '', review_date: '', agenda: '' }

export function ManagementReviewPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('glp.management_review.manage')

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: reviews = [], isLoading } = useQuery({
    queryKey: ['management-reviews', filterStatus],
    queryFn: () => listManagementReviews({ status: filterStatus || undefined }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createManagementReview({
        title: form.title,
        review_date: form.review_date,
        agenda: form.agenda || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['management-reviews'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: t('adminGlp.managementReview.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.managementReview.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.managementReview.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.managementReview.create')}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.shared.allStatuses')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.shared.allStatuses')}</SelectItem>
                {STATUS_OPTIONS.map((s) => (
                  <SelectItem key={s.value} value={s.value}>{t(s.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead>{t('adminGlp.managementReview.col.reviewNumber')}</TableHead>
                <TableHead>{t('adminGlp.shared.title')}</TableHead>
                <TableHead>{t('adminGlp.managementReview.col.reviewDate')}</TableHead>
                <TableHead>{t('adminGlp.shared.status')}</TableHead>
                <TableHead>{t('adminGlp.managementReview.col.chair')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={5} className="p-0"><TableSkeleton rows={5} cols={5} /></TableCell></TableRow>
              ) : reviews.length === 0 ? (
                <TableEmptyRow colSpan={5} icon={ClipboardList} title={t('adminGlp.managementReview.empty')} />
              ) : (
                reviews.map((r) => {
                  const statusOption = STATUS_OPTIONS.find((s) => s.value === r.status)
                  return (
                    <TableRow key={r.id}>
                      <TableCell className="font-mono text-sm">{r.review_number}</TableCell>
                      <TableCell className="font-medium">{r.title}</TableCell>
                      <TableCell>{r.review_date}</TableCell>
                      <TableCell>
                        <Badge variant={STATUS_VARIANTS[r.status] ?? 'secondary'}>
                          {statusOption ? t(statusOption.labelKey) : r.status}
                        </Badge>
                      </TableCell>
                      <TableCell>{r.chaired_by ?? '-'}</TableCell>
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('adminGlp.managementReview.dialog.title')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.titleRequired')}</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.managementReview.dialog.reviewDateRequired')}</label>
              <Input type="date" value={form.review_date} onChange={(e) => setForm((f) => ({ ...f, review_date: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.managementReview.dialog.agenda')}</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.agenda}
                onChange={(e) => setForm((f) => ({ ...f, agenda: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || !form.review_date || createMutation.isPending}>
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
