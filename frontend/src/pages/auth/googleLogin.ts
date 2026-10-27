/**
 * R121：Google 登入導回登入頁時，後端把結果放在網址上（見 `backend/src/handlers/auth/google.rs`）：
 * - 成功：`/login?google=success`（access / refresh cookie 已由後端設好）
 * - 需要 2FA：`/login#google_2fa=<temp token>`（放 fragment：不會送到任何伺服器、不進 Referer）
 * - 失敗：`/login?google_error=<code>`
 *
 * 本檔只做純解析，不碰 React，方便單元測試。
 */

/** 登入頁網址上跟 Google 登入有關的 query 參數；處理完要從網址移除 */
export const GOOGLE_SEARCH_PARAMS = ['google', 'google_error'] as const

const GOOGLE_2FA_HASH_KEY = 'google_2fa'

/** 2FA temp token 是三段式 JWT；格式不對就不要當 token 送出去 */
const JWT_SHAPE = /^[\w-]+\.[\w-]+\.[\w-]+$/

/**
 * 與後端 `GoogleLoginFailure::code()` 一一對應，另加前端自己判定的 `session_missing`
 * （後端回報成功，但隨後讀不到登入狀態）。不在清單內的代碼一律顯示 `unknown`。
 */
const KNOWN_ERROR_CODES = [
  'not_configured',
  'cancelled',
  'invalid_state',
  'provider_error',
  'email_not_verified',
  'no_account',
  'ambiguous_account',
  'account_unavailable',
  'identity_conflict',
  'session_missing',
] as const

export type GoogleErrorCode = (typeof KNOWN_ERROR_CODES)[number] | 'unknown'

export type GoogleRedirectResult =
  | { kind: 'success' }
  | { kind: 'twoFactor'; tempToken: string }
  | { kind: 'error'; code: GoogleErrorCode }

export function normalizeGoogleErrorCode(raw: string): GoogleErrorCode {
  return (KNOWN_ERROR_CODES as readonly string[]).includes(raw) ? (raw as GoogleErrorCode) : 'unknown'
}

/** 讀出導向結果；網址上沒有任何 Google 登入相關參數時回 `null`。 */
export function parseGoogleRedirect(search: URLSearchParams, hash: string): GoogleRedirectResult | null {
  const hashParams = new URLSearchParams(hash.startsWith('#') ? hash.slice(1) : hash)
  const tempToken = hashParams.get(GOOGLE_2FA_HASH_KEY)
  if (tempToken !== null) {
    return JWT_SHAPE.test(tempToken)
      ? { kind: 'twoFactor', tempToken }
      : { kind: 'error', code: 'invalid_state' }
  }

  const error = search.get('google_error')
  if (error !== null) return { kind: 'error', code: normalizeGoogleErrorCode(error) }

  if (search.get('google') === 'success') return { kind: 'success' }
  return null
}

export function googleErrorMessageKey(code: GoogleErrorCode): string {
  return `auth.login.googleErrors.${code}`
}

/** 使用者自己在 Google 畫面按取消，不是錯誤：用中性 toast，不用紅色警告。 */
export function isNeutralGoogleError(code: GoogleErrorCode): boolean {
  return code === 'cancelled'
}
