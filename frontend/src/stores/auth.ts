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
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      isImpersonating: !!localStorage.getItem('admin_access_token'),

      login: async (email: string, password: string) => {
        set({ isLoading: true })
        try {
          const response = await api.post<LoginResponse>('/auth/login', {
            email,
            password,
          })

          const { access_token, refresh_token, user } = response.data

          localStorage.setItem('access_token', access_token)
          localStorage.setItem('refresh_token', refresh_token)

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
          await api.post('/auth/logout')
        } catch {
          // Ignore errors during logout
        } finally {
          localStorage.removeItem('access_token')
          localStorage.removeItem('refresh_token')
          localStorage.removeItem('admin_access_token')
          localStorage.removeItem('admin_refresh_token')
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
          // Save current admin tokens if not already impersonating
          if (!get().isImpersonating) {
            const adminToken = localStorage.getItem('access_token')
            const adminRefresh = localStorage.getItem('refresh_token')
            if (adminToken) localStorage.setItem('admin_access_token', adminToken)
            if (adminRefresh) localStorage.setItem('admin_refresh_token', adminRefresh)
          }

          const response = await api.post<LoginResponse>(`/users/${userId}/impersonate`)
          const { access_token, refresh_token, user } = response.data

          localStorage.setItem('access_token', access_token)
          localStorage.setItem('refresh_token', refresh_token)

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
          const adminToken = localStorage.getItem('admin_access_token')
          const adminRefresh = localStorage.getItem('admin_refresh_token')

          if (adminToken && adminRefresh) {
            localStorage.setItem('access_token', adminToken)
            localStorage.setItem('refresh_token', adminRefresh)
            localStorage.removeItem('admin_access_token')
            localStorage.removeItem('admin_refresh_token')

            // Fetch admin user data
            const response = await api.get<User>('/me')
            set({
              user: response.data,
              isAuthenticated: true,
              isLoading: false,
              isImpersonating: false,
            })

            window.location.href = '/'
          } else {
            // If no admin token found, just logout
            get().logout()
          }
        } catch (error) {
          console.error('Failed to stop impersonating:', error)
          get().logout()
        }
      },

      checkAuth: async () => {
        const token = localStorage.getItem('access_token')
        if (!token) {
          set({ user: null, isAuthenticated: false })
          return
        }

        try {
          const response = await api.get<User>('/me')
          set({
            user: response.data,
            isAuthenticated: true,
          })
        } catch {
          localStorage.removeItem('access_token')
          localStorage.removeItem('refresh_token')
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
      }),
    }
  )
)
