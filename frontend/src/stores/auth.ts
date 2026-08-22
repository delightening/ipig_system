import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { useShallow } from 'zustand/react/shallow'
import { AxiosError } from 'axios'
import api, { User, LoginResponse, TwoFactorRequiredResponse } from '@/lib/api'
import { attemptRefreshWithRetry } from '@/lib/api/client'
import { logger } from '@/lib/logger'
import { broadcastAuth, onAuthBroadcast } from '@/lib/authBroadcast'
import { isTabIdle } from '@/lib/tabActivity'

export const expiresInToTimestamp = (expiresInSeconds: number): number =>
  Date.now() + expiresInSeconds * 1000

/**
 * 清除認證狀態的共用物件（DRY）。由 `logout`、`clearAuth`、B1 broadcast handler
 * 三處使用。避免重複定義導致欄位漏新增（例如未來加新欄位時某一處忘了清）。
 * 來源：Gemini Code Assist review on PR #428 (medium)。
 *
 * 2026-05-18: 移除 sessionExpiresAt — 前端不再做 time-based session 倒數，
 * 改由後端 cleanup_expired cron + 401 interceptor 處理。
 */
const CLEARED_AUTH_STATE = {
  user: null,
  isAuthenticated: false,
  isImpersonating: false,
  accessTokenExpiresAt: null,
} as const

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  /** 是否已完成初始驗證（防止 stale localStorage state，SEC-24） */
  isInitialized: boolean
  /**
   * Access token (JWT) 到期的 Unix 時間戳（ms），由 login/refresh response.expires_in 推導。
   * 供 useProactiveRefresh 在 80% TTL silent refresh，消除 reactive 401 → refresh 延遲。
   * 不持久化（session-scoped + 避免 XSS 取得 timing 資訊）。
   */
  accessTokenExpiresAt: number | null
  login: (email: string, password: string) => Promise<void>
  verify2FA: (tempToken: string, code: string) => Promise<void>
  logout: () => Promise<void>
  /** 純前端訪客模式：建立本地假用戶，完全不呼叫後端 */
  enterGuestMode: () => void
  /** 僅清除前端 auth 狀態（不呼叫後端），供 interceptor 在 token 失效時使用 */
  clearAuth: () => void
  /** 清除本 Tab auth 狀態，不廣播給其他分頁（per-tab idle logout 專用） */
  clearAuthLocal: () => void
  isImpersonating: boolean
  /** SEC-33：reauthToken 由呼叫端先透過 POST /auth/confirm-password 取得後傳入 */
  impersonate: (userId: string, reauthToken?: string) => Promise<void>
  stopImpersonating: () => Promise<void>
  checkAuth: () => Promise<void>
  refreshSession: () => Promise<boolean>
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
  /** 與後端 `CurrentUser::is_admin()` 同語意：SYSTEM_ADMIN 或 legacy `admin` 皆為管理員 */
  isAdmin: () => boolean
  isGuest: () => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      isInitialized: false,
      accessTokenExpiresAt: null,
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

          const { user, expires_in } = response.data as LoginResponse

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: false,
            accessTokenExpiresAt: expiresInToTimestamp(expires_in),
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
          const { user, expires_in } = response.data
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: false,
            accessTokenExpiresAt: expiresInToTimestamp(expires_in),
          })
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      enterGuestMode: () => {
        // 訪客模式不需要追蹤用途的 cookie，自動接受必要性 cookie 避免 banner 彈出
        if (!localStorage.getItem('cookie-consent')) {
          localStorage.setItem('cookie-consent', 'essential')
        }
        set({
          user: {
            id: 'guest',
            display_name: '訪客',
            email: 'guest@guest.com',
            roles: ['GUEST'],
            permissions: [],
            is_active: true,
          } as User,
          isAuthenticated: true,
          isInitialized: true,
          isLoading: false,
          accessTokenExpiresAt: null,
        })
      },

      logout: async () => {
        if (get().isGuest()) {
          get().clearAuth()
          return
        }
        try {
          await api.post('/auth/logout')
        } catch {
          // Ignore errors during logout
        } finally {
          set({ ...CLEARED_AUTH_STATE })
          // B1: 廣播給其他分頁，讓它們同步登出狀態
          broadcastAuth({ type: 'cleared' })
        }
      },

      clearAuth: () => {
        set({ ...CLEARED_AUTH_STATE })
        // B1: 廣播給其他分頁，token 失效一次清乾淨
        broadcastAuth({ type: 'cleared' })
      },

      clearAuthLocal: () => {
        set({ ...CLEARED_AUTH_STATE })
      },

      impersonate: async (userId: string, reauthToken?: string) => {
        set({ isLoading: true })
        try {
          const config = reauthToken ? { headers: { 'X-Reauth-Token': reauthToken } } : undefined
          const response = await api.post<LoginResponse>(`/users/${userId}/impersonate`, {}, config)
          const { user, expires_in } = response.data

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            isImpersonating: true,
            accessTokenExpiresAt: expiresInToTimestamp(expires_in),
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
          const { user, expires_in } = response.data

          set({
            user,
            isAuthenticated: true,
            isImpersonating: false,
            isLoading: false,
            accessTokenExpiresAt: expiresInToTimestamp(expires_in),
          })

          // 重新載入頁面以清除所有 query cache 和重置狀態
          window.location.href = '/'
        } catch (error) {
          logger.error('Failed to stop impersonating:', error)
          get().logout()
        }
      },

      checkAuth: async () => {
        // 訪客模式：不呼叫後端，直接標記為已初始化
        if (get().isGuest()) {
          set({ isInitialized: true })
          return
        }
        try {
          const response = await api.get<User>('/me')
          set({
            user: response.data,
            isAuthenticated: true,
            isInitialized: true,
          })
        } catch (err) {
          // 僅在 401（未認證）時清除 auth；503/5xx/網路錯誤保留登入狀態，避免伺服器暫時不可用時誤登出
          const is401 = err instanceof AxiosError && err.response?.status === 401
          set({
            ...(is401
              ? { user: null, isAuthenticated: false }
              : {}),
            isInitialized: true,
          })
        }
      },

      refreshSession: async () => {
        try {
          // C1: 共用 attemptRefreshWithRetry，網路抖動 / 5xx 自動 retry 一次
          const response = await attemptRefreshWithRetry()
          const accessTokenExpiresAt = expiresInToTimestamp(response.data.expires_in)
          set({
            accessTokenExpiresAt,
          })
          // B1: 廣播給其他分頁，讓它們 reschedule proactive refresh timer
          // 而不需要各自呼叫一次 /auth/refresh（避免 R46 race window 噪音）
          broadcastAuth({ type: 'refreshed', accessTokenExpiresAt })
          return true
        } catch {
          return false
        }
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        if (!user) return false
        if (user.roles.includes('GUEST')) return true
        // SYSTEM_ADMIN 與舊版 admin 皆為超級管理員，比照後端 is_admin() 短路
        // （middleware/auth.rs::has_permission 對 is_admin 一律放行；兩者無顯式 permission grant）。
        if (user.roles.includes('admin') || user.roles.includes('SYSTEM_ADMIN')) return true
        return user.permissions.includes(permission)
      },

      hasRole: (role: string) => {
        const { user } = get()
        if (!user) return false
        return user.roles.includes(role)
      },

      // 比照後端 middleware/auth.rs::is_admin()：SYSTEM_ADMIN 與 legacy admin 皆為管理員。
      // 前端授權需與後端同語意，避免「按鈕顯示但後端擋下」或反之的不一致。
      isAdmin: () => {
        const { user } = get()
        if (!user) return false
        return user.roles.includes('admin') || user.roles.includes('SYSTEM_ADMIN')
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
      // R34-20：persist version + migrate 範本。
      //
      // 之前缺 version → 任何 partialize 結構或型別變動都會直接套入舊資料，
      // 可能讓既有使用者 session 出現未定義欄位 / 強制重登。
      //
      // 目前 version=1 為 baseline；下次動到 partialize shape 或新增 sensitive
      // 欄位時：(a) bump 為 2 (b) 在 migrate 增加 v1→v2 case 處理舊 payload。
      //
      // 範本：
      //   if (version === 1) {
      //     // v1 → v2: e.g. drop legacy field, rename, etc.
      //     return { ...persisted, newField: defaultValue }
      //   }
      //
      // 任何 schema 改動都應同步 bump version；忘了 bump 等於不做。
      version: 1,
      // 目前無歷史版本需要 migrate；保留型別注解讓 TypeScript 推斷不破。
      // 下次 bump 為 2 時：在這裡 if (version === 1) { ... transform persisted ... }
      migrate: (persisted, _version) => persisted,
    }
  )
)

