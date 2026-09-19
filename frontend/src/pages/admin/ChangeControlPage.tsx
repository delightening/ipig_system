import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import {
  listChangeRequests,
  createChangeRequest,
  approveChangeRequest,
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
import { Plus, CheckCircle, RefreshCw, Loader2 } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const CHANGE_TYPES = [
  { value: 'equipment', labelKey: 'adminGlp.shared.equipment' },
  { value: 'method', labelKey: 'adminGlp.changeControl.type.method' },
  { value: 'personnel', labelKey: 'adminGlp.changeControl.type.personnel' },
  { value: 'facility', labelKey: 'adminGlp.shared.facility' },
  { value: 'system', labelKey: 'adminGlp.changeControl.type.system' },
  { value: 'process', labelKey: 'adminGlp.changeControl.type.process' },
]

const STATUS_OPTIONS = [
  { value: 'draft', labelKey: 'adminGlp.shared.statusLabel.draft' },
  { value: 'submitted', labelKey: 'adminGlp.shared.statusLabel.submitted' },
  { value: 'under_review', labelKey: 'adminGlp.shared.statusLabel.underReview' },
  { value: 'approved', labelKey: 'adminGlp.shared.statusLabel.approved' },
  { value: 'implemented', labelKey: 'adminGlp.changeControl.status.implemented' },
  { value: 'verified', labelKey: 'adminGlp.shared.statusLabel.verified' },
  { value: 'rejected', labelKey: 'adminGlp.changeControl.status.rejected' },
]

const STATUS_VARIANTS: Record<
  string,
  'default' | 'secondary' | 'destructive' | 'success' | 'warning'
> = {
  draft: 'secondary',
  submitted: 'default',
  under_review: 'warning',
  approved: 'success',
  implemented: 'success',
  verified: 'success',
  rejected: 'destructive',
}

const INITIAL_FORM = { title: '', change_type: 'equipment', description: '', justification: '' }

export function ChangeControlPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('change.request.manage')
  const canApprove = hasPermission('change.request.approve')
  const { dialogState, confirm } = useConfirmDialog()

  // R71-10：核准變更請求送出前加二次確認。
  const handleApprove = async (id: string) => {
    const ok = await confirm({
      title: t('adminGlp.changeControl.confirm.title'),
      description: t('adminGlp.changeControl.confirm.description'),
      confirmLabel: t('adminGlp.changeControl.confirm.label'),
    })
    if (ok) approveMutation.mutate(id)
  }

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [filterType, setFilterType] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ['change-requests', filterStatus, filterType],
    queryFn: () => listChangeRequests({
      status: filterStatus || undefined,
      change_type: filterType || undefined,
    }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createChangeRequest({
        title: form.title,
        change_type: form.change_type,
        description: form.description,
        justification: form.justification || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['change-requests'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: t('adminGlp.changeControl.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) => approveChangeRequest(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['change-requests'] })
      toast({ title: t('adminGlp.changeControl.toast.approved') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.approveFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.changeControl.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.changeControl.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.changeControl.create')}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterType} onValueChange={setFilterType}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.shared.allTypes')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.shared.allTypes')}</SelectItem>
                {CHANGE_TYPES.map((ct) => (
                  <SelectItem key={ct.value} value={ct.value}>{t(ct.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
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
                <TableHead>{t('adminGlp.changeControl.col.changeNumber')}</TableHead>
                <TableHead>{t('adminGlp.shared.title')}</TableHead>
                <TableHead>{t('adminGlp.shared.type')}</TableHead>
                <TableHead>{t('adminGlp.shared.status')}</TableHead>
                <TableHead>{t('adminGlp.changeControl.col.requester')}</TableHead>
                <TableHead>{t('adminGlp.shared.createdAt')}</TableHead>
                <TableHead>{t('common.actions')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={5} cols={7} /></TableCell></TableRow>
              ) : requests.length === 0 ? (
                <TableEmptyRow colSpan={7} icon={RefreshCw} title={t('adminGlp.changeControl.empty')} />
              ) : (
                requests.map((cr) => {
                  const changeType = CHANGE_TYPES.find((ct) => ct.value === cr.change_type)
                  const statusOption = STATUS_OPTIONS.find((s) => s.value === cr.status)
                  return (
                    <TableRow key={cr.id}>
                      <TableCell className="font-mono text-sm">{cr.change_number}</TableCell>
                      <TableCell className="font-medium">{cr.title}</TableCell>
                      <TableCell>{changeType ? t(changeType.labelKey) : cr.change_type}</TableCell>
                      <TableCell>
                        <Badge variant={STATUS_VARIANTS[cr.status] ?? 'secondary'}>
                          {statusOption ? t(statusOption.labelKey) : cr.status}
                        </Badge>
                      </TableCell>
                      <TableCell>{cr.requester_name ?? '-'}</TableCell>
                      <TableCell>{new Date(cr.created_at).toLocaleDateString(uiLocale())}</TableCell>
                      <TableCell>
                        {canApprove && (cr.status === 'submitted' || cr.status === 'under_review') && (
                          <Button variant="outline" size="sm" onClick={() => handleApprove(cr.id)} disabled={approveMutation.isPending}>
                            {approveMutation.isPending && approveMutation.variables === cr.id ? (
                              <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                            ) : (
                              <CheckCircle className="h-3 w-3 mr-1" />
                            )}
                            {t('adminGlp.shared.approve')}
                          </Button>
                        )}
                      </TableCell>
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
          <DialogHeader><DialogTitle>{t('adminGlp.changeControl.create')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.titleRequired')}</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.changeControl.dialog.changeTypeRequired')}</label>
              <Select value={form.change_type} onValueChange={(v) => setForm((f) => ({ ...f, change_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CHANGE_TYPES.map((ct) => (
                    <SelectItem key={ct.value} value={ct.value}>{t(ct.labelKey)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.changeControl.dialog.descriptionRequired')}</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.description}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.changeControl.dialog.justification')}</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.justification}
                onChange={(e) => setForm((f) => ({ ...f, justification: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || !form.description || createMutation.isPending}>
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
