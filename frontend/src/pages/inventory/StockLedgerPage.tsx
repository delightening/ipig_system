import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import api, { StockLedgerDetail } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { PageHeader } from '@/components/ui/page-header'
import { Loader2, FileText, Download, ExternalLink } from 'lucide-react'
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
            <Button variant="outline" asChild>
              <Link to="/reports/stock-ledger">
                <ExternalLink className="mr-2 h-4 w-4" />
                報表中心（進階篩選）
              </Link>
            </Button>
            <Button onClick={exportToCSV} disabled={!ledger?.length}>
              <Download className="mr-2 h-4 w-4" />
              匯出 CSV
            </Button>
          </div>
        }
      />

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>時間</TableHead>
              <TableHead>倉庫</TableHead>
              <TableHead>品項</TableHead>
              <TableHead>單據</TableHead>
              <TableHead>方向</TableHead>
              <TableHead className="text-right">數量</TableHead>
              <TableHead className="text-right">單位成本</TableHead>
              <TableHead>批號</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : ledger && ledger.length > 0 ? (
              ledger.map((item) => (
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
