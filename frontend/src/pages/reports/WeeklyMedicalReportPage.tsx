import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Download, Search, FileSpreadsheet, FileText } from 'lucide-react'

import api from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { GuestDateNotice } from '@/components/ui/guest-date-notice'
import {
  weeklyMedicalReportApi,
  type WeeklyMedicalReportFilter,
  type MedicalTimelineEvent,
} from '@/lib/api/weeklyMedicalReport'
import type { ProtocolListItem } from '@/types/aup'

/** ISO 日期字串（YYYY-MM-DD）拆成年份／月日兩行顯示，縮短欄寬用 */
function DateStack({ value }: { value: string | null | undefined }) {
  if (!value) return <>-</>
  const [y, m, d] = value.split('-')
  return (
    <div className="flex flex-col leading-tight tabular-nums">
      <span className="text-[11px] text-muted-foreground">{y}</span>
      <span className="font-semibold tracking-wide">{m}{d}</span>
    </div>
  )
}

export function WeeklyMedicalReportPage() {
  const [protocolId, setProtocolId] = useState('')
  const [earTags, setEarTags] = useState('')
  const [startDate, setStartDate] = useState('')
  const [endDate, setEndDate] = useState('')
  const [filter, setFilter] = useState<WeeklyMedicalReportFilter | null>(null)
  const [exporting, setExporting] = useState<'xlsx' | 'pdf' | null>(null)

  const { data: protocols } = useQuery({
    queryKey: ['protocols-for-report'],
    queryFn: async () => {
      const res = await api.get<ProtocolListItem[]>('/protocols')
      return res.data.filter(p => p.status === 'APPROVED')
    },
  })

  const { data: events, isLoading } = useQuery({
    queryKey: ['weekly-medical-report', filter],
    queryFn: () => weeklyMedicalReportApi.query(filter!),
    enabled: filter != null,
  })

  const buildFilter = (): WeeklyMedicalReportFilter => {
    const f: WeeklyMedicalReportFilter = {}
    if (protocolId) f.protocol_ids = [protocolId]
    if (earTags.trim()) {
      f.animal_ear_tags = earTags.split(',').map(s => s.trim()).filter(Boolean)
    }
    if (startDate) f.start_date = startDate
    if (endDate) f.end_date = endDate
    return f
  }

  const handleSearch = () => setFilter(buildFilter())

  const handleExport = async (format: 'xlsx' | 'pdf') => {
    setExporting(format)
    try {
      if (format === 'xlsx') {
        await weeklyMedicalReportApi.exportXlsx(filter ?? buildFilter())
      } else {
        await weeklyMedicalReportApi.exportPdf(filter ?? buildFilter())
      }
    } finally {
      setExporting(null)
    }
  }

  return (
    <div className="container mx-auto py-6 space-y-6">
      <div className="flex items-center gap-3">
        <FileSpreadsheet className="h-6 w-6 text-primary" />
        <h1 className="text-2xl font-bold">豬隻病歷彙整報表</h1>
      </div>

      <div className="p-4 border rounded-lg bg-muted/50 space-y-4">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label>計畫案</Label>
            <select
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              value={protocolId}
              onChange={e => setProtocolId(e.target.value)}
            >
              <option value="">全部計畫案</option>
              {(protocols ?? []).map(p => (
                <option key={p.id} value={p.id}>
                  {p.iacuc_no ? `${p.iacuc_no} — ` : ''}{p.title}
                </option>
              ))}
            </select>
          </div>
          <div>
            <Label>耳號（逗號分隔，可留空）</Label>
            <Input placeholder="640, 727, 784" value={earTags} onChange={e => setEarTags(e.target.value)} />
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label>開始日期</Label>
            <Input type="date" value={startDate} onChange={e => setStartDate(e.target.value)} />
          </div>
          <div>
            <Label>結束日期</Label>
            <Input type="date" value={endDate} onChange={e => setEndDate(e.target.value)} />
          </div>
        </div>
        <GuestDateNotice />
        <div className="flex items-center gap-2 pt-1">
          <Button onClick={handleSearch} disabled={isLoading}>
            <Search className="h-4 w-4 mr-1" />
            查詢
          </Button>
          <Button variant="outline" onClick={() => handleExport('xlsx')} disabled={!!exporting}>
            <Download className="h-4 w-4 mr-1" />
            {exporting === 'xlsx' ? '匯出中...' : 'Excel'}
          </Button>
          <Button variant="outline" onClick={() => handleExport('pdf')} disabled={!!exporting}>
            <FileText className="h-4 w-4 mr-1" />
            {exporting === 'pdf' ? '匯出中...' : 'PDF'}
          </Button>
        </div>
      </div>

      {isLoading && (
        <div className="flex justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>
      )}

      {events && (
        <div className="border rounded-lg overflow-auto">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="px-3 py-2 text-left whitespace-nowrap">日期</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">耳號</th>
                <th className="px-2 py-2 text-center whitespace-nowrap [writing-mode:vertical-rl] [text-orientation:upright]">類別</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">出生日期</th>
                <th className="px-3 py-2 text-right whitespace-nowrap">體重(kg)</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">試驗單位</th>
                <th className="px-3 py-2 text-left">試驗內容</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">特殊儀器</th>
                <th className="px-3 py-2 text-left whitespace-nowrap">麻醉時間</th>
              </tr>
            </thead>
            <tbody>
              {events.map((ev: MedicalTimelineEvent) => (
                <tr key={ev.source_id} className="border-t hover:bg-muted/30">
                  <td className="px-3 py-2 whitespace-nowrap"><DateStack value={ev.event_date} /></td>
                  <td className="px-3 py-2 whitespace-nowrap font-mono">{ev.ear_tag}</td>
                  <td className="px-2 py-2 text-center whitespace-nowrap">
                    <span className={`inline-block px-1 py-1.5 rounded text-xs font-medium text-white tracking-widest [writing-mode:vertical-rl] [text-orientation:upright] ${
                      ev.event_type === 'SURGERY' ? 'bg-red-500'
                      : ev.event_type === 'BLOOD_TEST' ? 'bg-amber-500'
                      : 'bg-blue-500'
                    }`}>
                      {ev.event_type === 'OBSERVATION' ? '觀察'
                       : ev.event_type === 'SURGERY' ? '手術'
                       : ev.event_type === 'BLOOD_TEST' ? '血檢'
                       : ev.event_type}
                    </span>
                  </td>
                  <td className="px-3 py-2 whitespace-nowrap"><DateStack value={ev.birth_date} /></td>
                  <td className="px-3 py-2 text-right">{ev.latest_weight ?? '-'}</td>
                  <td className="px-3 py-2">
                    <div className="whitespace-nowrap">{ev.iacuc_no ?? '-'}</div>
                    {ev.sponsor_name && (
                      <div className="whitespace-nowrap text-xs text-muted-foreground">{ev.sponsor_name}</div>
                    )}
                  </td>
                  <td className="px-3 py-2">{ev.details ?? ev.summary}</td>
                  <td className="px-3 py-2 whitespace-nowrap">{ev.equipment_used ?? ''}</td>
                  <td className="px-3 py-2 whitespace-nowrap text-xs">
                    {ev.anesthesia_start && ev.anesthesia_end
                      ? `${new Date(ev.anesthesia_start).toLocaleTimeString(uiLocale(), { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit' })}–${new Date(ev.anesthesia_end).toLocaleTimeString(uiLocale(), { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit' })}`
                      : ''}
                  </td>
                </tr>
              ))}
              {events.length === 0 && (
                <tr>
                  <td colSpan={9} className="px-3 py-8 text-center text-muted-foreground">查無資料</td>
                </tr>
              )}
            </tbody>
          </table>
          <div className="px-3 py-2 text-sm text-muted-foreground border-t">共 {events.length} 筆</div>
        </div>
      )}
    </div>
  )
}
