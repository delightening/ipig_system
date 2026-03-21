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
import type { ArAgingRow, Partner } from '@/types/accounting'

function CreateArReceiptDialog({
  asOfDate,
  onSuccess,
}: {
  asOfDate: string
  onSuccess: () => void
}) {
  const [open, setOpen] = useState(false)
  const [partnerId, setPartnerId] = useState('')
  const [amount, setAmount] = useState('')
  const [receiptDate, setReceiptDate] = useState(asOfDate)
  const [reference, setReference] = useState('')
  const queryClient = useQueryClient()

  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners', 'customer'],
    queryFn: async () => {
      const r = await api.get<Partner[]>('/partners', { params: { partner_type: 'customer' } })
      return r.data
    },
    enabled: open,
  })

  const createReceiptMutation = useMutation({
    mutationFn: (payload: { partner_id: string; receipt_date: string; amount: number; reference?: string }) =>
      api.post('/accounting/ar-receipts', payload),
    onSuccess: () => {
      toast({ title: '收款已建立' })
      setOpen(false)
      setPartnerId('')
      setAmount('')
      setReference('')
      queryClient.invalidateQueries({ queryKey: ['accounting-ar-aging'] })
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
    if (!partnerId || !amount || !receiptDate) {
      toast({ title: '請填寫必填欄位', variant: 'destructive' })
      return
    }
    const amt = parseFloat(amount)
    if (isNaN(amt) || amt <= 0) {
      toast({ title: '請輸入有效金額', variant: 'destructive' })
      return
    }
    createReceiptMutation.mutate({
      partner_id: partnerId,
      receipt_date: receiptDate,
      amount: amt,
      reference: reference || undefined,
    })
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="mr-2 h-4 w-4" />
          新增收款
        </Button>
      </DialogTrigger>
      <DialogContent>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>應收帳款收款</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label>客戶 *</Label>
              <Select value={partnerId} onValueChange={setPartnerId} required>
                <SelectTrigger>
                  <SelectValue placeholder="選擇客戶" />
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
              <Label>收款日期 *</Label>
              <Input
                type="date"
                value={receiptDate}
                onChange={(e) => setReceiptDate(e.target.value)}
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
            <Button type="submit" disabled={createReceiptMutation.isPending}>
              {createReceiptMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              建立
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface ArAgingTabProps {
  asOfDate: string
  onAsOfDateChange: (date: string) => void
}

export function ArAgingTab({ asOfDate, onAsOfDateChange }: ArAgingTabProps) {
  const { data: arAging, isLoading } = useQuery<ArAgingRow[]>({
    queryKey: ['accounting-ar-aging', asOfDate],
    queryFn: async () => {
      const r = await api.get<ArAgingRow[]>('/accounting/ar-aging', {
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
        <CreateArReceiptDialog asOfDate={asOfDate} onSuccess={() => {}} />
      </div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>客戶代碼</TableHead>
              <TableHead>客戶名稱</TableHead>
              <TableHead className="text-right">應收總額</TableHead>
              <TableHead className="text-right">已收總額</TableHead>
              <TableHead className="text-right">餘額</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {arAging && arAging.length > 0 ? (
              arAging.map((r) => (
                <TableRow key={r.partner_id}>
                  <TableCell className="font-mono">{r.partner_code}</TableCell>
                  <TableCell>{r.partner_name}</TableCell>
                  <TableCell className="text-right">
                    {formatNumber(Number(r.total_receivable), 2)}
                  </TableCell>
                  <TableCell className="text-right">
                    {formatNumber(Number(r.total_received), 2)}
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(Number(r.balance), 2)}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8 text-muted-foreground">
                  尚無應收帳款餘額
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
