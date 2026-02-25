import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import api, { User, LoginResponse } from '@/lib/api'

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  /** 是否已完成初始驗證（防止 stale localStorage state，SEC-24） */
  isInitialized: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  /** 僅清除前端 auth 狀態（不呼叫後端），供 interceptor 在 token 失效時使用 */
  clearAuth: () => void
  isImpersonating: boolean
  /** SEC-33：reauthToken 由呼叫端先透過 POST /auth/confirm-password 取得後傳入 */
  impersonate: (userId: string, reauthToken?: string) => Promise<void>
  stopImpersonating: () => Promise<void>
  checkAuth: () => Promise<void>
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      isInitialized: false,
      // 從 user 資料的 impersonated_by 欄位判斷是否為模擬登入
      isImpersonating: false,

      login: async (email: string, password: string) => {
        set({ isLoading: true })
        try {
          const response = await api.post<LoginResponse>('/auth/login', {
            email,
            password,
          })

          // Cookie 由後端設定，前端不再需要手動儲存 token（SEC-02）
          const { user } = response.data

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: false,
          })
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      logout: async () => {
        try {
          // 後端會清除 HttpOnly Cookie
          await api.post('/auth/logout')
        } catch {
          // Ignore errors during logout
        } finally {
          set({
            user: null,
            isAuthenticated: false,
            isImpersonating: false,
          })
        }
      },

      clearAuth: () => {
        set({
          user: null,
          isAuthenticated: false,
          isImpersonating: false,
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
          console.error('Failed to stop impersonating:', error)
          get().logout()
        }
      },

      checkAuth: async () => {
        try {
          // Cookie 自動傳送，直接呼叫 /me 即可驗證
          const response = await api.get<User>('/me')
          set({
            user: response.data,
            isAuthenticated: true,
            isInitialized: true,
          })
        } catch {
          set({
            user: null,
            isAuthenticated: false,
            isInitialized: true,
          })
        }
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        if (!user) return false
        if (user.roles.includes('admin')) return true
        return user.permissions.includes(permission)
      },

      hasRole: (role: string) => {
        const { user } = get()
        if (!user) return false
        return user.roles.includes(role)
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
        isImpersonating: state.isImpersonating,
      }),
    }
  )
)
