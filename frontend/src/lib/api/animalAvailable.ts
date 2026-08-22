import api from './client'

// R47 — 可用豬隻快速查詢（庫存盤點）API client
// 對應 backend GET /api/v1/animals/available

export interface AvailablePigQuery {
  sex?: 'male' | 'female' | null
  age_months_min?: number | null
  age_months_max?: number | null
  weight_min?: number | null
  weight_max?: number | null
  include_breeding?: boolean
  page?: number
  per_page?: number
}

export interface AvailablePigRow {
  id: string
  ear_tag: string
  animal_no: string | null
  breed: 'minipig' | 'white' | 'lyd' | 'other'
  gender: 'male' | 'female'
  birth_date: string
  age_months: number
  latest_weight_kg: string
  weight_measured_at: string
  pen_location: string | null
  remark: string | null
}

export interface AvailablePigSummary {
  total: number
  male: number
  female: number
  by_breed: Record<string, number>
  excluded_weight_expired: number
}

export interface AvailablePigListResponse {
  animals: AvailablePigRow[]
  summary: AvailablePigSummary
  page: number
  per_page: number
}

/** 移除 null / undefined 欄位後組成 query params */
function pruneParams(q: AvailablePigQuery): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [k, v] of Object.entries(q)) {
    if (v !== null && v !== undefined && v !== '') {
      out[k] = String(v)
    }
  }
  return out
}

export const availablePigsApi = {
  list: (q: AvailablePigQuery) =>
    api.get<AvailablePigListResponse>('/animals/available', { params: pruneParams(q) }),

  /** 觸發瀏覽器下載 Excel 檔案 */
  exportXlsx: async (q: AvailablePigQuery) => {
    const params = pruneParams(q)
    params.export = 'xlsx'
    const res = await api.get('/animals/available', {
      params,
      responseType: 'blob',
    })
    const blob = new Blob([(res as { data: BlobPart }).data], {
      type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    })
    const cd = (res as { headers?: Record<string, string> }).headers?.['content-disposition'] ?? ''
    const m = /filename="?([^";]+)"?/.exec(cd)
    const filename = m?.[1] ?? `available_pigs_${Date.now()}.xlsx`

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
