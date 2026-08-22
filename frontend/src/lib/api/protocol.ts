import api from './client'

import type { ProtocolActivity } from '@/types'

export const getProtocolActivities = async (id: string): Promise<ProtocolActivity[]> => {
  const response = await api.get<ProtocolActivity[]>(`/protocols/${id}/activities`)
  return response.data
}

/** 匯入已核准計劃請求（對應後端 ImportApprovedProtocolRequest） */
export interface ImportApprovedProtocolRequest {
  title: string
  /** 系統內 PI 使用者 id；外部 PI（無帳號）為 null，PI 身分存於 working_content.basic */
  pi_user_id: string | null
  /** 計劃負責人 / Study Director（必填，本公司員工 EXPERIMENT_STAFF） */
  study_director_user_id: string
  /** 計劃書內容（外部 PI 時帶 basic.pi / basic.sponsor） */
  working_content?: Record<string, unknown> | null
  /** IACUC 核准編號（例 PIG-115001） */
  iacuc_no: string
  /** 申請編號（選填，例 APIG-103001） */
  application_no?: string | null
  start_date?: string | null
  end_date?: string | null
  /** 審查里程碑日期（歷史時間軸，依序：申請→預審→獸醫→委員一審→補件→委員二審→核准） */
  submitted_at?: string | null
  pre_review_at?: string | null
  vet_review_at?: string | null
  committee_first_review_at?: string | null
  revision_required_at?: string | null
  committee_second_review_at?: string | null
  approved_at?: string | null
  remark?: string | null
  /** 計畫書表單版本鍵（C/D/E/F…），補登「先選版本」帶入；驅動版本名冊渲染 */
  source_form_version?: string | null
}

/** 匯入場內既有、已通過審查的計劃（直接成 APPROVED，跳過審查流程） */
export const importApprovedProtocol = async (data: ImportApprovedProtocolRequest) => {
  const response = await api.post('/protocols/import-approved', data)
  return response.data
}

// ── import P3：補登審查文件 + 完成補登 ──────────────────────────────

/** 一條補登審查意見（系統內審查者用 id、院外用姓名擇一；補登 UI 預設填姓名） */
export interface ImportReviewComment {
  reviewer_id?: string | null
  reviewer_name?: string | null
  content: string
  /** 申請人 / 客戶對此意見的回覆（選填） */
  reply?: string | null
  /** 對應計畫書項次（如 4.1.2，選填） */
  section_no?: string | null
}

/** 一位委員的意見（一審 + 二審） */
export interface ImportCommitteeReviewer {
  reviewer_id?: string | null
  reviewer_name?: string | null
  first_round: ImportReviewComment[]
  second_round: ImportReviewComment[]
}

/** 獸醫師評比與意見 */
export interface ImportVetReview {
  vet_id?: string | null
  vet_name?: string | null
  decision?: string | null
  decision_remark?: string | null
  items: { item_name: string; compliance: string; comment?: string | null; pi_reply?: string | null }[]
  signed_at?: string | null
}

/** 補登 5 份審查文件（主席核准同意函另以附件上傳） */
export interface ImportReviewsRequest {
  secretary_comments: ImportReviewComment[]
  committee_reviewers: ImportCommitteeReviewer[]
  vet_review?: ImportVetReview | null
}

/** 補登審查文件至真實審查表（全量取代；限 import_pending 計劃） */
export const recordImportReviews = async (id: string, data: ImportReviewsRequest) => {
  await api.post(`/protocols/${id}/import-reviews`, data)
}

/** 完成補登：清 import_pending + 建 v1 版本快照 + 記原始版本號 */
export const finalizeImport = async (id: string, originalVersionLabel?: string | null) => {
  const response = await api.post(`/protocols/${id}/finalize-import`, {
    original_version_label: originalVersionLabel || null,
  })
  return response.data
}

// ── PI 帳號開通（外部 PI 補建帳號 + relink；寄信須 admin 核准）─────────────

/** 開通計畫外部 PI 帳號 + relink（不寄信）。限建立者 / SD / admin。 */
export const provisionPiAccount = async (protocolId: string) => {
  const response = await api.post(`/protocols/${protocolId}/provision-pi`)
  return response.data as { pi_user_id: string; email: string; created_new_account: boolean }
}

/** 待核准 / 已寄送的 PI 開通信 */
export interface PiAccountInviteItem {
  id: string
  protocol_id: string
  protocol_no: string
  /** 官方 IACUC 編號（清單顯示用，對齊主計畫清單） */
  iacuc_no?: string | null
  protocol_title: string
  pi_user_id: string
  pi_name: string
  email: string
  status: string
  provisioned_by_name?: string | null
  provisioned_at: string
}

/** 列出 PI 開通信（admin only），預設 pending */
export const listPiAccountInvites = async (status = 'pending'): Promise<PiAccountInviteItem[]> => {
  const response = await api.get<PiAccountInviteItem[]>(`/pi-account-invites?status=${status}`)
  return response.data
}

/** 核准並寄送 PI 開通（設定密碼）信（admin only） */
export const approveSendPiInvite = async (inviteId: string) => {
  const response = await api.post(`/pi-account-invites/${inviteId}/approve-send`)
  return response.data
}
