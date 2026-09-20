import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listFormulationRecords,
  createFormulationRecord,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Plus, FlaskConical } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

const INITIAL = {
  product_id: '',
  protocol_id: '',
  formulation_date: '',
  batch_number: '',
  concentration: '',
  volume: '',
  expiry_date: '',
  notes: '',
}

export function FormulationRecordsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('formulation.record.manage')

  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL)

  const { data: records = [], isLoading } = useQuery({
    queryKey: ['formulation-records'],
    queryFn: () => listFormulationRecords(),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createFormulationRecord({
        product_id: form.product_id,
        protocol_id: form.protocol_id || undefined,
        formulation_date: form.formulation_date,
        batch_number: form.batch_number || undefined,
        concentration: form.concentration || undefined,
        volume: form.volume || undefined,
        expiry_date: form.expiry_date || undefined,
        notes: form.notes || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['formulation-records'] })
      setShowCreate(false)
      setForm(INITIAL)
      toast({ title: t('adminGlp.formulationRecords.toast.created') })
    },
    onError: (err: unknown) =>
      toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const set = (k: string, v: string) => setForm((f) => ({ ...f, [k]: v }))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.formulationRecords.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.formulationRecords.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.formulationRecords.create')}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader />
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('adminGlp.formulationRecords.col.product')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.formulationDate')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.batchNumber')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.concentration')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.volume')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.preparer')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.verifier')}</TableHead>
                <TableHead>{t('adminGlp.formulationRecords.col.expiryDate')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={8} cols={8} /></TableCell></TableRow>
              ) : records.length === 0 ? (
                <TableEmptyRow colSpan={8} icon={FlaskConical} title={t('adminGlp.formulationRecords.empty')} />
              ) : (
                records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.product_name ?? r.product_id}</TableCell>
                    <TableCell>{r.formulation_date}</TableCell>
                    <TableCell className="font-mono text-sm">{r.batch_number ?? '-'}</TableCell>
                    <TableCell>{r.concentration ?? '-'}</TableCell>
                    <TableCell>{r.volume ?? '-'}</TableCell>
                    <TableCell>{r.preparer_name ?? '-'}</TableCell>
                    <TableCell>{r.verified_by ? t('adminGlp.shared.statusLabel.verified') : t('adminGlp.shared.statusLabel.pendingVerification')}</TableCell>
                    <TableCell>{r.expiry_date ?? '-'}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('adminGlp.formulationRecords.create')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.formulationRecords.dialog.productIdRequired')}</label>
              <Input value={form.product_id} onChange={(e) => set('product_id', e.target.value)} placeholder={t('adminGlp.formulationRecords.dialog.productIdPlaceholder')} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.formulationRecords.dialog.protocolId')}</label>
              <Input value={form.protocol_id} onChange={(e) => set('protocol_id', e.target.value)} placeholder={t('adminGlp.formulationRecords.dialog.optional')} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.formulationRecords.dialog.formulationDateRequired')}</label>
                <Input type="date" value={form.formulation_date} onChange={(e) => set('formulation_date', e.target.value)} />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.formulationRecords.col.batchNumber')}</label>
                <Input value={form.batch_number} onChange={(e) => set('batch_number', e.target.value)} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.formulationRecords.col.concentration')}</label>
                <Input value={form.concentration} onChange={(e) => set('concentration', e.target.value)} placeholder="e.g. 10 mg/mL" />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.formulationRecords.col.volume')}</label>
                <Input value={form.volume} onChange={(e) => set('volume', e.target.value)} placeholder="e.g. 500 mL" />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.formulationRecords.col.expiryDate')}</label>
              <Input type="date" value={form.expiry_date} onChange={(e) => set('expiry_date', e.target.value)} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.remarks')}</label>
              <Textarea value={form.notes} onChange={(e) => set('notes', e.target.value)} rows={2} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button
              onClick={() => createMutation.mutate()}
              disabled={!form.product_id || !form.formulation_date || createMutation.isPending}
            >
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
