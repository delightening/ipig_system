import { Outlet } from 'react-router-dom'

import { useAuthStore } from '@/stores/auth'

import { RequirePermission } from './RequirePermission'

export function AdminRoute({ children }: { children?: React.ReactNode }) {
    const { hasRole, isGuest } = useAuthStore()

    // Guest 全通行
    if (isGuest()) return children ? <>{children}</> : <Outlet />

    if (!hasRole('admin')) {
        return (
            <RequirePermission role="admin">
                {children || <Outlet />}
            </RequirePermission>
        )
    }

    return children ? <>{children}</> : <Outlet />
}
