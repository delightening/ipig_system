import { useQuery } from '@tanstack/react-query'
import api, { StockOnHandReport } from '@/lib/api'
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
import { Loader2, Download, Package } from 'lucide-react'

export function StockOnHandReportPage() {
  const { data: report, isLoading } = useQuery<StockOnHandReport[]>({
    queryKey: ['report-stock-on-hand'],
    queryFn: async () => {
      const response = await api.get<StockOnHandReport[]>('/reports/stock-on-hand')
      return response.data
    },
  })

  const exportToCSV = () => {
    if (!report) return

    const headers = ['倉庫代碼', '倉庫名稱', '產品代碼', '產品名稱', '類別', '單位', '庫存量', '平均成本', '庫存價值', '安全庫存', '補貨點']
    const rows = report.map(r => [
      r.warehouse_code,
      r.warehouse_name,
      r.product_sku,
      r.product_name,
      r.category_name || '',
      r.base_uom,
      r.qty_on_hand,
      r.avg_cost || '',
      r.total_value || '',
      r.safety_stock || '',
      r.reorder_point || '',
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `stock_on_hand_${new Date().toISOString().split('T')[0]}.csv`
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
          <h1 className="text-3xl font-bold tracking-tight">庫存現況報表</h1>
          <p className="text-muted-foreground">各倉庫商品庫存量與價值</p>
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
              <TableHead>倉庫</TableHead>
              <TableHead>產品代碼</TableHead>
              <TableHead>產品名稱</TableHead>
              <TableHead>類別</TableHead>
              <TableHead>單位</TableHead>
              <TableHead className="text-right">庫存量</TableHead>
              <TableHead className="text-right">平均成本</TableHead>
              <TableHead className="text-right">庫存價值</TableHead>
              <TableHead className="text-right">安全庫存</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {report && report.length > 0 ? (
              report.map((row) => (
                <TableRow key={`${row.warehouse_id}-${row.product_id}`}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{row.warehouse_name}</div>
                      <div className="text-xs text-muted-foreground">{row.warehouse_code}</div>
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">{row.product_sku}</TableCell>
                  <TableCell>{row.product_name}</TableCell>
                  <TableCell>{row.category_name || '-'}</TableCell>
                  <TableCell>{row.base_uom}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(row.qty_on_hand, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {row.avg_cost ? `$${formatNumber(row.avg_cost, 2)}` : '-'}
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {row.total_value ? `$${formatNumber(row.total_value, 2)}` : '-'}
                  </TableCell>
                  <TableCell className="text-right">
                    {row.safety_stock ? formatNumber(row.safety_stock, 0) : '-'}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={9} className="text-center py-8">
                  <Package className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無庫存資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
