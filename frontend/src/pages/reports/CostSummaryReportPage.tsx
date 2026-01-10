import { useQuery } from '@tanstack/react-query'
import api, { CostSummaryReport } from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, Download, DollarSign } from 'lucide-react'

export function CostSummaryReportPage() {
  const { data: report, isLoading } = useQuery<CostSummaryReport[]>({
    queryKey: ['report-cost-summary'],
    queryFn: async () => {
      const response = await api.get<CostSummaryReport[]>('/reports/cost-summary')
      return response.data
    },
  })

  const totalValue = report?.reduce((sum, r) => sum + parseFloat(r.total_value || '0'), 0) || 0
  const totalQty = report?.reduce((sum, r) => sum + parseFloat(r.qty_on_hand || '0'), 0) || 0

  const exportToCSV = () => {
    if (!report) return

    const headers = ['倉庫代碼', '倉庫名稱', '產品代碼', '產品名稱', '類別', '庫存量', '平均成本', '庫存價值']
    const rows = report.map(r => [
      r.warehouse_code,
      r.warehouse_name,
      r.product_sku,
      r.product_name,
      r.category_name || '',
      r.qty_on_hand,
      r.avg_cost || '',
      r.total_value || '',
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `cost_summary_${new Date().toISOString().split('T')[0]}.csv`
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
          <h1 className="text-3xl font-bold tracking-tight">成本摘要報表</h1>
          <p className="text-muted-foreground">庫存成本與價值摘要</p>
        </div>
        <Button onClick={exportToCSV} disabled={!report?.length}>
          <Download className="mr-2 h-4 w-4" />
          匯出 CSV
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">總庫存價值</CardTitle>
            <DollarSign className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${formatNumber(totalValue, 2)}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">總庫存量</CardTitle>
            <DollarSign className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(totalQty, 0)}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">品項數</CardTitle>
            <DollarSign className="h-4 w-4 text-purple-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{report?.length || 0}</div>
          </CardContent>
        </Card>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>倉庫</TableHead>
              <TableHead>產品代碼</TableHead>
              <TableHead>產品名稱</TableHead>
              <TableHead>類別</TableHead>
              <TableHead className="text-right">庫存量</TableHead>
              <TableHead className="text-right">平均成本</TableHead>
              <TableHead className="text-right">庫存價值</TableHead>
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
                  <TableCell className="text-right font-medium">
                    {formatNumber(row.qty_on_hand, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {row.avg_cost ? `$${formatNumber(row.avg_cost, 2)}` : '-'}
                  </TableCell>
                  <TableCell className="text-right font-medium text-emerald-600">
                    {row.total_value ? `$${formatNumber(row.total_value, 2)}` : '-'}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-8">
                  <DollarSign className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無成本資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
