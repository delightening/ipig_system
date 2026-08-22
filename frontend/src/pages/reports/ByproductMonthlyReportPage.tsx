import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Download, Search, FlaskConical, AlertCircle } from 'lucide-react'

import { uiLocale } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import {
  byproductMonthlyReportApi,
  type ByproductMonthlyFilter,
  type ByproductMonthlyRow,
} from '@/lib/api/byproductMonthlyReport'

export function ByproductMonthlyReportPage() {
  const [startDate, setStartDate] = useState('')
  const [endDate, setEndDate] = useState('')
  const [filter, setFilter] = useState<ByproductMonthlyFilter | null>(null)
  const [exporting, setExporting] = useState(false)

  const { data: rows, isLoading, error } = useQuery({
    queryKey: ['byproduct-monthly-report', filter],
    queryFn: () => byproductMonthlyReportApi.query(filter!),
    enabled: filter != null,
  })

  const buildFilter = (): ByproductMonthlyFilter => {
    const f: ByproductMonthlyFilter = {}
    if (startDate) f.start_date = startDate
    if (endDate) f.end_date = endDate
    return f
  }

  const handleSearch = () => setFilter(buildFilter())

  const handleExport = async () => {
    setExporting(true)
    try {
      await byproductMonthlyReportApi.exportXlsx(filter ?? buildFilter())
    } catch {
      toast({ title: '匯出失敗', description: '請稍後再試', variant: 'destructive' })
    } finally {
      setExporting(false)
    }
  }

  const formatDate = (iso: string) => {
    if (!iso) return '-'
    try {
      return new Date(iso).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })
    } catch {
      return iso.includes('T') ? iso.split('T')[0] : iso
    }
  }

  const hasData = rows && rows.length > 0

  return (
    <div className="container mx-auto py-6 space-y-6">
      <div className="flex items-center gap-3">
        <FlaskConical className="h-6 w-6 text-primary" />
        <h1 className="text-2xl font-bold">廢棄物再利用月結報表</h1>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 p-4 border rounded-lg bg-muted/50">
        <div>
          <Label>開始日期</Label>
          <Input type="date" value={startDate} onChange={e => setStartDate(e.target.value)} />
        </div>
        <div>
          <Label>結束日期</Label>
          <Input type="date" value={endDate} onChange={e => setEndDate(e.target.value)} />
        </div>
        <div className="flex items-end gap-2">
          <Button onClick={handleSearch} disabled={isLoading}>
            <Search className="h-4 w-4 mr-1" />
            查詢
          </Button>
          <Button variant="outline" onClick={handleExport} disabled={exporting || !hasData}>
            <Download className="h-4 w-4 mr-1" />
            {exporting ? '匯出中...' : '匯出 Excel'}
          </Button>
        </div>
      </div>

      {isLoading && (
        <div className="flex justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 p-4 border border-destructive/50 rounded-lg bg-destructive/10 text-destructive">
          <AlertCircle className="h-5 w-5 shrink-0" />
          <span>查詢失敗，請確認權限或稍後再試</span>
        </div>
      )}

      {filter == null && !isLoading && (
        <div className="text-center py-12 text-muted-foreground">
          請選擇日期區間並點擊「查詢」
        </div>
      )}

      {rows && (
        <div className="border rounded-lg overflow-auto">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="px-3 py-2 text-left whitespace-nowrap">採樣日期</th>
                <th className="px-3 py-2 text-left">案子</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">耳號</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">需求客戶</th>
                <th className="px-3 py-2 text-left">採樣內容</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">記錄者</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r: ByproductMonthlyRow) => (
                <tr key={r.id} className="border-t hover:bg-muted/30">
                  <td className="px-3 py-2 whitespace-nowrap">{formatDate(r.sampled_at)}</td>
                  <td className="px-3 py-2">{r.protocol_title ?? r.iacuc_no ?? '-'}</td>
                  <td className="px-3 py-2 whitespace-nowrap font-mono">{r.ear_tag}</td>
                  <td className="px-3 py-2 whitespace-nowrap">{r.requester_display ?? '-'}</td>
                  <td className="px-3 py-2">{r.sample_content}</td>
                  <td className="px-3 py-2 whitespace-nowrap">{r.collector_name ?? '-'}</td>
                </tr>
              ))}
              {rows.length === 0 && (
                <tr>
                  <td colSpan={6} className="px-3 py-8 text-center text-muted-foreground">
                    查無資料
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          <div className="px-3 py-2 text-sm text-muted-foreground border-t">
            共 {rows.length} 筆
          </div>
        </div>
      )}
    </div>
  )
}
