import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

import { getWaitingDays, getWaitingDaysClass } from '@/lib/waitingDays'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { PendingOwner } from '@/types/pendingOwner'

/**
 * 「待 XX」徽章的 hover 內容：這一關卡在誰手上、已經等了幾天。
 *
 * 為什麼要有這個元件：一顆「待核准」徽章回答不了使用者真正的問題——要去催誰。
 * 後端 `services/pending_owner.rs` 依各關卡**真正的授權判準**算出候選人，
 * 這裡只負責呈現，不自己推論誰有權限（推論會與後端分岔，使用者照著催的人點下去拿 403）。
 *
 * 文案形狀依 `kind` 分四種，規則來自 2026-08-26 使用者裁定：
 * - `role`：綁角色 → 角色 + 人員（「倉庫管理員：王大明、李小華 等 5 人」）
 * - `person`：綁特定人員 → 只給人員，不提角色
 * - `applicant`：球在申請人身上（補件），關卡名稱已表明，只給人名
 * - `anonymous`：IACUC 委員會審查 → **對所有人一律不列名**，只給人數
 */

/** 姓名清單的顯示字串；`role` / `person` / `applicant` 共用。 */
function useOwnerNames(owner: PendingOwner): string {
    const { t } = useTranslation()
    if (owner.kind === 'anonymous') {
        return t('pendingOwner.reviewerCount', { count: owner.overflow })
    }
    if (owner.candidates.length === 0) {
        // 候選人被 SoD 排空（例：唯一的倉管就是建單者）。這是有意義的資訊，
        // 不能顯示成空白——那會讓卡死的單看起來跟正常待審的單一樣。
        return t('pendingOwner.nobody')
    }
    const names = owner.candidates.join('、')
    if (owner.overflow <= 0) return names
    const total = owner.candidates.length + owner.overflow
    return `${names} ${t('pendingOwner.andOthers', { count: total })}`
}

/** 「誰」那一行的完整文字（含角色前綴，若該關卡綁角色）。 */
function useOwnerLine(owner: PendingOwner): string {
    const { t } = useTranslation()
    const names = useOwnerNames(owner)
    const showsRole =
        (owner.kind === 'role' || owner.kind === 'anonymous') && owner.role_code !== null
    if (!showsRole) return names
    return `${t(`pendingOwner.role.${owner.role_code}`)}：${names}`
}

function PendingOwnerBody({ owner }: { owner: PendingOwner }) {
    const { t } = useTranslation()
    const line = useOwnerLine(owner)
    const days = getWaitingDays(owner.since)
    return (
        <>
            <div className="font-medium">{t(`pendingOwner.stage.${owner.stage}`)}</div>
            <div className="mt-0.5">{line}</div>
            {days !== null && (
                <div className={`mt-0.5 ${getWaitingDaysClass(days)}`}>
                    {t('common.waitingDays', { count: days })}
                </div>
            )}
        </>
    )
}

interface PendingOwnerBadgeProps {
    /** 後端算出的待處理人；`null` / `undefined`（非在途狀態）時原樣渲染 children，不加 tooltip */
    owner: PendingOwner | null | undefined
    /** 要被包起來的狀態徽章 */
    children: ReactNode
}

/**
 * 用 tooltip 包住狀態徽章。
 *
 * ⚠️ **必須用 `asChild` + `<div>` 包裝**，兩者缺一不可（CodeRabbit 於 PR #30 指出）：
 *
 * - 不加 `asChild`：Radix 自己渲染一顆 `<button>` 包住 children，而 `Badge`
 *   （`components/ui/badge.tsx:36`）渲染的是 `<div>`——`<button><div>` 是無效巢狀。
 * - 包裝元素**不能用 `<span>`**：`<span>` 是 phrasing content，一樣容不下 `<div>`。
 *   `<div>` 則同時容得下 `Badge` 的 `<div>` 與 `StatusBadge` 的 `<span>`，
 *   配 `inline-flex` 維持行內排版。
 *
 * `tabIndex={0}` + `role="button"` 補回原本「鍵盤可達、不是純 hover-only」的行為
 * （Radix 預設 trigger 是 button，改 `asChild` 後那個特性要自己帶）。
 *
 * ⚠️ 手機沒有 hover，窄版面請改用 [`PendingOwnerInline`] 直接寫在版面上。
 */
export function PendingOwnerBadge({ owner, children }: PendingOwnerBadgeProps) {
    if (!owner) return <>{children}</>
    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <div className="cursor-help align-middle inline-flex" tabIndex={0} role="button">
                    {children}
                </div>
            </TooltipTrigger>
            <TooltipContent>
                <PendingOwnerBody owner={owner} />
            </TooltipContent>
        </Tooltip>
    )
}

function PendingOwnerInlineBody({ owner }: { owner: PendingOwner }) {
    const { t } = useTranslation()
    const line = useOwnerLine(owner)
    const days = getWaitingDays(owner.since)
    return (
        <div>
            <span className="text-muted-foreground">
                {t(`pendingOwner.stage.${owner.stage}`)}：
            </span>
            {line}
            {days !== null && (
                <>
                    {' · '}
                    <span className={getWaitingDaysClass(days)}>
                        {t('common.waitingDays', { count: days })}
                    </span>
                </>
            )}
        </div>
    )
}

/** 窄容器 / 卡片版：不靠 hover，直接把「卡在誰、等了幾天」寫在版面上。 */
export function PendingOwnerInline({ owner }: { owner: PendingOwner | null | undefined }) {
    if (!owner) return null
    return <PendingOwnerInlineBody owner={owner} />
}
