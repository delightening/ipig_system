import { useState, useMemo } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { PurchaseLinesReport } from '@/lib/api'
import { formatNumber, formatDate, formatUom } from '@/lib/utils'
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
  TableFooter,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Download, Truck } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { GuestDateNotice } from '@/components/ui/guest-date-notice'
import type { Partner, Warehouse } from '@/types/erp'

const ALL_VALUE = '__all__'

export function PurchaseLinesReportPage() {
  const { t } = useTranslation()
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
        return <Badge variant="secondary">{t('reportsPages.shared.docStatus.draft')}</Badge>
      case 'submitted':
        return <Badge variant="warning">{t('reportsPages.shared.docStatus.submitted')}</Badge>
      case 'approved':
        return <Badge variant="success">{t('reportsPages.shared.docStatus.approved')}</Badge>
      case 'cancelled':
        return <Badge variant="destructive">{t('reportsPages.shared.docStatus.cancelled')}</Badge>
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

  const { sortedData: sortedReport, sort, toggleSort } = useTableSort(report)

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('reportsPages.purchaseLines.title')}
        description={t('reportsPages.purchaseLines.description')}
        actions={
          <Button size="sm" onClick={exportToCSV} disabled={!report?.length}>
            <Download className="mr-2 h-4 w-4" />
            {t('reportsPages.shared.exportCsv')}
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.startDate')}</Label>
          <Input
            type="date"
            value={from}
            onChange={e => setFrom(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.endDate')}</Label>
          <Input
            type="date"
            value={to}
            onChange={e => setTo(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.supplier')}</Label>
          <Select
            value={partnerId || ALL_VALUE}
            onValueChange={v => setPartnerId(v === ALL_VALUE ? '' : v)}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('reportsPages.shared.allSuppliers')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>{t('reportsPages.shared.allSuppliers')}</SelectItem>
              {partners?.map(p => (
                <SelectItem key={p.id} value={p.id}>
                  {p.code} - {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.warehouse')}</Label>
          <Select
            value={warehouseId || ALL_VALUE}
            onValueChange={v => setWarehouseId(v === ALL_VALUE ? '' : v)}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('reportsPages.shared.allWarehouses')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>{t('reportsPages.shared.allWarehouses')}</SelectItem>
              {warehouses?.map(w => (
                <SelectItem key={w.id} value={w.id}>
                  {w.code} - {w.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      <GuestDateNotice />

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="doc_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.docDate')}</SortableTableHead>
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.docNo')}</SortableTableHead>
              <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.status')}</SortableTableHead>
              <SortableTableHead sortKey="partner_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.supplier')}</SortableTableHead>
              <SortableTableHead sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.warehouse')}</SortableTableHead>
              <SortableTableHead sortKey="product_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.product')}</SortableTableHead>
              <SortableTableHead sortKey="qty" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.shared.quantity')}</SortableTableHead>
              <SortableTableHead sortKey="unit_price" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.shared.unitPrice')}</SortableTableHead>
              <SortableTableHead sortKey="line_total" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">{t('reportsPages.shared.amount')}</SortableTableHead>
              <SortableTableHead sortKey="created_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.createdBy')}</SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={10} className="p-0">
                  <TableSkeleton rows={8} cols={10} />
                </TableCell>
              </TableRow>
            ) : sortedReport && sortedReport.length > 0 ? (
              sortedReport.map((row, idx) => (
                <TableRow key={`${row.doc_no}-${idx}`}>
                  <TableCell>{formatDate(row.doc_date)}</TableCell>
                  <TableCell className="font-mono text-sm">
                    <Link to={`/documents/${row.doc_id}`} className="text-primary hover:underline">
                      {row.doc_no}
                    </Link>
                  </TableCell>
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
              <TableEmptyRow colSpan={10} icon={Truck} title={t('reportsPages.purchaseLines.emptyTitle')} />
            )}
          </TableBody>
          {report && report.length > 0 && (
            <TableFooter>
              <TableRow>
                <TableCell colSpan={6} className="font-bold">
                  {t('reportsPages.purchaseLines.totalWithCount', { count: report.length })}
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
