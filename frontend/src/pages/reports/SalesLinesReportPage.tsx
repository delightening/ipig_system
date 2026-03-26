import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import api, { SalesLinesReport } from '@/lib/api'
import { formatNumber, formatDate } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { Partner } from '@/types/erp'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Badge } from '@/components/ui/badge'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2, Download, ShoppingCart } from 'lucide-react'

// 客戶分類對照表
const CUSTOMER_CATEGORY_MAP: Record<string, string> = {
  internal: '內部單位',
  external: '外部客戶',
  research: '研究計畫',
  other: '其他',
}

// 格式化客戶分類
const formatCustomerCategory = (cat?: string): string => {
  if (!cat) return '-'
  return CUSTOMER_CATEGORY_MAP[cat] || cat
}

export function SalesLinesReportPage() {
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [partnerId, setPartnerId] = useState<string>('all')
  const { from, to, setFrom, setTo } = useDateRangeFilter()

  // 取得客戶清單
  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners-customer-list'],
    queryFn: async () => {
      const res = await api.get<Partner[]>('/partners?partner_type=customer')
      return res.data
    },
  })

  const { data: report, isLoading } = useQuery<SalesLinesReport[]>({
    queryKey: ['report-sales-lines', categoryFilter, from, to, partnerId],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (categoryFilter && categoryFilter !== 'all') params.set('customer_category', categoryFilter)
      if (from) params.set('date_from', from)
      if (to) params.set('date_to', to)
      if (partnerId && partnerId !== 'all') params.set('partner_id', partnerId)
      const qs = params.toString()
      const response = await api.get<SalesLinesReport[]>(`/reports/sales-lines${qs ? '?' + qs : ''}`)
      return response.data
    },
  })

  // 合計列
  const summary = useMemo(() => {
    if (!report || report.length === 0) return null
    const totalQty = report.reduce((sum, r) => sum + Number(r.qty || 0), 0)
    const totalAmount = report.reduce((sum, r) => sum + Number(r.line_total || 0), 0)
    return { totalQty, totalAmount }
  }, [report])

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

    const headers = ['單據日期', '單據編號', '狀態', '客戶代碼', '客戶名稱', '客戶分類', '倉庫', '產品代碼', '產品名稱', '數量', '單位', '單價', '金額', '建立者', '核准者']
    const rows = report.map(r => [
      r.doc_date,
      r.doc_no,
      r.status,
      r.partner_code || '',
      r.partner_name || '',
      formatCustomerCategory(r.customer_category),
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
    link.download = `sales_lines_${new Date().toISOString().split('T')[0]}.csv`
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
      <PageHeader
        title="銷貨明細報表"
        description="銷貨單、銷貨出庫明細"
        actions={
          <Button size="sm" onClick={exportToCSV} disabled={!report?.length}>
            <Download className="mr-2 h-4 w-4" />
            匯出 CSV
          </Button>
        }
      />

      {/* 篩選列 */}
      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1">
          <Label>起始日期</Label>
          <Input type="date" value={from} onChange={e => setFrom(e.target.value)} className="w-40" aria-label="起始日期" />
        </div>
        <div className="space-y-1">
          <Label>結束日期</Label>
          <Input type="date" value={to} onChange={e => setTo(e.target.value)} className="w-40" aria-label="結束日期" />
        </div>
        <div className="space-y-1">
          <Label>客戶</Label>
          <Select value={partnerId} onValueChange={setPartnerId}>
            <SelectTrigger className="w-48">
              <SelectValue placeholder="全部客戶" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部客戶</SelectItem>
              {partners?.map(p => (
                <SelectItem key={p.id} value={p.id}>
                  {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>客戶分類</Label>
          <Select value={categoryFilter} onValueChange={setCategoryFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="全部分類" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部分類</SelectItem>
              <SelectItem value="internal">內部單位</SelectItem>
              <SelectItem value="external">外部客戶</SelectItem>
              <SelectItem value="research">研究計畫</SelectItem>
              <SelectItem value="other">其他</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>單據日期</TableHead>
              <TableHead>單據編號</TableHead>
              <TableHead>狀態</TableHead>
              <TableHead>客戶</TableHead>
              <TableHead>客戶分類</TableHead>
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
              <>
                {report.map((row, idx) => (
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
                    <TableCell>
                      {row.customer_category ? (
                        <Badge variant="outline">{formatCustomerCategory(row.customer_category)}</Badge>
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
                ))}
                {summary && (
                  <TableRow className="bg-muted/50 font-semibold">
                    <TableCell colSpan={7} className="text-right">合計</TableCell>
                    <TableCell className="text-right">{formatNumber(summary.totalQty, 0)}</TableCell>
                    <TableCell />
                    <TableCell className="text-right">${formatNumber(summary.totalAmount, 2)}</TableCell>
                    <TableCell />
                  </TableRow>
                )}
              </>
            ) : (
              <TableEmptyRow colSpan={11} icon={ShoppingCart} title="尚無銷貨資料" />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
