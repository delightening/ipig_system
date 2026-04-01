import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { useShallow } from 'zustand/react/shallow'
import { AxiosError } from 'axios'
import api, { User, LoginResponse, TwoFactorRequiredResponse } from '@/lib/api'
import { logger } from '@/lib/logger'

/** 前端 Session 逾時（6 小時），供 SessionTimeoutWarning 使用 */
export const SESSION_TIMEOUT_MS = 6 * 60 * 60 * 1000

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  /** 是否已完成初始驗證（防止 stale localStorage state，SEC-24） */
  isInitialized: boolean
  /** Session 到期的 Unix 時間戳（ms），供逾時預警使用 */
  sessionExpiresAt: number | null
  login: (email: string, password: string) => Promise<void>
  verify2FA: (tempToken: string, code: string) => Promise<void>
  logout: () => Promise<void>
  /** 僅清除前端 auth 狀態（不呼叫後端），供 interceptor 在 token 失效時使用 */
  clearAuth: () => void
  isImpersonating: boolean
  /** SEC-33：reauthToken 由呼叫端先透過 POST /auth/confirm-password 取得後傳入 */
  impersonate: (userId: string, reauthToken?: string) => Promise<void>
  stopImpersonating: () => Promise<void>
  checkAuth: () => Promise<void>
  refreshSession: () => Promise<boolean>
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
  isGuest: () => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      isInitialized: false,
      sessionExpiresAt: null,
      isImpersonating: false,

      login: async (email: string, password: string) => {
        set({ isLoading: true })
        try {
          const response = await api.post<LoginResponse | TwoFactorRequiredResponse>('/auth/login', {
            email,
            password,
          })

          if ('requires_2fa' in response.data && response.data.requires_2fa) {
            set({ isLoading: false })
            throw { is2FA: true, tempToken: (response.data as TwoFactorRequiredResponse).temp_token }
          }

          const { user } = response.data as LoginResponse

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: false,
            sessionExpiresAt: Date.now() + SESSION_TIMEOUT_MS,
          })
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      verify2FA: async (tempToken: string, code: string) => {
        set({ isLoading: true })
        try {
          const response = await api.post<LoginResponse>('/auth/2fa/verify', {
            temp_token: tempToken,
            code,
          })
          const { user } = response.data
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: false,
            sessionExpiresAt: Date.now() + SESSION_TIMEOUT_MS,
          })
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      logout: async () => {
        try {
          await api.post('/auth/logout')
        } catch {
          // Ignore errors during logout
        } finally {
          set({
            user: null,
            isAuthenticated: false,
            isImpersonating: false,
            sessionExpiresAt: null,
          })
        }
      },

      clearAuth: () => {
        set({
          user: null,
          isAuthenticated: false,
          isImpersonating: false,
          sessionExpiresAt: null,
        })
      },

      impersonate: async (userId: string, reauthToken?: string) => {
        set({ isLoading: true })
        try {
          const config = reauthToken ? { headers: { 'X-Reauth-Token': reauthToken } } : undefined
          const response = await api.post<LoginResponse>(`/users/${userId}/impersonate`, {}, config)
          const { user } = response.data

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: true,
          })

          // Force a full page reload to reset all query cache and states
          window.location.href = '/'
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      stopImpersonating: async () => {
        set({ isLoading: true })
        try {
          // 呼叫後端恢復管理員 session（後端會重新簽發管理員 token）
          const response = await api.post<LoginResponse>('/auth/stop-impersonate')
          const { user } = response.data

          set({
            user,
            isAuthenticated: true,
            isImpersonating: false,
            isLoading: false,
          })

          // 重新載入頁面以清除所有 query cache 和重置狀態
          window.location.href = '/'
        } catch (error) {
          logger.error('Failed to stop impersonating:', error)
          get().logout()
        }
      },

      checkAuth: async () => {
        try {
          const response = await api.get<User>('/me')
          const currentExpiry = get().sessionExpiresAt
          set({
            user: response.data,
            isAuthenticated: true,
            isInitialized: true,
            sessionExpiresAt: currentExpiry ?? Date.now() + SESSION_TIMEOUT_MS,
          })
        } catch (err) {
          // 僅在 401（未認證）時清除 auth；503/5xx/網路錯誤保留登入狀態，避免伺服器暫時不可用時誤登出
          const is401 = err instanceof AxiosError && err.response?.status === 401
          set({
            ...(is401
              ? { user: null, isAuthenticated: false, sessionExpiresAt: null }
              : {}),
            isInitialized: true,
          })
        }
      },

      refreshSession: async () => {
        try {
          await api.post('/auth/refresh')
          set({ sessionExpiresAt: Date.now() + SESSION_TIMEOUT_MS })
          return true
        } catch {
          return false
        }
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        if (!user) return false
        if (user.roles.includes('GUEST')) return true
        if (user.roles.includes('admin')) return true
        return user.permissions.includes(permission)
      },

      hasRole: (role: string) => {
        const { user } = get()
        if (!user) return false
        return user.roles.includes(role)
      },

      isGuest: () => {
        const { user } = get()
        return user?.roles.includes('GUEST') ?? false
      },
    }),
    {
      name: 'auth-storage',
      // SEC-C5: 只持久化最小必要資訊，避免 XSS 時洩漏個人資料（phone、email 等）
      // 完整 user 物件由 checkAuth() 從後端重新取得
      partialize: (state) => ({
        user: state.user ? {
          id: state.user.id,
          display_name: state.user.display_name,
          roles: state.user.roles,
          permissions: state.user.permissions,
          is_active: state.user.is_active,
        } : null,
        isImpersonating: state.isImpersonating,
      }),
    }
  )
)

// ─── Optimised selectors ───
// Components that only need a subset of the store should use these
// to avoid re-rendering when unrelated state changes.

export const useAuthUser = () => useAuthStore((s) => s.user)
export const useAuthIsAuthenticated = () => useAuthStore((s) => s.isAuthenticated)
export const useAuthIsInitialized = () => useAuthStore((s) => s.isInitialized)
export const useAuthHasRole = () => useAuthStore((s) => s.hasRole)
export const useAuthHasPermission = () => useAuthStore((s) => s.hasPermission)
export const useAuthIsGuest = () => useAuthStore((s) => s.isGuest)
export const useAuthActions = () =>
  useAuthStore(
    useShallow((s) => ({
      login: s.login,
      logout: s.logout,
      verify2FA: s.verify2FA,
      checkAuth: s.checkAuth,
      refreshSession: s.refreshSession,
      impersonate: s.impersonate,
      stopImpersonating: s.stopImpersonating,
      clearAuth: s.clearAuth,
    })),
  )
