import { useQuery } from '@tanstack/react-query'
import api, { StockLedgerReport } from '@/lib/api'
import { formatNumber, formatDateTime } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, Download, TrendingUp } from 'lucide-react'

export function StockLedgerReportPage() {
  const { data: report, isLoading } = useQuery<StockLedgerReport[]>({
    queryKey: ['report-stock-ledger'],
    queryFn: async () => {
      const response = await api.get<StockLedgerReport[]>('/reports/stock-ledger')
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

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">庫存流水報表</h1>
          <p className="text-muted-foreground">所有庫存異動記錄</p>
        </div>
        <Button onClick={exportToCSV} disabled={!report?.length}>
          <Download className="mr-2 h-4 w-4" />
          匯出 CSV
        </Button>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>交易時間</TableHead>
              <TableHead>倉庫</TableHead>
              <TableHead>產品</TableHead>
              <TableHead>單據類型</TableHead>
              <TableHead>單據編號</TableHead>
              <TableHead>方向</TableHead>
              <TableHead className="text-right">數量</TableHead>
              <TableHead className="text-right">單位成本</TableHead>
              <TableHead>批號</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {report && report.length > 0 ? (
              report.map((row, idx) => (
                <TableRow key={idx}>
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
              <TableRow>
                <TableCell colSpan={9} className="text-center py-8">
                  <TrendingUp className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無流水資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
