import { useQuery } from '@tanstack/react-query'
import api, { PurchaseLinesReport } from '@/lib/api'
import { formatNumber, formatDate } from '@/lib/utils'
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
import { Loader2, Download, Truck } from 'lucide-react'

export function PurchaseLinesReportPage() {
  const { data: report, isLoading } = useQuery<PurchaseLinesReport[]>({
    queryKey: ['report-purchase-lines'],
    queryFn: async () => {
      const response = await api.get<PurchaseLinesReport[]>('/reports/purchase-lines')
      return response.data
    },
  })

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'draft':
        return <Badge variant="secondary">草稿</Badge>
      case 'submitted':
        return <Badge variant="warning">待核准</Badge>
      case 'approved':
        return <Badge variant="success">已核准</Badge>
      case 'cancelled':
        return <Badge variant="destructive">已作廢</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  const exportToCSV = () => {
    if (!report) return

    const headers = ['單據日期', '單據編號', '狀態', '供應商代碼', '供應商名稱', '倉庫', '產品代碼', '產品名稱', '數量', '單位', '單價', '金額', '建立者', '核准者']
    const rows = report.map(r => [
      r.doc_date,
      r.doc_no,
      r.status,
      r.partner_code || '',
      r.partner_name || '',
      r.warehouse_name || '',
      r.product_sku,
      r.product_name,
      r.qty,
      r.uom,
      r.unit_price || '',
      r.line_total || '',
      r.created_by_name,
      r.approved_by_name || '',
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${cell}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `purchase_lines_${new Date().toISOString().split('T')[0]}.csv`
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
          <h1 className="text-3xl font-bold tracking-tight">採購明細報表</h1>
          <p className="text-muted-foreground">採購單、採購入庫、採購退貨明細</p>
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
              <TableHead>單據日期</TableHead>
              <TableHead>單據編號</TableHead>
              <TableHead>狀態</TableHead>
              <TableHead>供應商</TableHead>
              <TableHead>倉庫</TableHead>
              <TableHead>產品</TableHead>
              <TableHead className="text-right">數量</TableHead>
              <TableHead className="text-right">單價</TableHead>
              <TableHead className="text-right">金額</TableHead>
              <TableHead>建立者</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {report && report.length > 0 ? (
              report.map((row, idx) => (
                <TableRow key={idx}>
                  <TableCell>{formatDate(row.doc_date)}</TableCell>
                  <TableCell className="font-mono text-sm">{row.doc_no}</TableCell>
                  <TableCell>{getStatusBadge(row.status)}</TableCell>
                  <TableCell>
                    {row.partner_name ? (
                      <div>
                        <div className="font-medium">{row.partner_name}</div>
                        <div className="text-xs text-muted-foreground">{row.partner_code}</div>
                      </div>
                    ) : '-'}
                  </TableCell>
                  <TableCell>{row.warehouse_name || '-'}</TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{row.product_name}</div>
                      <div className="text-xs text-muted-foreground">{row.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    {formatNumber(row.qty, 0)} {row.uom}
                  </TableCell>
                  <TableCell className="text-right">
                    {row.unit_price ? `$${formatNumber(row.unit_price, 2)}` : '-'}
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {row.line_total ? `$${formatNumber(row.line_total, 2)}` : '-'}
                  </TableCell>
                  <TableCell>{row.created_by_name}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={10} className="text-center py-8">
                  <Truck className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無採購資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
