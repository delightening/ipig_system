import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2 } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { ProfitLossSummary } from '@/types/report'

interface ProfitLossTabProps {
  dateFrom: string
  dateTo: string
  onDateFromChange: (date: string) => void
  onDateToChange: (date: string) => void
}

export function ProfitLossTab({
  dateFrom,
  dateTo,
  onDateFromChange,
  onDateToChange,
}: ProfitLossTabProps) {
  const { data: profitLoss, isLoading } = useQuery<ProfitLossSummary>({
    queryKey: ['accounting-profit-loss', dateFrom, dateTo],
    queryFn: async () => {
      const r = await api.get<ProfitLossSummary>('/accounting/profit-loss', {
        params: { date_from: dateFrom, date_to: dateTo },
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
          <Label>日期起</Label>
          <Input
            type="date"
            value={dateFrom}
            onChange={(e) => onDateFromChange(e.target.value)}
            className="w-40"
          />
        </div>
        <div className="space-y-2">
          <Label>日期訖</Label>
          <Input
            type="date"
            value={dateTo}
            onChange={(e) => onDateToChange(e.target.value)}
            className="w-40"
          />
        </div>
      </div>
      {profitLoss ? (
        <div className="space-y-6">
          <div>
            <h3 className="text-lg font-semibold mb-2">收入</h3>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>科目代碼</TableHead>
                    <TableHead>科目名稱</TableHead>
                    <TableHead className="text-right">金額</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {profitLoss.rows
                    .filter((r) => r.account_type === 'revenue')
                    .map((r) => (
                      <TableRow key={r.account_code}>
                        <TableCell className="font-mono">{r.account_code}</TableCell>
                        <TableCell>{r.account_name}</TableCell>
                        <TableCell className="text-right">
                          {formatNumber(Number(r.amount), 2)}
                        </TableCell>
                      </TableRow>
                    ))}
                  <TableRow className="bg-muted/50 font-semibold">
                    <TableCell colSpan={2} className="text-right">收入合計</TableCell>
                    <TableCell className="text-right">
                      {formatNumber(Number(profitLoss.total_revenue), 2)}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
          </div>

          <div>
            <h3 className="text-lg font-semibold mb-2">費用</h3>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>科目代碼</TableHead>
                    <TableHead>科目名稱</TableHead>
                    <TableHead className="text-right">金額</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {profitLoss.rows
                    .filter((r) => r.account_type === 'expense')
                    .map((r) => (
                      <TableRow key={r.account_code}>
                        <TableCell className="font-mono">{r.account_code}</TableCell>
                        <TableCell>{r.account_name}</TableCell>
                        <TableCell className="text-right">
                          {formatNumber(Number(r.amount), 2)}
                        </TableCell>
                      </TableRow>
                    ))}
                  <TableRow className="bg-muted/50 font-semibold">
                    <TableCell colSpan={2} className="text-right">費用合計</TableCell>
                    <TableCell className="text-right">
                      {formatNumber(Number(profitLoss.total_expense), 2)}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
          </div>

          <div className="rounded-md border p-4 bg-muted/30">
            <div className="flex items-center justify-between text-lg font-bold">
              <span>淨利（損）</span>
              <span className={Number(profitLoss.net_income) >= 0 ? 'text-green-600' : 'text-red-600'}>
                ${formatNumber(Number(profitLoss.net_income), 2)}
              </span>
            </div>
          </div>
        </div>
      ) : (
        <div className="rounded-md border py-12 text-center text-muted-foreground">
          尚無損益資料
        </div>
      )}
    </div>
  )
}
