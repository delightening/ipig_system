import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listControlledDocuments,
  createControlledDocument,
  approveControlledDocument,
  acknowledgeDocument,
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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Plus, FileCheck, CheckCircle, Inbox } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const DOC_TYPES = [
  { value: 'quality_manual', labelKey: 'adminGlp.documentControl.docType.qualityManual' },
  { value: 'sop', labelKey: 'adminGlp.documentControl.docType.sop' },
  { value: 'form', labelKey: 'adminGlp.documentControl.docType.form' },
  { value: 'external', labelKey: 'adminGlp.documentControl.docType.external' },
  { value: 'policy', labelKey: 'adminGlp.documentControl.docType.policy' },
  { value: 'report', labelKey: 'adminGlp.documentControl.docType.report' },
]

const STATUS_VARIANTS: Record<string, 'default' | 'secondary' | 'destructive' | 'success' | 'warning'> = {
  draft: 'secondary',
  under_review: 'warning',
  approved: 'success',
  active: 'success',
  obsolete: 'destructive',
}

const STATUS_LABEL_KEYS: Record<string, string> = {
  draft: 'adminGlp.shared.statusLabel.draft',
  under_review: 'adminGlp.shared.statusLabel.underReview',
  approved: 'adminGlp.shared.statusLabel.approved',
  active: 'adminGlp.shared.statusLabel.active',
  obsolete: 'adminGlp.documentControl.status.obsolete',
}

export function DocumentControlPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('dms.document.manage')
  const canApprove = hasPermission('dms.document.approve')

  const [filterType, setFilterType] = useState<string>('')
  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState({ title: '', doc_type: 'sop', category: '', retention_years: '' })

  const { data: documents = [], isLoading } = useQuery({
    queryKey: ['controlled-documents', filterType, filterStatus],
    queryFn: () =>
      listControlledDocuments({
        doc_type: filterType || undefined,
        status: filterStatus || undefined,
      }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createControlledDocument({
        title: form.title,
        doc_type: form.doc_type,
        category: form.category || undefined,
        retention_years: form.retention_years ? Number(form.retention_years) : undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['controlled-documents'] })
      setShowCreate(false)
      setForm({ title: '', doc_type: 'sop', category: '', retention_years: '' })
      toast({ title: t('adminGlp.documentControl.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) => approveControlledDocument(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['controlled-documents'] })
      toast({ title: t('adminGlp.documentControl.toast.approved') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.approveFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const acknowledgeMutation = useMutation({
    mutationFn: (id: string) => acknowledgeDocument(id),
    onSuccess: () => toast({ title: t('adminGlp.documentControl.toast.acknowledged') }),
    onError: (err: unknown) => toast({ title: t('adminGlp.documentControl.toast.acknowledgeFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.documentControl.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.documentControl.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.documentControl.create')}
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
                {DOC_TYPES.map((dt) => (
                  <SelectItem key={dt.value} value={dt.value}>{t(dt.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.shared.allStatuses')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.shared.allStatuses')}</SelectItem>
                <SelectItem value="draft">{t('adminGlp.shared.statusLabel.draft')}</SelectItem>
                <SelectItem value="under_review">{t('adminGlp.shared.statusLabel.underReview')}</SelectItem>
                <SelectItem value="approved">{t('adminGlp.shared.statusLabel.approved')}</SelectItem>
                <SelectItem value="active">{t('adminGlp.shared.statusLabel.active')}</SelectItem>
                <SelectItem value="obsolete">{t('adminGlp.shared.statusLabel.obsolete')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('adminGlp.shared.documentNumber')}</TableHead>
                <TableHead>{t('adminGlp.shared.title')}</TableHead>
                <TableHead>{t('adminGlp.shared.type')}</TableHead>
                <TableHead>{t('adminGlp.shared.version')}</TableHead>
                <TableHead>{t('adminGlp.shared.status')}</TableHead>
                <TableHead>{t('adminGlp.shared.owner')}</TableHead>
                <TableHead>{t('common.actions')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={8} cols={7} /></TableCell></TableRow>
              ) : documents.length === 0 ? (
                <TableEmptyRow colSpan={7} icon={Inbox} title={t('adminGlp.documentControl.empty')} />
              ) : (
                documents.map((doc) => {
                  const docType = DOC_TYPES.find((d) => d.value === doc.doc_type)
                  return (
                    <TableRow key={doc.id}>
                      <TableCell className="font-mono text-sm">{doc.doc_number}</TableCell>
                      <TableCell className="font-medium">{doc.title}</TableCell>
                      <TableCell>{docType ? t(docType.labelKey) : doc.doc_type}</TableCell>
                      <TableCell>v{doc.current_version}</TableCell>
                      <TableCell>
                        <Badge variant={STATUS_VARIANTS[doc.status] ?? 'secondary'}>
                          {STATUS_LABEL_KEYS[doc.status] ? t(STATUS_LABEL_KEYS[doc.status]) : doc.status}
                        </Badge>
                      </TableCell>
                      <TableCell>{doc.owner_name ?? '-'}</TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          {canApprove && (doc.status === 'draft' || doc.status === 'under_review') && (
                            <Button variant="outline" size="sm" onClick={() => approveMutation.mutate(doc.id)}>
                              <FileCheck className="h-3 w-3 mr-1" />{t('adminGlp.shared.approve')}
                            </Button>
                          )}
                          <Button variant="ghost" size="sm" onClick={() => acknowledgeMutation.mutate(doc.id)}>
                            <CheckCircle className="h-3 w-3 mr-1" />{t('adminGlp.documentControl.acknowledge')}
                          </Button>
                        </div>
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
          <DialogHeader><DialogTitle>{t('adminGlp.documentControl.dialog.title')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.titleRequired')}</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.documentControl.dialog.docTypeRequired')}</label>
              <Select value={form.doc_type} onValueChange={(v) => setForm((f) => ({ ...f, doc_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {DOC_TYPES.map((dt) => (
                    <SelectItem key={dt.value} value={dt.value}>{t(dt.labelKey)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.documentControl.dialog.category')}</label>
              <Input value={form.category} onChange={(e) => setForm((f) => ({ ...f, category: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.documentControl.dialog.retentionYears')}</label>
              <Input type="number" value={form.retention_years} onChange={(e) => setForm((f) => ({ ...f, retention_years: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
