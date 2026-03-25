import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import api, { PurchaseLinesReport } from '@/lib/api'
import { formatNumber, formatDate, formatUom } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
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
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, Download, Truck } from 'lucide-react'
import type { Partner, Warehouse } from '@/types/erp'

const ALL_VALUE = '__all__'

export function PurchaseLinesReportPage() {
  const { from, to, setFrom, setTo } = useDateRangeFilter()
  const [partnerId, setPartnerId] = useState('')
  const [warehouseId, setWarehouseId] = useState('')

  const { data: partners } = useQuery<Partner[]>({
    queryKey: ['partners-supplier'],
    queryFn: async () => {
      const res = await api.get<Partner[]>('/partners?partner_type=supplier')
      return res.data
    },
  })

  const { data: warehouses } = useQuery<Warehouse[]>({
    queryKey: ['warehouses'],
    queryFn: async () => {
      const res = await api.get<Warehouse[]>('/warehouses')
      return res.data
    },
  })

  const { data: report, isLoading } = useQuery<PurchaseLinesReport[]>({
    queryKey: ['report-purchase-lines', from, to, partnerId, warehouseId],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (from) params.set('date_from', from)
      if (to) params.set('date_to', to)
      if (partnerId) params.set('partner_id', partnerId)
      if (warehouseId) params.set('warehouse_id', warehouseId)
      const qs = params.toString()
      const response = await api.get<PurchaseLinesReport[]>(
        `/reports/purchase-lines${qs ? '?' + qs : ''}`
      )
      return response.data
    },
  })

  const totals = useMemo(() => {
    if (!report?.length) return { qty: 0, amount: 0 }
    return report.reduce(
      (acc, row) => ({
        qty: acc.qty + Number(row.qty || 0),
        amount: acc.amount + Number(row.line_total || 0),
      }),
      { qty: 0, amount: 0 }
    )
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

    const headers = [
      '單據日期', '單據編號', '狀態', '供應商代碼', '供應商名稱',
      '倉庫', '產品代碼', '產品名稱', '數量', '單位',
      '單價', '金額', '建立者', '核准者',
    ]
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
      formatUom(r.uom),
      r.unit_price || '',
      r.line_total || '',
      r.created_by_name,
      r.approved_by_name || '',
    ])

    const filterInfo = [
      from ? `起始日期: ${from}` : '',
      to ? `結束日期: ${to}` : '',
    ].filter(Boolean).join(' | ')

    const csvContent = [
      ...(filterInfo ? [[filterInfo]] : []),
      headers,
      ...rows,
    ]
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
      <PageHeader
        title="採購明細報表"
        description="採購單、採購入庫、採購退貨明細"
        actions={
          <Button onClick={exportToCSV} disabled={!report?.length}>
            <Download className="mr-2 h-4 w-4" />
            匯出 CSV
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <div className="space-y-1">
          <Label>起始日期</Label>
          <Input
            type="date"
            value={from}
            onChange={e => setFrom(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label>結束日期</Label>
          <Input
            type="date"
            value={to}
            onChange={e => setTo(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label>供應商</Label>
          <Select
            value={partnerId || ALL_VALUE}
            onValueChange={v => setPartnerId(v === ALL_VALUE ? '' : v)}
          >
            <SelectTrigger>
              <SelectValue placeholder="全部供應商" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>全部供應商</SelectItem>
              {partners?.map(p => (
                <SelectItem key={p.id} value={p.id}>
                  {p.code} - {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>倉庫</Label>
          <Select
            value={warehouseId || ALL_VALUE}
            onValueChange={v => setWarehouseId(v === ALL_VALUE ? '' : v)}
          >
            <SelectTrigger>
              <SelectValue placeholder="全部倉庫" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>全部倉庫</SelectItem>
              {warehouses?.map(w => (
                <SelectItem key={w.id} value={w.id}>
                  {w.code} - {w.name}
                </SelectItem>
              ))}
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
                    {formatNumber(row.qty, 0)} {formatUom(row.uom)}
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
              <TableEmptyRow colSpan={10} icon={Truck} title="尚無採購資料" />
            )}
          </TableBody>
          {report && report.length > 0 && (
            <TableFooter>
              <TableRow>
                <TableCell colSpan={6} className="font-bold">
                  合計（{report.length} 筆）
                </TableCell>
                <TableCell className="text-right font-bold">
                  {formatNumber(totals.qty, 0)}
                </TableCell>
                <TableCell />
                <TableCell className="text-right font-bold">
                  ${formatNumber(totals.amount, 2)}
                </TableCell>
                <TableCell />
              </TableRow>
            </TableFooter>
          )}
        </Table>
      </div>
    </div>
  )
}
