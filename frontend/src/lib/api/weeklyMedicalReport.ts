import api from './client'

export interface WeeklyMedicalReportFilter {
  animal_ear_tags?: string[]
  protocol_ids?: string[]
  start_date?: string
  end_date?: string
}

export interface MedicalTimelineEvent {
  animal_id: string
  ear_tag: string
  iacuc_no: string | null
  event_date: string
  event_type: string
  summary: string
  details: string | null
  actor_name: string | null
  source_id: string
  created_at: string
  birth_date: string | null
  latest_weight: number | null
  sponsor_name: string | null
  equipment_used: string | null
  anesthesia_start: string | null
  anesthesia_end: string | null
}

export const weeklyMedicalReportApi = {
  query: async (filter: WeeklyMedicalReportFilter): Promise<MedicalTimelineEvent[]> => {
    const res = await api.post<MedicalTimelineEvent[]>('/reports/animal-medical/weekly', filter)
    return res.data
  },

  exportXlsx: async (filter: WeeklyMedicalReportFilter) => {
    const res = await api.post('/reports/animal-medical/weekly/xlsx', filter, {
      responseType: 'blob',
    })
    _downloadBlob(res, 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', 'weekly_medical_report.xlsx')
  },

  exportPdf: async (filter: WeeklyMedicalReportFilter) => {
    const res = await api.post('/reports/animal-medical/weekly/pdf', filter, {
      responseType: 'blob',
    })
    _downloadBlob(res, 'application/pdf', 'weekly_medical_report.pdf')
  },
}

function _downloadBlob(res: unknown, mime: string, fallbackName: string) {
  const r = res as { data: BlobPart; headers?: Record<string, string> }
  const blob = new Blob([r.data], { type: mime })
  const cd = r.headers?.['content-disposition'] ?? ''
  const m = /filename="?([^";]+)"?/.exec(cd)
  const filename = m?.[1] ?? fallbackName

  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}