// B1: 訂閱來自其他分頁的 auth 廣播，本地直接 setState（不呼叫 store action
// 避免 re-broadcast 形成 ping-pong）。模組載入時註冊一次，整個 SPA 生命週期共用。
//
// HMR guard：開發環境 Vite HMR 重新執行此模組，module-scoped flag 也會被重置 →
// 改用 window 持久化 flag，跨 HMR cycle 仍保留（CodeRabbit nitpick on PR #428）。
// 副效應：舊 BroadcastChannel listener 在 HMR 後仍掛著，但因新模組跳過註冊，
// 不會出現 N+1 listener；且每次 HMR cycle 之間舊 listener 仍指向舊 useAuthStore
// 實例，這在開發環境是可接受的微行為偏差。
const HMR_GUARD_KEY = '__slidingSessionAuthSubscribed' as const
type GuardedWindow = Window & { [HMR_GUARD_KEY]?: boolean }
if (typeof window !== 'undefined' && !(window as GuardedWindow)[HMR_GUARD_KEY]) {
  ;(window as GuardedWindow)[HMR_GUARD_KEY] = true
  onAuthBroadcast((msg) => {
    if (msg.type === 'refreshed') {
      if (isTabIdle()) {
        useAuthStore.setState({ ...CLEARED_AUTH_STATE })
        return
      }
      const current = useAuthStore.getState().accessTokenExpiresAt
      if (current == null || msg.accessTokenExpiresAt > current) {
        useAuthStore.setState({ accessTokenExpiresAt: msg.accessTokenExpiresAt })
      }
    } else if (msg.type === 'cleared') {
      // 直接 setState 不呼叫 clearAuth()，避免再次 broadcast
      useAuthStore.setState({ ...CLEARED_AUTH_STATE })
    }
  })
}

