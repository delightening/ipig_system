import { Navigate, Outlet } from 'react-router-dom'

import { useAuthHasPermission, useAuthIsGuest } from '@/stores/auth'

/**
 * 是否可進入儀表板。
 *
 * 判準是 `dashboard.view` 權限，不是 role 清單。原本硬編
 * `['purchasing','approver','WAREHOUSE_MANAGER','EXPERIMENT_STAFF','INTERN','REVIEWER','VET','IACUC_CHAIR']`
 * 加上「任一權限以 `erp.` 開頭」的前綴 hack，兩者都有問題：
 *
 * - `purchasing`、`approver` 是**不存在的 role code**（實際是 `PURCHASING`，且沒有 `approver`）。
 *   PURCHASING 使用者今天能進來，靠的是 `erp.` 前綴那條漏網，不是這份清單。
 * - seed 把 `dashboard.view` 授予 15 個角色，前端清單只認 8 個中的 6 個有效者
 *   —— 宣告的意圖與實作長期不一致。
 *
 * 改用 permission 後兩個問題一起消失，日後把 `dashboard.view` 授予新角色也不必再改前端。
 * `hasPermission` 對 admin 與 GUEST 皆短路放行，行為與原本一致。
 */
// eslint-disable-next-line react-refresh/only-export-components
export function useHasDashboardAccess(): boolean {
    return useAuthHasPermission()('dashboard.view')
}

export function DashboardRoute({ children }: { children?: React.ReactNode }) {
    const isGuest = useAuthIsGuest()
    const hasDashboardAccess = useHasDashboardAccess()

    // Guest 全通行（hasPermission 對 GUEST 亦回 true，這行是明示意圖）
    if (isGuest) return children ? <>{children}</> : <Outlet />

    if (!hasDashboardAccess) {
        return <Navigate to="/my-projects" replace />
    }

    return children ? <>{children}</> : <Outlet />
}
