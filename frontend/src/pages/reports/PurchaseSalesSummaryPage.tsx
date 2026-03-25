import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import type {
  PurchaseSalesMonthlySummary,
  PurchaseSalesPartnerSummary,
  PurchaseSalesCategorySummary,
} from '@/types/report'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, Download, BarChart3 } from 'lucide-react'

function buildQs(from: string, to: string) {
  const params = new URLSearchParams()
  if (from) params.set('date_from', from)
  if (to) params.set('date_to', to)
  const qs = params.toString()
  return qs ? `?${qs}` : ''
}

export function PurchaseSalesSummaryPage() {
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
        title="進銷貨彙總報表"
        description="按月份、供應商/客戶、產品類別多維度分析"
      />

      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1">
          <Label>起始日期</Label>
          <Input type="date" value={from} onChange={e => setFrom(e.target.value)} className="w-40" aria-label="起始日期" />
        </div>
        <div className="space-y-1">
          <Label>結束日期</Label>
          <Input type="date" value={to} onChange={e => setTo(e.target.value)} className="w-40" aria-label="結束日期" />
        </div>
      </div>

      <PageTabs
        tabs={[
          { value: 'monthly', label: '月份彙總' },
          { value: 'partner', label: '供應商/客戶排名' },
          { value: 'category', label: '產品類別分析' },
        ]}
        defaultTab="monthly"
      >
        <PageTabContent value="monthly" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportMonthlyCSV} disabled={!monthly?.length}>
              <Download className="mr-2 h-4 w-4" />匯出 CSV
            </Button>
          </div>
          {loadingMonthly ? (
            <LoadingState />
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>月份</TableHead>
                    <TableHead className="text-right">採購總額</TableHead>
                    <TableHead className="text-right">採購退貨</TableHead>
                    <TableHead className="text-right">淨進貨</TableHead>
                    <TableHead className="text-right">銷貨總額</TableHead>
                    <TableHead className="text-right">銷貨退貨</TableHead>
                    <TableHead className="text-right">淨銷貨</TableHead>
                    <TableHead className="text-right">銷貨成本</TableHead>
                    <TableHead className="text-right">毛利</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {monthly && monthly.length > 0 ? (
                    monthly.map(row => (
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
          )}
        </PageTabContent>

        <PageTabContent value="partner" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportPartnerCSV} disabled={!byPartner?.length}>
              <Download className="mr-2 h-4 w-4" />匯出 CSV
            </Button>
          </div>
          {loadingPartner ? (
            <LoadingState />
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>代碼</TableHead>
                    <TableHead>名稱</TableHead>
                    <TableHead>類型</TableHead>
                    <TableHead className="text-right">總金額</TableHead>
                    <TableHead className="text-right">退貨金額</TableHead>
                    <TableHead className="text-right">淨金額</TableHead>
                    <TableHead className="text-right">單據數</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {byPartner && byPartner.length > 0 ? (
                    byPartner.map(row => (
                      <TableRow key={row.partner_id}>
                        <TableCell className="font-mono text-sm">{row.partner_code}</TableCell>
                        <TableCell className="font-medium">{row.partner_name}</TableCell>
                        <TableCell>
                          <Badge variant={row.partner_type === 'supplier' ? 'secondary' : 'outline'}>
                            {row.partner_type === 'supplier' ? '供應商' : '客戶'}
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
          )}
        </PageTabContent>

        <PageTabContent value="category" className="space-y-4">
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={exportCategoryCSV} disabled={!byCategory?.length}>
              <Download className="mr-2 h-4 w-4" />匯出 CSV
            </Button>
          </div>
          {loadingCategory ? (
            <LoadingState />
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>產品類別</TableHead>
                    <TableHead className="text-right">採購金額</TableHead>
                    <TableHead className="text-right">銷貨金額</TableHead>
                    <TableHead className="text-right">銷貨成本</TableHead>
                    <TableHead className="text-right">毛利</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {byCategory && byCategory.length > 0 ? (
                    byCategory.map(row => (
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
          )}
        </PageTabContent>
      </PageTabs>
    </div>
  )
}

function LoadingState() {
  return (
    <div className="flex items-center justify-center py-8">
      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  )
}

function EmptyRow({ colSpan }: { colSpan: number }) {
  return <TableEmptyRow colSpan={colSpan} icon={BarChart3} title="尚無資料" />
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