// ─── Optimised selectors ───
// Components that only need a subset of the store should use these
// to avoid re-rendering when unrelated state changes.

export const useAuthUser = () => useAuthStore((s) => s.user)
export const useAuthIsAuthenticated = () => useAuthStore((s) => s.isAuthenticated)
export const useAuthIsInitialized = () => useAuthStore((s) => s.isInitialized)
// 訂閱 user state（user 變動時組件 rerender），回傳 store stable 函式 ref（內部 get() 讀最新 user）。
// 採此模式而非 closure useCallback：保留呼叫端 `const hasRole = useAuthHasRole(); hasRole('admin')` 的 API
// 不破壞既有 3 處 callers (ReportsPage/AuditLogsPage/DocumentDetailPage)，且避免 user 物件 reference
// 不穩定造成的多餘 rerender。
//
// 來源：PR #400 gemini code review (medium)
export const useAuthHasRole = () => {
  useAuthStore((s) => s.user)
  return useAuthStore.getState().hasRole
}
// admin 判定（SYSTEM_ADMIN || legacy admin，比照後端 is_admin()）。回傳布林、於角色變動時 rerender。
export const useAuthIsAdmin = () =>
  useAuthStore((s) => !!s.user && (s.user.roles.includes('admin') || s.user.roles.includes('SYSTEM_ADMIN')))
export const useAuthHasPermission = () => {
  useAuthStore((s) => s.user)
  return useAuthStore.getState().hasPermission
}
export const useAuthIsGuest = () => useAuthStore((s) => s.isGuest())
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
      enterGuestMode: s.enterGuestMode,
    })),
  )
