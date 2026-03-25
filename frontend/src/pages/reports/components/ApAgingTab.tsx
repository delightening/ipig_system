import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import { apPaymentSchema, type ApPaymentFormData } from '@/lib/validation'
import { Button } from '@/components/ui/button'
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
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import type { ApAgingRow, Partner } from '@/types/accounting'

function CreateApPaymentDialog({
  asOfDate,
  onSuccess,
}: {
  asOfDate: string
  onSuccess: () => void
}) {
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
    resolver: zodResolver(apPaymentSchema),
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
      toast({ title: '付款已建立' })
      setOpen(false)
      queryClient.invalidateQueries({ queryKey: ['accounting-ap-aging'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-trial-balance'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-journal-entries'] })
      onSuccess()
    },
    onError: (err: unknown) => {
      const msg = (err as { response?: { data?: { detail?: string } } })?.response?.data?.detail || '建立失敗'
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
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="mr-2 h-4 w-4" />
          新增付款
        </Button>
      </DialogTrigger>
      <DialogContent>
        <form onSubmit={handleSubmit(onValid)}>
          <DialogHeader>
            <DialogTitle>應付帳款付款</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label>供應商 *</Label>
              <Select value={partnerId} onValueChange={(v) => setValue('partner_id', v, { shouldValidate: true })}>
                <SelectTrigger>
                  <SelectValue placeholder="選擇供應商" />
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
              <Label>付款日期 *</Label>
              <Input type="date" {...register('payment_date')} />
              {errors.payment_date && (
                <p className="text-sm text-destructive">{errors.payment_date.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label>金額 *</Label>
              <Input
                type="number"
                step="0.01"
                min="0"
                {...register('amount')}
                placeholder="0.00"
              />
              {errors.amount && (
                <p className="text-sm text-destructive">{errors.amount.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label>備註</Label>
              <Input {...register('reference')} placeholder="選填" aria-label="備註" />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>
              取消
            </Button>
            <Button type="submit" disabled={createPaymentMutation.isPending}>
              {createPaymentMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              建立
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
  const { data: apAging, isLoading } = useQuery<ApAgingRow[]>({
    queryKey: ['accounting-ap-aging', asOfDate],
    queryFn: async () => {
      const r = await api.get<ApAgingRow[]>('/accounting/ap-aging', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  if (isLoading) {
    return (
      <div className="flex justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-end gap-4 flex-wrap">
        <div className="space-y-2">
          <Label>截至日期</Label>
          <Input
            type="date"
            value={asOfDate}
            onChange={(e) => onAsOfDateChange(e.target.value)}
            className="w-40"
          />
        </div>
        <CreateApPaymentDialog asOfDate={asOfDate} onSuccess={() => {}} />
      </div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>供應商代碼</TableHead>
              <TableHead>供應商名稱</TableHead>
              <TableHead className="text-right">應付總額</TableHead>
              <TableHead className="text-right">已付總額</TableHead>
              <TableHead className="text-right">餘額</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {apAging && apAging.length > 0 ? (
              apAging.map((r) => (
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
              <TableEmptyRow colSpan={5} icon={FileText} title="尚無應付帳款餘額" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
