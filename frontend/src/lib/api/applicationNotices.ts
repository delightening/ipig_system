import api from './client'

/** 動物試驗申請須知版本（院區層級，admin 管理） */
export interface ApplicationNotice {
  id: string
  version_label: string
  title: string
  /** 須知正文（markdown） */
  content: string
  attachment_id?: string | null
  effective_from: string
  is_active: boolean
  created_by: string
  created_by_name?: string | null
  created_at: string
}

export interface CreateApplicationNoticeRequest {
  version_label: string
  title: string
  content: string
  effective_from: string
  attachment_id?: string | null
}

const BASE = '/application-notices'

export const listApplicationNotices = async () =>
  (await api.get<ApplicationNotice[]>(BASE)).data

/** 當前生效須知（無生效版本時回 null）。 */
export const getActiveApplicationNotice = async () =>
  (await api.get<ApplicationNotice | null>(`${BASE}/active`)).data

export const createApplicationNotice = async (data: CreateApplicationNoticeRequest) =>
  (await api.post<ApplicationNotice>(BASE, data)).data

export const activateApplicationNotice = async (id: string) =>
  (await api.post<ApplicationNotice>(`${BASE}/${id}/activate`)).data
