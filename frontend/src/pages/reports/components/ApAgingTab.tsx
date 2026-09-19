import { useEffect, useState } from 'react'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useForm } from 'react-hook-form'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type ApPaymentFormData = {
  partner_id: string
  payment_date: string
  amount: string
  reference: string
}
const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2, Plus, FileText } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { GuestDateNotice } from '@/components/ui/guest-date-notice'
import type { ApAgingRow, Partner } from '@/types/accounting'

function CreateApPaymentDialog({
  asOfDate,
  onSuccess,
}: {
  asOfDate: string
  onSuccess: () => void
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const queryClient = useQueryClient()

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors },
  } = useForm<ApPaymentFormData>({
    defaultValues: {
      partner_id: '',
      payment_date: asOfDate,
      amount: '',
      reference: '',
    },
  })

  const partnerId = watch('partner_id')

  useEffect(() => {
    if (open) {
      reset({
        partner_id: '',
        payment_date: asOfDate,
        amount: '',
        reference: '',
      })
    }
  }, [open, asOfDate, reset])

  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners', 'supplier'],
    queryFn: async () => {
      const r = await api.get<Partner[]>('/partners', { params: { partner_type: 'supplier' } })
      return r.data
    },
    enabled: open,
  })

  const createPaymentMutation = useMutation({
    mutationFn: (payload: { partner_id: string; payment_date: string; amount: number; reference?: string }) =>
      api.post('/accounting/ap-payments', payload),
    onSuccess: () => {
      toast({ title: t('reportsPages.accounting.apAging.created') })
      setOpen(false)
      queryClient.invalidateQueries({ queryKey: ['accounting-ap-aging'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-trial-balance'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-journal-entries'] })
      onSuccess()
    },
    onError: (err: unknown) => {
      const msg = (err as { response?: { data?: { detail?: string } } })?.response?.data?.detail || t('reportsPages.shared.createFailed')
      toast({ title: msg, variant: 'destructive' })
    },
  })

  const onValid = (data: ApPaymentFormData) => {
    createPaymentMutation.mutate({
      partner_id: data.partner_id,
      payment_date: data.payment_date,
      amount: parseFloat(data.amount),
      reference: data.reference || undefined,
    })
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {/* 後端 create_ap_payment / create_ar_receipt 皆 require_permission!("erp.document.create")；
          原本此頁完全沒有按鈕層閘。閘在 DialogTrigger 外層，連對話框都打不開。 */}
      <Can permission={PERMISSIONS.ERP_DOCUMENT_CREATE}>
        <DialogTrigger asChild>
          <Button size="sm">
            <Plus className="mr-2 h-4 w-4" />
            {t('reportsPages.accounting.apAging.addPayment')}
          </Button>
        </DialogTrigger>
      </Can>
      <DialogContent>
        <form onSubmit={handleSubmit(onValid)}>
          <DialogHeader>
            <DialogTitle>{t('reportsPages.accounting.apAging.dialogTitle')}</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label>{t('reportsPages.accounting.apAging.supplierRequired')}</Label>
              <input type="hidden" {...register('partner_id', { required: t('reportsPages.accounting.apAging.supplierError') })} />
              <Select value={partnerId} onValueChange={(v) => setValue('partner_id', v, { shouldValidate: true })}>
                <SelectTrigger>
                  <SelectValue placeholder={t('reportsPages.accounting.apAging.selectSupplier')} />
                </SelectTrigger>
                <SelectContent>
                  {partners?.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.code} - {p.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.partner_id && (
                <p className="text-sm text-destructive">{errors.partner_id.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label>{t('reportsPages.accounting.apAging.paymentDateRequired')}</Label>
              <Input
                type="date"
                {...register('payment_date', {
                  required: t('reportsPages.accounting.apAging.paymentDateError'),
                  pattern: { value: DATE_PATTERN, message: t('reportsPages.accounting.apAging.paymentDateError') },
                })}
              />
              {errors.payment_date && (
                <p className="text-sm text-destructive">{errors.payment_date.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label>{t('reportsPages.accounting.apAging.amountRequired')}</Label>
              <Input
                type="number"
                step="0.01"
                min="0"
                {...register('amount', {
                  required: t('reportsPages.accounting.amountInvalid'),
                  validate: (v) => {
                    const n = parseFloat(v)
                    return (!isNaN(n) && n > 0) || t('reportsPages.accounting.amountInvalid')
                  },
                })}
                placeholder="0.00"
              />
              {errors.amount && (
                <p className="text-sm text-destructive">{errors.amount.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label>{t('reportsPages.shared.note')}</Label>
              <Input {...register('reference')} placeholder={t('reportsPages.shared.optional')} aria-label={t('reportsPages.shared.note')} />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={createPaymentMutation.isPending}>
              {createPaymentMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('reportsPages.shared.createAction')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface ApAgingTabProps {
  asOfDate: string
  onAsOfDateChange: (date: string) => void
}

export function ApAgingTab({ asOfDate, onAsOfDateChange }: ApAgingTabProps) {
  const { t } = useTranslation()
  const { data: apAging, isLoading } = useQuery<ApAgingRow[]>({
    queryKey: ['accounting-ap-aging', asOfDate],
    queryFn: async () => {
      const r = await api.get<ApAgingRow[]>('/accounting/ap-aging', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  const { sortedData, sort, toggleSort } = useTableSort(apAging)

  return (
    <div className="space-y-4">
      <div className="flex items-end gap-4 flex-wrap">
        <div className="space-y-2">
          <Label>{t('reportsPages.shared.asOfDate')}</Label>
          <Input
            type="date"
            value={asOfDate}
            onChange={(e) => onAsOfDateChange(e.target.value)}
            className="w-40"
          />
        </div>
        <CreateApPaymentDialog asOfDate={asOfDate} onSuccess={() => {}} />
      </div>
      <GuestDateNotice />
      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="partner_code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.accounting.apAging.supplierCode')}</SortableTableHead>
              <SortableTableHead sortKey="partner_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.accounting.apAging.supplierName')}</SortableTableHead>
              <SortableTableHead sortKey="total_payable" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.accounting.apAging.totalPayable')}</SortableTableHead>
              <SortableTableHead sortKey="total_paid" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.accounting.apAging.totalPaid')}</SortableTableHead>
              <SortableTableHead sortKey="balance" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.shared.balance')}</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="p-0">
                  <TableSkeleton rows={8} cols={5} />
                </TableCell>
              </TableRow>
            ) : sortedData && sortedData.length > 0 ? (
              sortedData.map((r) => (
                <TableRow key={r.partner_id}>
                  <TableCell className="font-mono">{r.partner_code}</TableCell>
                  <TableCell>{r.partner_name}</TableCell>
                  <TableCell className="text-right">
                    {formatNumber(Number(r.total_payable), 2)}
                  </TableCell>
                  <TableCell className="text-right">
                    {formatNumber(Number(r.total_paid), 2)}
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(Number(r.balance), 2)}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={5} icon={FileText} title={t('reportsPages.accounting.apAging.emptyTitle')} />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
