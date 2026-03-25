import { useQuery } from '@tanstack/react-query'
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
import { Loader2, Download, FileText } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { TrialBalanceRow } from '@/types/accounting'

interface TrialBalanceTabProps {
  asOfDate: string
  onAsOfDateChange: (date: string) => void
}

export function TrialBalanceTab({ asOfDate, onAsOfDateChange }: TrialBalanceTabProps) {
  const { data: trialBalance, isLoading } = useQuery<TrialBalanceRow[]>({
    queryKey: ['accounting-trial-balance', asOfDate],
    queryFn: async () => {
      const r = await api.get<TrialBalanceRow[]>('/accounting/trial-balance', {
        params: { as_of_date: asOfDate },
      })
      return r.data
    },
  })

  const exportCSV = () => {
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

  if (isLoading) {
    return (
      <div className="flex justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-end gap-4">
        <div className="space-y-2">
          <Label>截至日期</Label>
          <Input
            type="date"
            value={asOfDate}
            onChange={(e) => onAsOfDateChange(e.target.value)}
            className="w-40"
          />
        </div>
        <Button onClick={exportCSV} disabled={!trialBalance?.length}>
          <Download className="mr-2 h-4 w-4" />
          匯出 CSV
        </Button>
      </div>
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
              <TableEmptyRow colSpan={5} icon={FileText} title="尚無試算表資料" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
