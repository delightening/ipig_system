import { useMemo } from 'react'
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
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { useTableSort } from '@/hooks/useTableSort'
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

  const revenueRows = useMemo(() => profitLoss?.rows.filter((r) => r.account_type === 'revenue') ?? [], [profitLoss])
  const expenseRows = useMemo(() => profitLoss?.rows.filter((r) => r.account_type === 'expense') ?? [], [profitLoss])
  const { sortedData: sortedRevenue, sort: revSort, toggleSort: toggleRevSort } = useTableSort(revenueRows)
  const { sortedData: sortedExpense, sort: expSort, toggleSort: toggleExpSort } = useTableSort(expenseRows)

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
                    <SortableTableHead sortKey="account_code" currentSort={revSort.column} currentDirection={revSort.direction} onSort={toggleRevSort}>科目代碼</SortableTableHead>
                    <SortableTableHead sortKey="account_name" currentSort={revSort.column} currentDirection={revSort.direction} onSort={toggleRevSort}>科目名稱</SortableTableHead>
                    <SortableTableHead sortKey="amount" currentSort={revSort.column} currentDirection={revSort.direction} onSort={toggleRevSort} className="text-right">金額</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedRevenue ?? revenueRows).map((r) => (
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
                    <SortableTableHead sortKey="account_code" currentSort={expSort.column} currentDirection={expSort.direction} onSort={toggleExpSort}>科目代碼</SortableTableHead>
                    <SortableTableHead sortKey="account_name" currentSort={expSort.column} currentDirection={expSort.direction} onSort={toggleExpSort}>科目名稱</SortableTableHead>
                    <SortableTableHead sortKey="amount" currentSort={expSort.column} currentDirection={expSort.direction} onSort={toggleExpSort} className="text-right">金額</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedExpense ?? expenseRows).map((r) => (
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
              <span className={Number(profitLoss.net_income) >= 0 ? 'text-status-success-text' : 'text-destructive'}>
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
