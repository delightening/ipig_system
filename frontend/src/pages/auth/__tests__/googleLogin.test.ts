import { describe, it, expect } from 'vitest'

import zhTW from '@/locales/zh-TW.json'
import en from '@/locales/en.json'

import {
  googleErrorMessageKey,
  isNeutralGoogleError,
  normalizeGoogleErrorCode,
  parseGoogleRedirect,
  type GoogleErrorCode,
} from '../googleLogin'

/**
 * R121：Google 登入導回登入頁的網址解析。
 * 後端的導向格式定義在 backend/src/handlers/auth/google.rs；兩邊不一致時登入會「成功了卻沒反應」。
 */

const params = (query: string) => new URLSearchParams(query)
const SAMPLE_JWT = 'eyJhbGciOiJFUzI1NiJ9.eyJzdWIiOiJ4In0.c2ln'

describe('parseGoogleRedirect', () => {
  it('沒有任何 Google 參數時回 null（一般開啟登入頁）', () => {
    expect(parseGoogleRedirect(params(''), '')).toBeNull()
    expect(parseGoogleRedirect(params('reason=session_expired'), '')).toBeNull()
  })

  it('?google=success 視為成功', () => {
    expect(parseGoogleRedirect(params('google=success'), '')).toEqual({ kind: 'success' })
  })

  it('google 參數不是 success 時不當成功', () => {
    expect(parseGoogleRedirect(params('google=yes'), '')).toBeNull()
  })

  it('#google_2fa=<jwt> 取出 temp token', () => {
    expect(parseGoogleRedirect(params(''), `#google_2fa=${SAMPLE_JWT}`)).toEqual({
      kind: 'twoFactor',
      tempToken: SAMPLE_JWT,
    })
  })

  it('fragment 裡的 token 形狀不對時不送出，改報 invalid_state', () => {
    expect(parseGoogleRedirect(params(''), '#google_2fa=not-a-jwt')).toEqual({
      kind: 'error',
      code: 'invalid_state',
    })
    expect(parseGoogleRedirect(params(''), '#google_2fa=')).toEqual({
      kind: 'error',
      code: 'invalid_state',
    })
  })

  it('?google_error=<code> 帶出錯誤代碼；未知代碼歸為 unknown', () => {
    expect(parseGoogleRedirect(params('google_error=no_account'), '')).toEqual({
      kind: 'error',
      code: 'no_account',
    })
    expect(parseGoogleRedirect(params('google_error=<script>'), '')).toEqual({
      kind: 'error',
      code: 'unknown',
    })
  })

  it('2FA fragment 優先於 query（兩者同時出現時不可跳過第二關）', () => {
    expect(parseGoogleRedirect(params('google=success'), `#google_2fa=${SAMPLE_JWT}`)).toEqual({
      kind: 'twoFactor',
      tempToken: SAMPLE_JWT,
    })
  })
})

describe('Google 錯誤代碼與語言包', () => {
  // 與後端 GoogleLoginFailure::code() 同一份清單，外加前端的 session_missing 與 unknown
  const ALL_CODES: GoogleErrorCode[] = [
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
    'unknown',
  ]

  const lookup = (tree: unknown, path: string): unknown =>
    path.split('.').reduce<unknown>(
      (node, key) => (node && typeof node === 'object' ? (node as Record<string, unknown>)[key] : undefined),
      tree,
    )

  it.each(ALL_CODES)('%s 在 zh-TW 與 en 都有訊息', (code) => {
    const key = googleErrorMessageKey(code)
    expect(typeof lookup(zhTW, key)).toBe('string')
    expect(typeof lookup(en, key)).toBe('string')
  })

  it('已知代碼原樣保留', () => {
    ALL_CODES.filter((c) => c !== 'unknown').forEach((code) => {
      expect(normalizeGoogleErrorCode(code)).toBe(code)
    })
  })

  it('只有使用者自己取消是中性訊息，其餘都是錯誤', () => {
    expect(ALL_CODES.filter(isNeutralGoogleError)).toEqual(['cancelled'])
  })
})
