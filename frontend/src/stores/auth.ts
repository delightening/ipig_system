import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import api, { User, LoginResponse } from '@/lib/api'

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  isImpersonating: boolean
  impersonate: (userId: string) => Promise<void>
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

      impersonate: async (userId: string) => {
        set({ isLoading: true })
        try {
          // 後端設定新的 HttpOnly Cookie（含模擬登入的 token）
          const response = await api.post<LoginResponse>(`/users/${userId}/impersonate`)
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
          // 登出模擬帳號，使用者需重新以管理員帳號登入
          await api.post('/auth/logout')
          set({
            user: null,
            isAuthenticated: false,
            isImpersonating: false,
            isLoading: false,
          })
          window.location.href = '/login'
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
          })
        } catch {
          set({
            user: null,
            isAuthenticated: false,
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
