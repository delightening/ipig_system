import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios'
import { useAuthStore, SESSION_TIMEOUT_MS } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'

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

let isRefreshing = false
let refreshSubscribers: Array<(success: boolean) => void> = []

/** 訂閱 refresh 結果 */
function subscribeTokenRefresh(callback: (success: boolean) => void) {
  refreshSubscribers.push(callback)
}

/** 通知所有等待中的請求 refresh 結果 */
function onRefreshResolved(success: boolean) {
  refreshSubscribers.forEach(cb => cb(success))
  refreshSubscribers = []
}

// 防重複登出鎖：避免多個並行 401 請求同時觸發 logout
let isLoggingOut = false

// Response interceptor - handle errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as typeof error.config & { _retry?: boolean; _503RetryCount?: number; _csrfRetry?: boolean; _silentError?: boolean }

    // 如果已經在登出流程中，直接拒絕所有 401，不再重試
    if (isLoggingOut) {
      return Promise.reject(error)
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
        await api.get('/auth/me')
        return api(originalRequest)
      } catch {
        // CSRF refresh 也失敗，拋出原始錯誤
      }
    }

    // If 401 and not already retrying, try to refresh token
    if (error.response?.status === 401 && !originalRequest?._retry) {
      originalRequest._retry = true

      // SEC-25: 如果已經有 refresh 在進行中，等待其結果
      if (isRefreshing) {
        return new Promise((resolve, reject) => {
          subscribeTokenRefresh((success: boolean) => {
            if (success && originalRequest) {
              resolve(api(originalRequest))
            } else {
              reject(error)
            }
          })
        })
      }

      isRefreshing = true

      try {
        await api.post('/auth/refresh')

        // Reset session expiry timer after successful refresh
        useAuthStore.getState().sessionExpiresAt = Date.now() + SESSION_TIMEOUT_MS

        isRefreshing = false
        onRefreshResolved(true)

        if (originalRequest) {
          return api(originalRequest)
        }
      } catch {
        isRefreshing = false
        onRefreshResolved(false)

        // Refresh 也失敗 → 清除 auth 狀態，讓 React Router 自然導向 /login
        // 使用鎖避免多個並行請求重複觸發
        if (!isLoggingOut) {
          isLoggingOut = true
          try {
            // 使用 getState() 在非 React 上下文存取 store
            const store = useAuthStore.getState()
            // 只清 state，不再呼叫後端 logout（token 已失效）
            store.clearAuth()
          } finally {
            // 延遲重置鎖，讓所有排隊的 401 都被靜默拒絕
            setTimeout(() => { isLoggingOut = false }, 1000)
          }
        }
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

// Utility function to format ear tag: if it's a number < 100, pad to 3 digits
export function formatEarTag(earTag: string): string {
  if (!earTag) return earTag
  // Check if it's a pure number
  if (/^\d+$/.test(earTag)) {
    const num = parseInt(earTag, 10)
    if (num < 100) {
      return earTag.padStart(3, '0')
    }
  }
  return earTag
}

// ============================================
// SEC-33：敏感操作二級認證
// ============================================
/** 以密碼換取短期 reauth token，供敏感操作 API 帶入 X-Reauth-Token header */
export async function confirmPassword(password: string): Promise<{ reauth_token: string; expires_in: number }> {
  const { data } = await api.post<{ reauth_token: string; expires_in: number }>('/auth/confirm-password', { password })
  return data
}

export default api
