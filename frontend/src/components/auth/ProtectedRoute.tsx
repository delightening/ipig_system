import { Navigate } from 'react-router-dom'

import { useAuthStore } from '@/stores/auth'
import { useHeartbeat } from '@/hooks/useHeartbeat'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
    const { isAuthenticated, isInitialized, user } = useAuthStore()

    // 啟動 heartbeat 監聽使用者活動
    useHeartbeat(isAuthenticated)

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
