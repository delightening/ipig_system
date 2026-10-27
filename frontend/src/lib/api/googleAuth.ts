import api from './client'

/**
 * R121：Google 帳號登入（OIDC）。
 *
 * 整個 OAuth 流程都在後端（`backend/src/handlers/auth/google.rs`）：前端只負責
 * (1) 問後端有沒有開放、(2) 以頂層導覽進入 start 端點、(3) 在登入頁讀回導向結果。
 * 前端不接觸任何 Google token，也不載入 Google 的 JS（CSP 不必放寬）。
 */
export interface GoogleLoginStatus {
  /** 伺服器是否已設定 Google 登入 */
  enabled: boolean
}

export async function getGoogleLoginStatus(): Promise<GoogleLoginStatus> {
  const { data } = await api.get<GoogleLoginStatus>('/auth/google/status')
  return data
}

/**
 * start 端點網址。必須用頂層導覽（`<a href>` / `window.location`）進入，不能用 XHR：
 * 後端會設 flow cookie 並 302 到 Google 的授權頁。
 * 與 axios 共用同一個 baseURL，避免兩處各自寫死 `/api/v1`。
 *
 * ⚠️ 必須是函式、在呼叫時才讀 `api.defaults`：寫成模組層常數會在 import 當下求值，
 * 而不少測試以 `vi.mock('./client')` 換掉 axios 實例（沒有 `defaults`），
 * 整支測試檔會在載入階段就失敗——第一版就是這樣弄壞了兩支無關的測試。
 */
export function googleLoginStartUrl(): string {
  return `${api.defaults.baseURL ?? ''}/auth/google/start`
}
