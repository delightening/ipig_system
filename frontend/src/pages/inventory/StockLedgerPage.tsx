import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import api, { StockLedgerDetail } from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { PageHeader } from '@/components/ui/page-header'
import { FileText, Download, ExternalLink } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { formatDateTime, formatNumber, formatCurrency } from '@/lib/utils'

const directionNames: Record<string, string> = {
  in: '入庫',
  out: '出庫',
  transfer_in: '調入',
  transfer_out: '調出',
  adjust_in: '調增',
  adjust_out: '調減',
}

export function StockLedgerPage() {
  const { data: ledger, isLoading } = useQuery({
    queryKey: ['stock-ledger'],
    queryFn: async () => {
      const response = await api.get<StockLedgerDetail[]>('/inventory/ledger')
      return response.data
    },
  })

  const { sortedData: sortedLedger, sort, toggleSort } = useTableSort(ledger)

  const exportToCSV = () => {
    if (!ledger) return

    const headers = ['時間', '倉庫', '品項代碼', '品項名稱', '單據編號', '方向', '數量', '單位成本', '批號']
    const rows = ledger.map(item => [
      item.trx_date,
      item.warehouse_name,
      item.product_sku,
      item.product_name,
      item.doc_no,
      directionNames[item.direction] || item.direction,
      item.qty_base,
      item.unit_cost || '',
      item.batch_no || '',
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

  const getDirectionBadge = (direction: string) => {
    const isInbound = ['in', 'transfer_in', 'adjust_in'].includes(direction)
    return (
      <Badge variant={isInbound ? 'success' : 'destructive'}>
        {directionNames[direction] || direction}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="庫存流水"
        description="查看所有庫存異動記錄"
        actions={
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" asChild>
              <Link to="/reports/stock-ledger">
                <ExternalLink className="mr-2 h-4 w-4" />
                報表中心（進階篩選）
              </Link>
            </Button>
            <Button size="sm" onClick={exportToCSV} disabled={!ledger?.length}>
              <Download className="mr-2 h-4 w-4" />
              匯出 CSV
            </Button>
          </div>
        }
      />

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="trx_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>時間</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>倉庫</SortableTableHead>
              <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>品項</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單據</SortableTableHead>
              <SortableTableHead sortKey="direction" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>方向</SortableTableHead>
              <SortableTableHead sortKey="qty_base" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">數量</SortableTableHead>
              <SortableTableHead sortKey="unit_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">單位成本</SortableTableHead>
              <SortableTableHead sortKey="batch_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>批號</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="p-0">
                  <TableSkeleton rows={8} cols={8} />
                </TableCell>
              </TableRow>
            ) : sortedLedger && sortedLedger.length > 0 ? (
              sortedLedger.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="text-sm">{formatDateTime(item.trx_date)}</TableCell>
                  <TableCell>{item.warehouse_name}</TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{item.product_name}</div>
                      <div className="text-xs text-muted-foreground">{item.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">{item.doc_no}</TableCell>
                  <TableCell>{getDirectionBadge(item.direction)}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(item.qty_base, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.unit_cost ? formatCurrency(item.unit_cost) : '-'}
                  </TableCell>
                  <TableCell>{item.batch_no || '-'}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={8} icon={FileText} title="尚無庫存流水資料" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
