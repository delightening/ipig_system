import api from './client'

/**
 * R30-27b：影響 UI 行為的 feature flag 集合
 *
 * 從 GET /api/v1/system/features 取得；前端據此決定是否顯示對應 UI（如
 * role / permission 變更簽章 dialog）。已登入即可呼叫。
 */
export interface SystemFeatures {
  /** R30-27：role / permission 變更是否強制密碼 + 手寫雙因子簽章 */
  role_signature_required: boolean
}

export async function getSystemFeatures(): Promise<SystemFeatures> {
  const { data } = await api.get<SystemFeatures>('/system/features')
  return data
}

/**
 * R30-27：role / permission 變更請求附帶的雙因子電子簽章 payload。
 *
 * `role_signature_required=true` 時 backend 強制要求；false 時欄位忽略。
 * 對應 21 CFR §11.10(d) 存取控制簽章不可否認性。
 *
 * `stroke_data` 為手寫筆畫向量（含時序、壓力等），用於日後鑑定簽章樣式（客戶 /
 * 員工 / 外部操作人員 / 老闆 — 各角色簽名特徵差異）。
 */
export interface MutationSignaturePayload {
  password: string
  handwriting_svg: string
  stroke_data?: object[]
}

// ============================================
// R30-27c：簽章 phone bridge（桌機掃 QR → 手機簽 → 桌機 consume）
// ============================================

export interface StartSignatureBridgeResponse {
  session_id: string
  /** plaintext mobile_token — 只此一次回給桌機，桌機編入 QR */
  mobile_token: string
  expires_at: string
}

export interface SignatureBridgeStatus {
  /** 'PENDING' | 'COMPLETED' | 'CONSUMED' | 'EXPIRED' */
  status: string
}

export interface ConsumeSignatureBridgeResponse {
  payload: MutationSignaturePayload
  submitted_at: string
}

/** 桌機開 session（已登入 + CSRF）。 */
export async function startSignatureBridge(
  purpose: string,
): Promise<StartSignatureBridgeResponse> {
  const { data } = await api.post<StartSignatureBridgeResponse>('/signing-bridge/start', {
    purpose,
  })
  return data
}

/** 桌機輪詢 session 狀態（已登入 + owner-only）。 */
export async function getSignatureBridgeStatus(
  sessionId: string,
): Promise<SignatureBridgeStatus> {
  const { data } = await api.get<SignatureBridgeStatus>(`/signing-bridge/${sessionId}/status`)
  return data
}

/** 桌機取走 payload（status COMPLETED → CONSUMED）。 */
export async function consumeSignatureBridge(
  sessionId: string,
): Promise<ConsumeSignatureBridgeResponse> {
  const { data } = await api.get<ConsumeSignatureBridgeResponse>(
    `/signing-bridge/${sessionId}/consume`,
  )
  return data
}

/** 手機公開提交（不需 JWT，由 mobile_token bearer 驗證）。 */
export async function submitSignatureBridgePublic(
  sessionId: string,
  mobileToken: string,
  payload: MutationSignaturePayload,
): Promise<void> {
  await api.post(`/public/signing-bridge/${sessionId}/submit`, {
    mobile_token: mobileToken,
    payload,
  })
}
