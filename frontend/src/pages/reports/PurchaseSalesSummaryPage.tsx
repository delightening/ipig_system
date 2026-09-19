import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTableSort } from '@/hooks/useTableSort'
import type {
  PurchaseSalesMonthlySummary,
  PurchaseSalesPartnerSummary,
  PurchaseSalesCategorySummary,
} from '@/types/report'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Download, BarChart3 } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { GuestDateNotice } from '@/components/ui/guest-date-notice'

function buildQs(from: string, to: string) {
  const params = new URLSearchParams()
  if (from) params.set('date_from', from)
  if (to) params.set('date_to', to)
  const qs = params.toString()
  return qs ? `?${qs}` : ''
}

export function PurchaseSalesSummaryPage() {
  const { t } = useTranslation()
  const { from, to, setFrom, setTo } = useDateRangeFilter()

  const qs = useMemo(() => buildQs(from, to), [from, to])

  const { data: monthly, isLoading: loadingMonthly } = useQuery<PurchaseSalesMonthlySummary[]>({
    queryKey: ['report-ps-monthly', from, to],
    queryFn: async () => {
      const res = await api.get<PurchaseSalesMonthlySummary[]>(`/reports/purchase-sales-monthly${qs}`)
      return res.data
    },
  })

  const { data: byPartner, isLoading: loadingPartner } = useQuery<PurchaseSalesPartnerSummary[]>({
    queryKey: ['report-ps-partner', from, to],
    queryFn: async () => {
      const res = await api.get<PurchaseSalesPartnerSummary[]>(`/reports/purchase-sales-by-partner${qs}`)
      return res.data
    },
  })

  const { data: byCategory, isLoading: loadingCategory } = useQuery<PurchaseSalesCategorySummary[]>({
    queryKey: ['report-ps-category', from, to],
    queryFn: async () => {
      const res = await api.get<PurchaseSalesCategorySummary[]>(`/reports/purchase-sales-by-category${qs}`)
      return res.data
    },
  })

  const { sortedData: sortedMonthly, sort: sortMonthly, toggleSort: toggleMonthlySort } = useTableSort(monthly)
  const { sortedData: sortedPartner, sort: sortPartner, toggleSort: togglePartnerSort } = useTableSort(byPartner)
  const { sortedData: sortedCategory, sort: sortCategory, toggleSort: toggleCategorySort } = useTableSort(byCategory)

  const exportMonthlyCSV = () => {
    if (!monthly?.length) return
    const headers = ['月份', '採購總額', '採購退貨', '淨進貨', '銷貨總額', '銷貨退貨', '淨銷貨', '銷貨成本', '毛利']
    const rows = monthly.map(r => [
      r.year_month, r.purchase_total, r.purchase_return, r.net_purchase,
      r.sales_total, r.sales_return, r.net_sales, r.cogs_total, r.gross_profit,
    ])
    downloadCSV('purchase_sales_monthly', headers, rows)
  }

  const exportPartnerCSV = () => {
    if (!byPartner?.length) return
    const headers = ['夥伴代碼', '夥伴名稱', '類型', '總金額', '退貨金額', '淨金額', '單據數']
    const rows = byPartner.map(r => [
      r.partner_code, r.partner_name, r.partner_type === 'supplier' ? '供應商' : '客戶',
      r.total_amount, r.return_amount, r.net_amount, r.doc_count,
    ])
    downloadCSV('purchase_sales_partner', headers, rows)
  }

  const exportCategoryCSV = () => {
    if (!byCategory?.length) return
    const headers = ['產品類別', '採購金額', '銷貨金額', '銷貨成本', '毛利']
    const rows = byCategory.map(r => [
      r.category_name, r.purchase_amount, r.sales_amount, r.cogs_amount, r.gross_profit,
    ])
    downloadCSV('purchase_sales_category', headers, rows)
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('reportsPages.purchaseSales.title')}
        description={t('reportsPages.purchaseSales.description')}
      />

      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.startDate')}</Label>
          <Input type="date" value={from} onChange={e => setFrom(e.target.value)} className="w-40" aria-label={t('reportsPages.shared.startDate')} />
        </div>
        <div className="space-y-1">
          <Label>{t('reportsPages.shared.endDate')}</Label>
          <Input type="date" value={to} onChange={e => setTo(e.target.value)} className="w-40" aria-label={t('reportsPages.shared.endDate')} />
        </div>
      </div>
      <GuestDateNotice />

      <PageTabs
        tabs={[
          { value: 'monthly', label: t('reportsPages.purchaseSales.tabs.monthly') },
          { value: 'partner', label: t('reportsPages.purchaseSales.tabs.partner') },
          { value: 'category', label: t('reportsPages.purchaseSales.tabs.category') },
        ]}
        defaultTab="monthly"
      >
        <PageTabContent value="monthly" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportMonthlyCSV} disabled={!monthly?.length}>
              <Download className="mr-2 h-4 w-4" />{t('reportsPages.shared.exportCsv')}
            </Button>
          </div>
          <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead sortKey="year_month" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort}>{t('reportsPages.purchaseSales.monthly.month')}</SortableTableHead>
                    <SortableTableHead sortKey="purchase_total" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.purchaseTotal')}</SortableTableHead>
                    <SortableTableHead sortKey="purchase_return" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.purchaseReturn')}</SortableTableHead>
                    <SortableTableHead sortKey="net_purchase" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.netPurchase')}</SortableTableHead>
                    <SortableTableHead sortKey="sales_total" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.salesTotal')}</SortableTableHead>
                    <SortableTableHead sortKey="sales_return" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.salesReturn')}</SortableTableHead>
                    <SortableTableHead sortKey="net_sales" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.netSales')}</SortableTableHead>
                    <SortableTableHead sortKey="cogs_total" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.cogs')}</SortableTableHead>
                    <SortableTableHead sortKey="gross_profit" currentSort={sortMonthly.column} currentDirection={sortMonthly.direction} onSort={toggleMonthlySort} className="text-right">{t('reportsPages.purchaseSales.monthly.grossProfit')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loadingMonthly ? (
                    <TableRow>
                      <TableCell colSpan={9} className="p-0">
                        <TableSkeleton rows={8} cols={9} />
                      </TableCell>
                    </TableRow>
                  ) : sortedMonthly && sortedMonthly.length > 0 ? (
                    sortedMonthly.map(row => (
                      <TableRow key={row.year_month}>
                        <TableCell className="font-medium">{row.year_month}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.purchase_total, 2)}</TableCell>
                        <TableCell className="text-right text-destructive">${formatNumber(row.purchase_return, 2)}</TableCell>
                        <TableCell className="text-right font-medium">${formatNumber(row.net_purchase, 2)}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.sales_total, 2)}</TableCell>
                        <TableCell className="text-right text-destructive">${formatNumber(row.sales_return, 2)}</TableCell>
                        <TableCell className="text-right font-medium">${formatNumber(row.net_sales, 2)}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.cogs_total, 2)}</TableCell>
                        <TableCell className={`text-right font-bold ${Number(row.gross_profit) >= 0 ? 'text-status-success-text' : 'text-destructive'}`}>
                          ${formatNumber(row.gross_profit, 2)}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <EmptyRow colSpan={9} />
                  )}
                </TableBody>
              </Table>
            </div>
        </PageTabContent>

        <PageTabContent value="partner" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportPartnerCSV} disabled={!byPartner?.length}>
              <Download className="mr-2 h-4 w-4" />{t('reportsPages.shared.exportCsv')}
            </Button>
          </div>
          <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead sortKey="partner_code" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort}>{t('reportsPages.purchaseSales.partner.code')}</SortableTableHead>
                    <SortableTableHead sortKey="partner_name" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort}>{t('reportsPages.purchaseSales.partner.name')}</SortableTableHead>
                    <SortableTableHead sortKey="partner_type" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort}>{t('reportsPages.purchaseSales.partner.type')}</SortableTableHead>
                    <SortableTableHead sortKey="total_amount" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort} className="text-right">{t('reportsPages.purchaseSales.partner.totalAmount')}</SortableTableHead>
                    <SortableTableHead sortKey="return_amount" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort} className="text-right">{t('reportsPages.purchaseSales.partner.returnAmount')}</SortableTableHead>
                    <SortableTableHead sortKey="net_amount" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort} className="text-right">{t('reportsPages.purchaseSales.partner.netAmount')}</SortableTableHead>
                    <SortableTableHead sortKey="doc_count" currentSort={sortPartner.column} currentDirection={sortPartner.direction} onSort={togglePartnerSort} className="text-right">{t('reportsPages.purchaseSales.partner.docCount')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loadingPartner ? (
                    <TableRow>
                      <TableCell colSpan={7} className="p-0">
                        <TableSkeleton rows={8} cols={7} />
                      </TableCell>
                    </TableRow>
                  ) : sortedPartner && sortedPartner.length > 0 ? (
                    sortedPartner.map(row => (
                      <TableRow key={row.partner_id}>
                        <TableCell className="font-mono text-sm">{row.partner_code}</TableCell>
                        <TableCell className="font-medium">{row.partner_name}</TableCell>
                        <TableCell>
                          <Badge variant={row.partner_type === 'supplier' ? 'secondary' : 'outline'}>
                            {row.partner_type === 'supplier' ? t('reportsPages.shared.supplier') : t('reportsPages.shared.customer')}
                          </Badge>
                        </TableCell>
                        <TableCell className="text-right">${formatNumber(row.total_amount, 2)}</TableCell>
                        <TableCell className="text-right text-destructive">${formatNumber(row.return_amount, 2)}</TableCell>
                        <TableCell className="text-right font-medium">${formatNumber(row.net_amount, 2)}</TableCell>
                        <TableCell className="text-right">{row.doc_count}</TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <EmptyRow colSpan={7} />
                  )}
                </TableBody>
              </Table>
            </div>
        </PageTabContent>

        <PageTabContent value="category" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportCategoryCSV} disabled={!byCategory?.length}>
              <Download className="mr-2 h-4 w-4" />{t('reportsPages.shared.exportCsv')}
            </Button>
          </div>
          <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead sortKey="category_name" currentSort={sortCategory.column} currentDirection={sortCategory.direction} onSort={toggleCategorySort}>{t('reportsPages.purchaseSales.category.productCategory')}</SortableTableHead>
                    <SortableTableHead sortKey="purchase_amount" currentSort={sortCategory.column} currentDirection={sortCategory.direction} onSort={toggleCategorySort} className="text-right">{t('reportsPages.purchaseSales.category.purchaseAmount')}</SortableTableHead>
                    <SortableTableHead sortKey="sales_amount" currentSort={sortCategory.column} currentDirection={sortCategory.direction} onSort={toggleCategorySort} className="text-right">{t('reportsPages.purchaseSales.category.salesAmount')}</SortableTableHead>
                    <SortableTableHead sortKey="cogs_amount" currentSort={sortCategory.column} currentDirection={sortCategory.direction} onSort={toggleCategorySort} className="text-right">{t('reportsPages.purchaseSales.monthly.cogs')}</SortableTableHead>
                    <SortableTableHead sortKey="gross_profit" currentSort={sortCategory.column} currentDirection={sortCategory.direction} onSort={toggleCategorySort} className="text-right">{t('reportsPages.purchaseSales.monthly.grossProfit')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loadingCategory ? (
                    <TableRow>
                      <TableCell colSpan={5} className="p-0">
                        <TableSkeleton rows={8} cols={5} />
                      </TableCell>
                    </TableRow>
                  ) : sortedCategory && sortedCategory.length > 0 ? (
                    sortedCategory.map(row => (
                      <TableRow key={row.category_name}>
                        <TableCell className="font-medium">{row.category_name}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.purchase_amount, 2)}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.sales_amount, 2)}</TableCell>
                        <TableCell className="text-right">${formatNumber(row.cogs_amount, 2)}</TableCell>
                        <TableCell className={`text-right font-bold ${Number(row.gross_profit) >= 0 ? 'text-status-success-text' : 'text-destructive'}`}>
                          ${formatNumber(row.gross_profit, 2)}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <EmptyRow colSpan={5} />
                  )}
                </TableBody>
              </Table>
            </div>
        </PageTabContent>
      </PageTabs>
    </div>
  )
}

function EmptyRow({ colSpan }: { colSpan: number }) {
  const { t } = useTranslation()
  return <TableEmptyRow colSpan={colSpan} icon={BarChart3} title={t('common.noData')} />
}

function downloadCSV(filename: string, headers: string[], rows: (string | number | null)[][]) {
  const csvContent = [headers, ...rows]
    .map(row => row.map(cell => `"${cell ?? ''}"`).join(','))
    .join('\n')
  const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = `${filename}_${new Date().toISOString().split('T')[0]}.csv`
  link.click()
}
