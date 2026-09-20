import { useState, useMemo } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { SalesLinesReport } from '@/lib/api'
import { formatNumber, formatDate } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTableSort } from '@/hooks/useTableSort'
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
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Download, ShoppingCart } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { GuestDateNotice } from '@/components/ui/guest-date-notice'

// 客戶分類對照表（僅供 CSV 匯出使用：匯出檔案內容為正式報表產出，維持中文；畫面顯示走 i18n）
const CUSTOMER_CATEGORY_MAP: Record<string, string> = {
  internal: '內部單位',
  external: '外部客戶',
  research: '研究計畫',
  other: '其他',
}

// 格式化客戶分類（CSV 匯出用）
const formatCustomerCategory = (cat?: string): string => {
  if (!cat) return '-'
  return CUSTOMER_CATEGORY_MAP[cat] || cat
}

// 客戶分類 → 畫面顯示用 i18n 鍵（未知分類原樣顯示）
const CUSTOMER_CATEGORY_LABEL_KEYS: Record<string, string> = {
  internal: 'reportsPages.salesLines.categories.internal',
  external: 'reportsPages.salesLines.categories.external',
  research: 'reportsPages.salesLines.categories.research',
  other: 'reportsPages.salesLines.categories.other',
}

export function SalesLinesReportPage() {
  const { t } = useTranslation()
  const getCustomerCategoryLabel = (cat?: string): string => {
    if (!cat) return '-'
    const key = CUSTOMER_CATEGORY_LABEL_KEYS[cat]
    return key ? t(key) : cat
  }
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

  const { sortedData: sortedReport, sort, toggleSort } = useTableSort(report)

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('reportsPages.salesLines.title')}
        description={t('reportsPages.salesLines.description')}
        actions={
          <Button size="sm" onClick={exportToCSV} disabled={!report?.length}>
            <Download className="mr-2 h-4 w-4" />
            {t('reportsPages.shared.exportCsv')}
          </Button>
        }
      />

      {/* 篩選列 */}
      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.startDate')}</Label>
          <Input type="date" value={from} onChange={e => setFrom(e.target.value)} className="w-40" aria-label={t('reportsPages.shared.startDate')} />
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.endDate')}</Label>
          <Input type="date" value={to} onChange={e => setTo(e.target.value)} className="w-40" aria-label={t('reportsPages.shared.endDate')} />
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.customer')}</Label>
          <Select value={partnerId} onValueChange={setPartnerId}>
            <SelectTrigger className="w-48">
              <SelectValue placeholder={t('reportsPages.shared.allCustomers')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('reportsPages.shared.allCustomers')}</SelectItem>
              {partners?.map(p => (
                <SelectItem key={p.id} value={p.id}>
                  {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.salesLines.customerCategory')}</Label>
          <Select value={categoryFilter} onValueChange={setCategoryFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder={t('reportsPages.salesLines.allCategories')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('reportsPages.salesLines.allCategories')}</SelectItem>
              <SelectItem value="internal">{t('reportsPages.salesLines.categories.internal')}</SelectItem>
              <SelectItem value="external">{t('reportsPages.salesLines.categories.external')}</SelectItem>
              <SelectItem value="research">{t('reportsPages.salesLines.categories.research')}</SelectItem>
              <SelectItem value="other">{t('reportsPages.salesLines.categories.other')}</SelectItem>
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
              <SortableTableHead sortKey="partner_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.shared.customer')}</SortableTableHead>
              <SortableTableHead sortKey="customer_category" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('reportsPages.salesLines.customerCategory')}</SortableTableHead>
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
                <TableCell colSpan={11} className="p-0">
                  <TableSkeleton rows={8} cols={11} />
                </TableCell>
              </TableRow>
            ) : sortedReport && sortedReport.length > 0 ? (
              <>
                {sortedReport.map((row, idx) => (
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
                    <TableCell>
                      {row.customer_category ? (
                        <Badge variant="outline">{getCustomerCategoryLabel(row.customer_category)}</Badge>
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
                    <TableCell colSpan={7} className="text-right">{t('reportsPages.shared.total')}</TableCell>
                    <TableCell className="text-right">{formatNumber(summary.totalQty, 0)}</TableCell>
                    <TableCell />
                    <TableCell className="text-right">${formatNumber(summary.totalAmount, 2)}</TableCell>
                    <TableCell />
                  </TableRow>
                )}
              </>
            ) : (
              <TableEmptyRow colSpan={11} icon={ShoppingCart} title={t('reportsPages.salesLines.emptyTitle')} />
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
