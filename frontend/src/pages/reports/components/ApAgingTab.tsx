import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
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
import { Loader2, Plus } from 'lucide-react'
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
  const [partnerId, setPartnerId] = useState('')
  const [amount, setAmount] = useState('')
  const [paymentDate, setPaymentDate] = useState(asOfDate)
  const [reference, setReference] = useState('')
  const queryClient = useQueryClient()

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
      setPartnerId('')
      setAmount('')
      setReference('')
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!partnerId || !amount || !paymentDate) {
      toast({ title: '請填寫必填欄位', variant: 'destructive' })
      return
    }
    const amt = parseFloat(amount)
    if (isNaN(amt) || amt <= 0) {
      toast({ title: '請輸入有效金額', variant: 'destructive' })
      return
    }
    createPaymentMutation.mutate({
      partner_id: partnerId,
      payment_date: paymentDate,
      amount: amt,
      reference: reference || undefined,
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
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>應付帳款付款</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label>供應商 *</Label>
              <Select value={partnerId} onValueChange={setPartnerId} required>
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
            </div>
            <div className="space-y-2">
              <Label>付款日期 *</Label>
              <Input
                type="date"
                value={paymentDate}
                onChange={(e) => setPaymentDate(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label>金額 *</Label>
              <Input
                type="number"
                step="0.01"
                min="0"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                required
              />
            </div>
            <div className="space-y-2">
              <Label>備註</Label>
              <Input value={reference} onChange={(e) => setReference(e.target.value)} placeholder="選填" />
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
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8 text-muted-foreground">
                  尚無應付帳款餘額
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
