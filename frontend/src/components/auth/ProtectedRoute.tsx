import { Navigate } from 'react-router-dom'

import { useAuthIsAuthenticated, useAuthIsInitialized, useAuthUser, useAuthIsGuest } from '@/stores/auth'
import { useHeartbeat } from '@/hooks/useHeartbeat'
import { useProactiveRefresh } from '@/hooks/useProactiveRefresh'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
    const isAuthenticated = useAuthIsAuthenticated()
    const isInitialized = useAuthIsInitialized()
    const user = useAuthUser()
    const isGuest = useAuthIsGuest()

    // 啟動 heartbeat 監聽使用者活動（須等 checkAuth 完成，避免 stale token 觸發 401）
    // 訪客模式無後端 session，跳過 heartbeat 避免觸發 401 → clearAuth 流程
    const isActiveUser = isAuthenticated && isInitialized && !isGuest
    useHeartbeat(isActiveUser)
    // A2: 在 access token 過期前主動 refresh，消除 reactive 401 → refresh 的卡頓
    useProactiveRefresh(isActiveUser)

    // SEC-24: 等待初始驗證完成，防止 stale localStorage state
    if (!isInitialized) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        )
    }

    if (!isAuthenticated) {
        return <Navigate to="/login" replace />
    }

    // 首次登入強制變更密碼
    if (user?.must_change_password) {
        return <Navigate to="/force-change-password" replace />
    }

    return <>{children}</>
}
