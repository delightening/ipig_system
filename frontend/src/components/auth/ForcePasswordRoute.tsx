import { Navigate } from 'react-router-dom'

import { useAuthStore } from '@/stores/auth'

export function ForcePasswordRoute({ children }: { children: React.ReactNode }) {
    const { isAuthenticated, user } = useAuthStore()

    if (!isAuthenticated) {
        return <Navigate to="/login" replace />
    }

    // 如果已變更密碼，導向 dashboard
    if (!user?.must_change_password) {
        return <Navigate to="/dashboard" replace />
    }

    return <>{children}</>
}
