import { Navigate, Outlet } from 'react-router-dom'

import { useAuthStore } from '@/stores/auth'

export const DASHBOARD_ROLES = ['purchasing', 'approver', 'WAREHOUSE_MANAGER', 'EXPERIMENT_STAFF', 'INTERN', 'REVIEWER', 'VET', 'IACUC_CHAIR']

export function DashboardRoute({ children }: { children?: React.ReactNode }) {
    const { user, hasRole } = useAuthStore()

    const hasDashboardAccess = hasRole('admin') ||
        user?.roles.some(r => DASHBOARD_ROLES.includes(r)) ||
        user?.permissions.some(p => p.startsWith('erp.'))

    if (!hasDashboardAccess) {
        return <Navigate to="/my-projects" replace />
    }

    return children ? <>{children}</> : <Outlet />
}
