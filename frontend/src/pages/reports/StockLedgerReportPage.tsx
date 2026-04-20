import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import api from '@/lib/api'
import { formatNumber, formatDateTime } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTableSort } from '@/hooks/useTableSort'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Badge } from '@/components/ui/badge'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Download, TrendingUp } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'

import type { StockLedgerReport } from '@/types/report'

interface Warehouse {
  id: string
  code: string
  name: string
}

function buildQueryString(from: string, to: string, warehouseId: string): string {
  const params = new URLSearchParams()
  if (from) params.set('date_from', from)
  if (to) params.set('date_to', to)
  if (warehouseId && warehouseId !== 'all') params.set('warehouse_id', warehouseId)
  const qs = params.toString()
  return qs ? `?${qs}` : ''
}

export function StockLedgerReportPage() {
  const today = new Date().toISOString().slice(0, 10)
  const { from, to, setFrom, setTo } = useDateRangeFilter({
    initialFrom: today.slice(0, 7) + '-01',
    initialTo: today,
  })
  const [warehouseId, setWarehouseId] = useState('all')

  const { data: warehouses } = useQuery<Warehouse[]>({
    queryKey: ['warehouses'],
    queryFn: async () => {
      const response = await api.get<Warehouse[]>('/warehouses')
      return response.data
    },
  })

  const { data: report, isLoading } = useQuery<StockLedgerReport[]>({
    queryKey: ['report-stock-ledger', from, to, warehouseId],
    queryFn: async () => {
      const qs = buildQueryString(from, to, warehouseId)
      const response = await api.get<StockLedgerReport[]>(`/reports/stock-ledger${qs}`)
      return response.data
    },
  })

  const getDirectionBadge = (direction: string) => {
    if (direction.includes('in') || direction.includes('adjust_in')) {
      return <Badge variant="success">入庫</Badge>
    } else if (direction.includes('out') || direction.includes('adjust_out')) {
      return <Badge variant="destructive">出庫</Badge>
    }
    return <Badge variant="outline">{direction}</Badge>
  }

  const exportToCSV = () => {
    if (!report) return

    const headers = ['交易時間', '倉庫代碼', '倉庫名稱', '產品代碼', '產品名稱', '單據類型', '單據編號', '方向', '數量', '單位成本', '批號', '效期']
    const rows = report.map(r => [
      r.trx_date,
      r.warehouse_code,
      r.warehouse_name,
      r.product_sku,
      r.product_name,
      r.doc_type,
      r.doc_no,
      r.direction,
      r.qty_base,
      r.unit_cost || '',
      r.batch_no || '',
      r.expiry_date || '',
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `stock_ledger_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
  }

  const { sortedData: sortedReport, sort, toggleSort } = useTableSort(report)

  return (
    <div className="space-y-6">
      <PageHeader
        title="庫存流水報表"
        description="所有庫存異動記錄"
        actions={
          <Button size="sm" onClick={exportToCSV} disabled={!report?.length}>
            <Download className="mr-2 h-4 w-4" />
            匯出 CSV
          </Button>
        }
      />

      {/* 篩選條件 */}
      <div className="flex items-end gap-4 flex-wrap">
        <div className="space-y-2">
          <Label>日期起</Label>
          <Input
            type="date"
            value={from}
            onChange={e => setFrom(e.target.value)}
            className="w-[160px]"
          />
        </div>
        <div className="space-y-2">
          <Label>日期訖</Label>
          <Input
            type="date"
            value={to}
            onChange={e => setTo(e.target.value)}
            className="w-[160px]"
          />
        </div>
        <div className="space-y-2">
          <Label>倉庫</Label>
          <Select value={warehouseId} onValueChange={setWarehouseId}>
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="全部倉庫" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部倉庫</SelectItem>
              {warehouses?.map(w => (
                <SelectItem key={w.id} value={w.id}>
                  {w.code} - {w.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="trx_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>交易時間</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>倉庫</SortableTableHead>
              <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>產品</SortableTableHead>
              <SortableTableHead sortKey="doc_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單據類型</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單據編號</SortableTableHead>
              <SortableTableHead sortKey="direction" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>方向</SortableTableHead>
              <SortableTableHead sortKey="qty_base" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">數量</SortableTableHead>
              <SortableTableHead sortKey="unit_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">單位成本</SortableTableHead>
              <SortableTableHead sortKey="batch_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>批號</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={9} className="p-0">
                  <TableSkeleton rows={8} cols={9} />
                </TableCell>
              </TableRow>
            ) : sortedReport && sortedReport.length > 0 ? (
              sortedReport.map((row, idx) => (
                <TableRow key={`${row.doc_no}-${idx}`}>
                  <TableCell className="text-sm">
                    {formatDateTime(row.trx_date)}
                  </TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{row.warehouse_name}</div>
                      <div className="text-xs text-muted-foreground">{row.warehouse_code}</div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{row.product_name}</div>
                      <div className="text-xs text-muted-foreground">{row.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell>{row.doc_type}</TableCell>
                  <TableCell className="font-mono text-sm">{row.doc_no}</TableCell>
                  <TableCell>{getDirectionBadge(row.direction)}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(row.qty_base, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {row.unit_cost ? `$${formatNumber(row.unit_cost, 2)}` : '-'}
                  </TableCell>
                  <TableCell>{row.batch_no || '-'}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={9} icon={TrendingUp} title="尚無流水資料" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
