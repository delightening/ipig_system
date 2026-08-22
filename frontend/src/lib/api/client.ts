import axios, { AxiosError, AxiosResponse, InternalAxiosRequestConfig } from 'axios'
import { useAuthStore, expiresInToTimestamp } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'
import { getGuestDemoData, getGuestWriteToast } from '@/lib/guest-demo/routes'
import { broadcastAuth, markLocalRefresh, msSinceLastBroadcastRefresh } from '@/lib/authBroadcast'
import type { LoginResponse } from '@/types/auth'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  // HttpOnly Cookie 自動隨請求傳送（SEC-02）
  withCredentials: true,
})

// ============================================
// SEC-24: CSRF Token 自動附加
// ============================================

/** 從 document.cookie 中讀取指定名稱的 cookie 值 */
function getCookie(name: string): string | null {
  const match = document.cookie.match(new RegExp(`(?:^|;\\s*)${name}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

// ============================================
// 資源刪除：使用標準 DELETE 方法
// ============================================

/** 刪除資源（使用標準 HTTP DELETE 方法） */
export function deleteResource(
  url: string,
  options?: { data?: object; headers?: { [key: string]: string } },
) {
  return api.delete(url, { data: options?.data, headers: options?.headers })
}

// ============================================
// Guest Demo Mode：攔截 API 請求，回傳靜態 demo data
// 必須在 CSRF interceptor 之前註冊（先執行）
// ============================================
api.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const isGuest = useAuthStore.getState().isGuest()
  if (isGuest) {
    const method = (config.method || 'GET').toUpperCase()
    const demoData = getGuestDemoData(config.url || '', method)
    if (demoData !== undefined) {
      // 對寫入 (POST/PUT/PATCH/DELETE) 顯示「示範模式」toast 提示；GET 靜默回傳 demo data
      if (method !== 'GET') {
        const t = getGuestWriteToast(method, config.url || '')
        if (t) toast({ title: t.title, description: t.description })
      }
      // 用 adapter 短路，不發 HTTP 請求
      config.adapter = () =>
        Promise.resolve({
          data: demoData,
          status: 200,
          statusText: 'OK',
          headers: {},
          config,
        })
    }
  }
  return config
})

// Request interceptor：自動將 csrf_token Cookie 值加到 X-CSRF-Token header
api.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  // FormData 上傳時移除 Content-Type，讓瀏覽器自動設定 multipart/form-data + boundary
  if (config.data instanceof FormData) {
    config.headers.delete('Content-Type')
  }

  const method = (config.method || '').toUpperCase()
  // 只有 POST/PUT/DELETE/PATCH 需要 CSRF token
  if (['POST', 'PUT', 'DELETE', 'PATCH'].includes(method)) {
    const csrfToken = getCookie('csrf_token')
    if (csrfToken) {
      config.headers.set('X-CSRF-Token', csrfToken)
    }
  }
  return config
})

// ============================================
// SEC-25: Refresh Token Queue（防競態）
// ============================================

// SEC-25: Promise-based singleton 防止重複 refresh（修復競態條件）
let refreshPromise: Promise<boolean> | null = null

// 防重複登出：Promise-based gate（修復非原子 flag 競態）
let logoutPromise: Promise<void> | null = null

// C1: refresh 失敗時的 retry backoff（網路抖動 / 後端短暫 5xx 不應立即踢使用者）
const REFRESH_RETRY_BACKOFF_MS = 1000

// 網路層錯誤靜默重試延遲（讓瀏覽器有時間重建 H2 connection）
const NETWORK_RETRY_DELAY_MS = 500

// 2026-05-20: 跨 tab refresh 互斥窗口。進入 navigator.locks 後，若其他 tab 在
// 此毫秒數內剛 refresh 過，本 tab 直接 retry 原請求，不再發 /auth/refresh。
// 避免兩個 tab 各自並行送 in-flight refresh、第二個帶舊 cookie 抵達 backend
// 觸發 reuse detection（即使 backend race window 已拉長至 300s，也不該產生
// 不必要的 audit 噪音）。
const CROSS_TAB_REFRESH_SKIP_WINDOW_MS = 30_000

/**
 * Attempt POST /auth/refresh once，失敗時：
 * - 4xx (含 401 / 400 / 403 / 422) → 立即 throw（token 真失效，retry 無益）
 * - 5xx / network error → 等 backoff 後 retry 一次
 *
 * 兩次都失敗才 throw — 此後 caller 視為真 refresh 失敗（清 auth）。
 * sliding session C1。
 */
export async function attemptRefreshWithRetry(): Promise<AxiosResponse<LoginResponse>> {
  try {
    return await api.post<LoginResponse>('/auth/refresh')
  } catch (err) {
    const e = err as AxiosError
    const status = e.response?.status
    // 4xx → real auth/business failure，不 retry
    if (status !== undefined && status >= 400 && status < 500) {
      throw err
    }
    // 5xx 或無 response（network error） → wait 1s retry 一次
    await new Promise((r) => setTimeout(r, REFRESH_RETRY_BACKOFF_MS))
    return api.post<LoginResponse>('/auth/refresh')
  }
}

/**
 * 將 refresh 放進 navigator.locks 跨 tab 互斥。
 *
 * 流程：
 * 1. `await navigator.locks.request('auth-refresh', ...)` — 同 origin 多 tab 序列化
 * 2. 進入 lock 後，先看其他 tab 是否在 CROSS_TAB_REFRESH_SKIP_WINDOW_MS 內剛
 *    refresh 過（透過 BroadcastChannel 'refreshed' 訊息追蹤），若是直接回 true，
 *    跳過自己的 refresh — 此時 cookie 已被前一個 tab 換成新的，retry 原請求即可
 * 3. 否則執行 attemptRefreshWithRetry，成功後更新本地 expiresAt + 廣播 + 標記
 *
 * 不支援 navigator.locks 的環境（舊瀏覽器）退回原本 in-tab Promise singleton，
 * backend race window 300s 仍能兜底跨 tab race。
 */
async function runRefreshUnderCrossTabLock(): Promise<boolean> {
  const refreshOnce = async (): Promise<boolean> => {
    // 別的 tab 剛 refresh 完，cookie 已新 → skip
    if (msSinceLastBroadcastRefresh() < CROSS_TAB_REFRESH_SKIP_WINDOW_MS) {
      return true
    }
    try {
      const res = await attemptRefreshWithRetry()
      const accessTokenExpiresAt = expiresInToTimestamp(res.data.expires_in)
      useAuthStore.setState({ accessTokenExpiresAt })
      broadcastAuth({ type: 'refreshed', accessTokenExpiresAt })
      markLocalRefresh()
      return true
    } catch {
      return false
    }
  }

  const locks = (navigator as Navigator & { locks?: LockManager }).locks
  if (!locks?.request) {
    return refreshOnce()
  }
  return locks.request('auth-refresh', refreshOnce)
}

// Response interceptor - handle errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as typeof error.config & { _retry?: boolean; _503RetryCount?: number; _csrfRetry?: boolean; _silentError?: boolean; _networkRetry?: boolean }

    // 如果已經在登出流程中，等待完成後拒絕
    if (logoutPromise) {
      return logoutPromise.then(() => Promise.reject(error))
    }

    // 網路層錯誤（ERR_CONNECTION_CLOSED / ECONNRESET）→ 靜默重試一次。
    // H2 connection 在 tab 隱藏 / token 即將到期時閒置超過 nginx keepalive_timeout
    // 會被關閉；polling query 恢復時打到已關閉的連線就會觸發此路徑。
    // 僅重試冪等方法（GET / HEAD / OPTIONS）避免 POST/PUT/DELETE 重複寫入。
    // 等 NETWORK_RETRY_DELAY_MS 讓瀏覽器重新建立連線後 retry；仍失敗才往後走顯示 toast。
    const reqMethod = (originalRequest?.method ?? '').toUpperCase()
    const isSafeMethod = ['GET', 'HEAD', 'OPTIONS'].includes(reqMethod)
    if (!error.response && isSafeMethod && !originalRequest?._networkRetry && originalRequest) {
      originalRequest._networkRetry = true
      await new Promise<void>((r) => setTimeout(r, NETWORK_RETRY_DELAY_MS))
      return api(originalRequest)
    }

    // 503 暫時性錯誤：依 Retry-After 重試（最多 2 次）
    const max503Retries = 2
    const retryCount = originalRequest?._503RetryCount ?? 0
    if (error.response?.status === 503 && retryCount < max503Retries && originalRequest) {
      const retryAfter = error.response?.headers?.['retry-after'] ?? error.response?.headers?.['Retry-After']
      const parsed = retryAfter ? parseInt(String(retryAfter), 10) : NaN
      const delayMs = !isNaN(parsed) ? Math.min(parsed * 1000, 3000) : 1000
      originalRequest._503RetryCount = retryCount + 1
      await new Promise((r) => setTimeout(r, delayMs))
      return api(originalRequest)
    }

    // SEC-24: CSRF Token 自動刷新
    // 419 (Page Expired) 表示 CSRF token 過期/無效，嘗試刷新 token 後重試
    if (
      error.response?.status === 419 &&
      !originalRequest?._csrfRetry &&
      originalRequest
    ) {
      originalRequest._csrfRetry = true
      try {
        // 呼叫任意 GET 端點以取得新的 CSRF cookie（CSRF 中介層會對每個回應重新設定 cookie）
        await api.get('/me')
        return api(originalRequest)
      } catch {
        // CSRF refresh 也失敗，拋出原始錯誤
      }
    }

    // If 401 and not already retrying, try to refresh token
    // 訪客模式無後端 session，直接略過 refresh 流程，不清除 auth
    // 對 /auth/login & /auth/refresh & /auth/confirm-password 自身的 401 不走 refresh 流程：
    //   - login 401 = 帳密錯，應由 LoginPage 自己 toast「登入失敗」
    //   - refresh 401 = session 已失效，遞迴自己會 deadlock（內層 await 外層的 refreshPromise）
    //   - confirm-password 401 = step-up reauth 密碼輸錯（#380），非 session 過期；不可
    //     refresh+retry（會重送錯密碼、多記一筆 reauth_failure 稽核），交由元件 onError 處理
    const reqUrl = originalRequest?.url ?? ''
    const isAuthSessionEndpoint =
      reqUrl.includes('/auth/login') ||
      reqUrl.includes('/auth/refresh') ||
      reqUrl.includes('/auth/confirm-password')
    if (
      error.response?.status === 401 &&
      !originalRequest?._retry &&
      !isAuthSessionEndpoint &&
      !useAuthStore.getState().isGuest()
    ) {
      originalRequest._retry = true

      // SEC-25: Promise-based singleton — 所有並行 401 共用同一個 refresh Promise
      // 2026-05-20: 進一步用 navigator.locks 跨 tab 序列化（同 origin 多 tab 互斥），
      // 進入 lock 後檢查近期是否已被其他 tab refresh，若是則 skip 自己的 refresh。
      // C1: 使用 attemptRefreshWithRetry 對網路抖動 / 5xx 容錯。
      if (!refreshPromise) {
        refreshPromise = runRefreshUnderCrossTabLock()
          .finally(() => { refreshPromise = null })
      }

      const success = await refreshPromise
      if (success && originalRequest) {
        return api(originalRequest)
      }

      // Refresh 失敗 → 清 auth 後導向 /login?reason=session_expired
      // 2026-05-18: 不再彈紅色 destructive toast，改由 LoginPage 讀 URL param
      // 顯示中性灰色 info toast（讓使用者知道為何被踢，但不像警告）。
      if (!logoutPromise) {
        logoutPromise = new Promise<void>((resolve) => {
          useAuthStore.getState().clearAuth()
          setTimeout(() => {
            logoutPromise = null
            if (!window.location.pathname.startsWith('/login')) {
              window.location.href = '/login?reason=session_expired'
            }
            resolve()
          }, 1000)
        })
      }
    }

    // Non-401 errors: show global toast for server errors and network issues
    // Skip toast if the request opted-in to silent error handling
    const silent = originalRequest?._silentError
    if (!silent) {
      if (error.response) {
        const status = error.response.status
        if (status >= 500) {
          toast({ variant: 'destructive', title: '伺服器錯誤，請稍後再試' })
        }
      } else if (error.code === 'ECONNABORTED') {
        toast({ variant: 'destructive', title: '請求逾時，請檢查網路連線' })
      } else if (!error.response) {
        toast({ variant: 'destructive', title: '無法連線至伺服器' })
      }
    }

    return Promise.reject(error)
  }
)

// ============================================
// SEC-33：敏感操作二級認證
// ============================================
/** 以密碼換取短期 reauth token，供敏感操作 API 帶入 X-Reauth-Token header */
export async function confirmPassword(password: string): Promise<{ reauth_token: string; expires_in: number }> {
  const { data } = await api.post<{ reauth_token: string; expires_in: number }>('/auth/confirm-password', { password })
  return data
}

/** Re-export isAxiosError so consumers don't need to import axios directly */
export { isAxiosError } from 'axios'

export default api
