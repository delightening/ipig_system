import api from './client'

export interface ByproductMonthlyFilter {
  start_date?: string
  end_date?: string
}

export interface ByproductMonthlyRow {
  sampled_at: string
  iacuc_no: string | null
  protocol_title: string | null
  ear_tag: string
  requester_display: string | null
  sample_content: string
  collector_name: string | null
  id: string
}

export const byproductMonthlyReportApi = {
  query: async (filter: ByproductMonthlyFilter): Promise<ByproductMonthlyRow[]> => {
    const res = await api.post<ByproductMonthlyRow[]>('/reports/byproduct-monthly', filter)
    return res.data
  },

  exportXlsx: async (filter: ByproductMonthlyFilter) => {
    const res = await api.post('/reports/byproduct-monthly/xlsx', filter, {
      responseType: 'blob',
    })
    const blob = new Blob([(res as { data: BlobPart }).data], {
      type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    })
    const cd = (res as { headers?: Record<string, string> }).headers?.['content-disposition'] ?? ''
    const m = /filename="?([^";]+)"?/.exec(cd)
    const filename = m?.[1] ?? `byproduct_monthly_${Date.now()}.xlsx`

    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    a.remove()
    URL.revokeObjectURL(url)
  },
}
