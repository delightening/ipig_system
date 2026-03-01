import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Loader2, Download, Calculator, FileText, Receipt, CreditCard, Plus } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'

// Types
interface TrialBalanceRow {
  account_id: string
  account_code: string
  account_name: string
  account_type: string
  debit_balance: string
  credit_balance: string
}

interface JournalEntryLine {
  line_id: string
  line_no: number
  account_code: string
  account_name: string
  debit_amount: string
  credit_amount: string
  description: string | null
}

interface JournalEntry {
  id: string
  entry_no: string
  entry_date: string
  description: string | null
  source_entity_type: string | null
  source_entity_id: string | null
}

interface JournalEntryResponse {
  entry: JournalEntry
  lines: JournalEntryLine[]
}

interface ApAgingRow {
  partner_id: string
  partner_code: string
  partner_name: string
  total_payable: string
  total_paid: string
  balance: string
}

interface ArAgingRow {
  partner_id: string
  partner_code: string
  partner_name: string
  total_receivable: string
  total_received: string
  balance: string
}

interface Partner {
  id: string
  code: string
  name: string
  partner_type: 'supplier' | 'customer'
}

function formatDate(d: string) {
  return new Date(d).toLocaleDateString('zh-TW')
}

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
  const [submitting, setSubmitting] = useState(false)
  const queryClient = useQueryClient()

  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners', 'supplier'],
    queryFn: async () => {
      const r = await api.get<Partner[]>('/partners', { params: { partner_type: 'supplier' } })
      return r.data
    },
    enabled: open,
  })

  const handleSubmit = async (e: React.FormEvent) => {
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
    setSubmitting(true)
    try {
      await api.post('/accounting/ap-payments', {
        partner_id: partnerId,
        payment_date: paymentDate,
        amount: amt,
        reference: reference || undefined,
      })
      toast({ title: '付款已建立' })
      setOpen(false)
      setPartnerId('')
      setAmount('')
      setReference('')
      queryClient.invalidateQueries({ queryKey: ['accounting-ap-aging'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-trial-balance'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-journal-entries'] })
      onSuccess()
    } catch (err: unknown) {
      const msg = (err as { response?: { data?: { detail?: string } } })?.response?.data?.detail || '建立失敗'
      toast({ title: msg, variant: 'destructive' })
    } finally {
      setSubmitting(false)
    }
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
            <Button type="submit" disabled={submitting}>
              {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              建立
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

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
  const [submitting, setSubmitting] = useState(false)
  const queryClient = useQueryClient()

  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners', 'customer'],
    queryFn: async () => {
      const r = await api.get<Partner[]>('/partners', { params: { partner_type: 'customer' } })
      return r.data
    },
    enabled: open,
  })

  const handleSubmit = async (e: React.FormEvent) => {
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
    setSubmitting(true)
    try {
      await api.post('/accounting/ar-receipts', {
        partner_id: partnerId,
        receipt_date: receiptDate,
        amount: amt,
        reference: reference || undefined,
      })
      toast({ title: '收款已建立' })
      setOpen(false)
      setPartnerId('')
      setAmount('')
      setReference('')
      queryClient.invalidateQueries({ queryKey: ['accounting-ar-aging'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-trial-balance'] })
      queryClient.invalidateQueries({ queryKey: ['accounting-journal-entries'] })
      onSuccess()
    } catch (err: unknown) {
      const msg = (err as { response?: { data?: { detail?: string } } })?.response?.data?.detail || '建立失敗'
      toast({ title: msg, variant: 'destructive' })
    } finally {
      setSubmitting(false)
    }
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
            <Button type="submit" disabled={submitting}>
              {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              建立
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function AccountingReportPage() {
  const today = new Date().toISOString().slice(0, 10)
  const [asOfDate, setAsOfDate] = useState(today)
  const [dateFrom, setDateFrom] = useState(today.slice(0, 7) + '-01')
  const [dateTo, setDateTo] = useState(today)

  const { data: trialBalance, isLoading: tbLoading } = useQuery<TrialBalanceRow[]>({
    queryKey: ['accounting-trial-balance', asOfDate],
    queryFn: async () => {
      const r = await api.get<TrialBalanceRow[]>('/accounting/trial-balance', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  const { data: journalEntries, isLoading: jeLoading } = useQuery<JournalEntryResponse[]>({
    queryKey: ['accounting-journal-entries', dateFrom, dateTo],
    queryFn: async () => {
      const r = await api.get<JournalEntryResponse[]>('/accounting/journal-entries', {
        params: { date_from: dateFrom, date_to: dateTo, limit: 100 },
      })
      return r.data
    },
  })

  const { data: apAging, isLoading: apLoading } = useQuery<ApAgingRow[]>({
    queryKey: ['accounting-ap-aging', asOfDate],
    queryFn: async () => {
      const r = await api.get<ApAgingRow[]>('/accounting/ap-aging', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  const { data: arAging, isLoading: arLoading } = useQuery<ArAgingRow[]>({
    queryKey: ['accounting-ar-aging', asOfDate],
    queryFn: async () => {
      const r = await api.get<ArAgingRow[]>('/accounting/ar-aging', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  const exportTrialBalanceCSV = () => {
    if (!trialBalance?.length) return
    const headers = ['科目代碼', '科目名稱', '類型', '借方餘額', '貸方餘額']
    const rows = trialBalance.map((r) => [
      r.account_code,
      r.account_name,
      r.account_type,
      r.debit_balance,
      r.credit_balance,
    ])
    const csv = [headers, ...rows].map((row) => row.map((c) => `"${c}"`).join(',')).join('\n')
    const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `trial_balance_${asOfDate}.csv`
    link.click()
  }

  const LoadingSpinner = () => (
    <div className="flex justify-center py-8">
      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  )

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">會計報表</h1>
          <p className="text-muted-foreground">試算表、傳票、應付／應收帳款</p>
        </div>
      </div>

      <Tabs defaultValue="trial-balance" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4 lg:w-auto lg:inline-grid">
          <TabsTrigger value="trial-balance" className="gap-2">
            <Calculator className="h-4 w-4" />
            試算表
          </TabsTrigger>
          <TabsTrigger value="journal-entries" className="gap-2">
            <FileText className="h-4 w-4" />
            傳票查詢
          </TabsTrigger>
          <TabsTrigger value="ap-aging" className="gap-2">
            <CreditCard className="h-4 w-4" />
            應付帳款
          </TabsTrigger>
          <TabsTrigger value="ar-aging" className="gap-2">
            <Receipt className="h-4 w-4" />
            應收帳款
          </TabsTrigger>
        </TabsList>

        <TabsContent value="trial-balance" className="space-y-4">
          <div className="flex items-end gap-4">
            <div className="space-y-2">
              <Label>截至日期</Label>
              <Input
                type="date"
                value={asOfDate}
                onChange={(e) => setAsOfDate(e.target.value)}
                className="w-40"
              />
            </div>
            <Button onClick={exportTrialBalanceCSV} disabled={!trialBalance?.length}>
              <Download className="mr-2 h-4 w-4" />
              匯出 CSV
            </Button>
          </div>
          {tbLoading ? (
            <LoadingSpinner />
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>科目代碼</TableHead>
                    <TableHead>科目名稱</TableHead>
                    <TableHead>類型</TableHead>
                    <TableHead className="text-right">借方餘額</TableHead>
                    <TableHead className="text-right">貸方餘額</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {trialBalance && trialBalance.length > 0 ? (
                    trialBalance.map((r) => (
                      <TableRow key={r.account_id}>
                        <TableCell className="font-mono">{r.account_code}</TableCell>
                        <TableCell>{r.account_name}</TableCell>
                        <TableCell>{r.account_type}</TableCell>
                        <TableCell className="text-right">
                          {Number(r.debit_balance) > 0 ? formatNumber(Number(r.debit_balance), 2) : '-'}
                        </TableCell>
                        <TableCell className="text-right">
                          {Number(r.credit_balance) > 0 ? formatNumber(Number(r.credit_balance), 2) : '-'}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell colSpan={5} className="text-center py-8 text-muted-foreground">
                        尚無試算表資料
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          )}
        </TabsContent>

        <TabsContent value="journal-entries" className="space-y-4">
          <div className="flex items-end gap-4 flex-wrap">
            <div className="space-y-2">
              <Label>日期起</Label>
              <Input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
                className="w-40"
              />
            </div>
            <div className="space-y-2">
              <Label>日期訖</Label>
              <Input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
                className="w-40"
              />
            </div>
          </div>
          {jeLoading ? (
            <LoadingSpinner />
          ) : (
            <div className="space-y-4">
              {journalEntries && journalEntries.length > 0 ? (
                journalEntries.map(({ entry, lines }) => (
                  <div key={entry.id} className="rounded-md border p-4 space-y-3">
                    <div className="flex items-center justify-between">
                      <div>
                        <span className="font-mono font-medium">{entry.entry_no}</span>
                        <span className="mx-2 text-muted-foreground">|</span>
                        <span>{formatDate(entry.entry_date)}</span>
                        {entry.description && (
                          <span className="ml-2 text-muted-foreground">{entry.description}</span>
                        )}
                      </div>
                    </div>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-16">行號</TableHead>
                          <TableHead>科目</TableHead>
                          <TableHead>說明</TableHead>
                          <TableHead className="text-right">借方</TableHead>
                          <TableHead className="text-right">貸方</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {lines.map((l) => (
                          <TableRow key={l.line_id}>
                            <TableCell>{l.line_no}</TableCell>
                            <TableCell className="font-mono">
                              {l.account_code} {l.account_name}
                            </TableCell>
                            <TableCell>{l.description || '-'}</TableCell>
                            <TableCell className="text-right">
                              {Number(l.debit_amount) > 0 ? formatNumber(Number(l.debit_amount), 2) : '-'}
                            </TableCell>
                            <TableCell className="text-right">
                              {Number(l.credit_amount) > 0 ? formatNumber(Number(l.credit_amount), 2) : '-'}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                ))
              ) : (
                <div className="rounded-md border py-12 text-center text-muted-foreground">
                  尚無傳票資料
                </div>
              )}
            </div>
          )}
        </TabsContent>

        <TabsContent value="ap-aging" className="space-y-4">
          <div className="flex items-end gap-4 flex-wrap">
            <div className="space-y-2">
              <Label>截至日期</Label>
              <Input
                type="date"
                value={asOfDate}
                onChange={(e) => setAsOfDate(e.target.value)}
                className="w-40"
              />
            </div>
            <CreateApPaymentDialog asOfDate={asOfDate} onSuccess={() => {}} />
          </div>
          {apLoading ? (
            <LoadingSpinner />
          ) : (
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
          )}
        </TabsContent>

        <TabsContent value="ar-aging" className="space-y-4">
          <div className="flex items-end gap-4 flex-wrap">
            <div className="space-y-2">
              <Label>截至日期</Label>
              <Input
                type="date"
                value={asOfDate}
                onChange={(e) => setAsOfDate(e.target.value)}
                className="w-40"
              />
            </div>
            <CreateArReceiptDialog asOfDate={asOfDate} onSuccess={() => {}} />
          </div>
          {arLoading ? (
            <LoadingSpinner />
          ) : (
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
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}
