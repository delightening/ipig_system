import type { ReactNode } from 'react'

import { useAuthHasPermission } from '@/stores/auth'
import type { PermissionCode } from '@/lib/permissions.generated'

interface CanProps {
    children: ReactNode
    /** 需要的權限代碼（單一） */
    permission?: PermissionCode
    /** 任一符合即可 */
    anyOf?: PermissionCode[]
    /** 全部符合才可 */
    allOf?: PermissionCode[]
    /**
     * 無權限時要渲染什麼。**預設 `null`＝完全不渲染**。
     *
     * 使用者 2026-08-07 裁定：無權限一律「完全隱藏」，不用 disabled + tooltip。
     * 只有在「整頁唯一的主要動作、藏掉會讓人以為功能壞了」時才傳 fallback 給說明文字，
     * 且不要傳一顆 disabled 的按鈕——那等於把全部功能清單公開給每個使用者。
     */
    fallback?: ReactNode
}

/**
 * 動作層權限閘：無權限時不渲染 children。
 *
 * 與 [`RequirePermission`](./RequirePermission.tsx) 的分工：
 * - `RequirePermission` 守**路由 / 整頁**，無權限時導頁或顯示「無權限訪問」卡片
 * - `Can` 守**單一動作**（按鈕、選單項、inline 編輯），無權限時安靜地不存在
 *
 * ## 為什麼要有這個元件
 *
 * 2026-08-07 稽核（`docs/audit/button-permission-gate-2026-08-07.md`）：
 * 117 個會發 mutation 的前端檔中有 73 個完全沒有任何權限判斷。後端擋得住，
 * 所以不是安全漏洞，但使用者看得到按鈕、按下去吃 403；而 `response_logger`
 * 對所有 403 無差別計數，連點幾次就可能撞到 IP 封鎖。
 *
 * 根因之一是「沒有共用的動作閘元件」——每個地方各自手寫
 * `hasPermission('...') && <Button>`，字串手打、容易漏、也沒有統一的測試點。
 *
 * ## 用法
 *
 * ```tsx
 * import { Can } from '@/components/auth'
 * import { PERMISSIONS } from '@/lib/permissions.generated'
 *
 * <Can permission={PERMISSIONS.ANIMAL_RECORD_CREATE}>
 *   <Button onClick={handleCreate}>新增觀察紀錄</Button>
 * </Can>
 * ```
 *
 * 權限字串請一律用 `PERMISSIONS` 常數，不要手打字串——`PermissionCode` 型別
 * 由後端 permissions 表產生，打錯字會在 `tsc` 階段就被擋下（見
 * `lib/permissions.generated.ts` 檔頭）。
 *
 * ## GUEST
 *
 * 判定走 `hasPermission()`，它對 GUEST 一律回 `true`（demo 模式的刻意設計，
 * 見 `GUEST_DEMO_ARCHITECTURE.md`）。因此 guest demo 的畫面不會因為補閘而變空。
 * **不要**繞過 `hasPermission()` 自己判角色，那會把 guest 的 demo 打壞。
 */
export function Can({ children, permission, anyOf, allOf, fallback = null }: CanProps) {
    const hasPermission = useAuthHasPermission()

    // 三種條件皆未指定 = 無條件放行（讓呼叫端可以用同一個 wrapper 表達「這顆不需要權限」）
    if (!permission && !anyOf?.length && !allOf?.length) return <>{children}</>

    const ok =
        (permission ? hasPermission(permission) : true) &&
        (anyOf?.length ? anyOf.some((p) => hasPermission(p)) : true) &&
        (allOf?.length ? allOf.every((p) => hasPermission(p)) : true)

    return ok ? <>{children}</> : <>{fallback}</>
}

export default Can
