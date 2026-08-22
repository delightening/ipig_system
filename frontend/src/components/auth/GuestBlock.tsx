import { Navigate } from 'react-router-dom'

import { useAuthIsGuest } from '@/stores/auth'

/**
 * 對 guest 完全擋下的 route wrapper（編輯/送審類頁面）。
 *
 * 與 `RequirePermission guestBlock` 不同：這個給「整條 route 邏輯都不適合 demo」的場景用。
 * 例如表單編輯頁 — guest 模式既不能存也不能 submit，整頁就沒意義。
 *
 * 行為：guest → redirect 到 `/dashboard`；非 guest → 渲染 children。
 */
export function GuestBlock({ children }: { children: React.ReactNode }) {
    const isGuest = useAuthIsGuest()
    if (isGuest) return <Navigate to="/dashboard" replace />
    return <>{children}</>
}
