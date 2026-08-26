import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen } from '@testing-library/react'

import type { PendingOwner } from '@/types/pendingOwner'

/**
 * 鎖住 2026-08-26 使用者裁定的四種文案形狀：
 * - 綁角色 → 角色 + 人員
 * - 綁特定人員 → 只有人員，不提角色
 * - 委員會審查（anonymous）→ **一律不列名**，只給人數
 * - 候選人被職務分離排空 → 明講「無人可處理」，不是空白
 *
 * i18n：t() 回 key 本身，帶 count 時附在後面；斷言比對 key 不比對文案，
 * 文案改寫不會弄紅這支測試。
 * ⚠️ 必須一併導出 initReactI18next——`lib/utils` 會拉進 `lib/i18n`。
 */
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, opts?: Record<string, unknown>) =>
            opts && 'count' in opts ? `${key}#${opts.count}` : key,
    }),
    initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { PendingOwnerBadge, PendingOwnerInline } = await import('../PendingOwnerBadge')

const base: PendingOwner = {
    stage: 'doc_wm_approve',
    kind: 'role',
    role_code: 'WAREHOUSE_MANAGER',
    candidates: [],
    overflow: 0,
    since: null,
}

describe('PendingOwnerInline 文案形狀', () => {
    it('綁角色：顯示角色與人員', () => {
        render(
            <PendingOwnerInline
                owner={{ ...base, candidates: ['王大明', '李小華'] }}
            />
        )
        expect(
            screen.getByText(/pendingOwner\.role\.WAREHOUSE_MANAGER：王大明、李小華/)
        ).toBeInTheDocument()
    })

    it('人數超過 3 位時附總人數，且列出的名字只有 3 個', () => {
        render(
            <PendingOwnerInline
                owner={{ ...base, candidates: ['甲', '乙', '丙'], overflow: 2 }}
            />
        )
        // overflow 的 count 是「總人數」不是「剩餘人數」——3 + 2 = 5
        expect(screen.getByText(/甲、乙、丙 pendingOwner\.andOthers#5/)).toBeInTheDocument()
    })

    it('綁特定人員：只給人名，不出現角色前綴', () => {
        render(
            <PendingOwnerInline
                owner={{ ...base, kind: 'person', role_code: null, candidates: ['張三'] }}
            />
        )
        expect(screen.getByText(/張三/)).toBeInTheDocument()
        expect(screen.queryByText(/pendingOwner\.role\./)).not.toBeInTheDocument()
    })

    it('委員會審查：一律不列名，只給人數', () => {
        render(
            <PendingOwnerInline
                owner={{
                    ...base,
                    stage: 'aup_under_review',
                    kind: 'anonymous',
                    role_code: 'REVIEWER',
                    candidates: [],
                    overflow: 3,
                }}
            />
        )
        expect(
            screen.getByText(/pendingOwner\.role\.REVIEWER：pendingOwner\.reviewerCount#3/)
        ).toBeInTheDocument()
    })

    it('候選人被職務分離排空：明講無人可處理，不留空白', () => {
        render(<PendingOwnerInline owner={base} />)
        expect(screen.getByText(/pendingOwner\.nobody/)).toBeInTheDocument()
    })

    it('沒有待處理人（非在途狀態）時不渲染任何東西', () => {
        const { container } = render(<PendingOwnerInline owner={null} />)
        expect(container).toBeEmptyDOMElement()
    })
})

describe('PendingOwnerInline 等待天數', () => {
    beforeEach(() => {
        vi.useFakeTimers()
        vi.setSystemTime(new Date('2026-08-26T04:00:00Z'))
    })
    afterEach(() => {
        vi.useRealTimers()
    })

    it('有 since 時附上已等待天數', () => {
        render(
            <PendingOwnerInline
                owner={{ ...base, candidates: ['王大明'], since: '2026-08-21T03:00:00Z' }}
            />
        )
        expect(screen.getByText('common.waitingDays#5')).toBeInTheDocument()
    })

    it('沒有 since 時不顯示天數', () => {
        render(<PendingOwnerInline owner={{ ...base, candidates: ['王大明'] }} />)
        expect(screen.queryByText(/common\.waitingDays/)).not.toBeInTheDocument()
    })
})

describe('PendingOwnerBadge', () => {
    it('沒有待處理人時原樣渲染徽章，不包 tooltip trigger', () => {
        render(
            <PendingOwnerBadge owner={null}>
                <span>待核准</span>
            </PendingOwnerBadge>
        )
        expect(screen.getByText('待核准')).toBeInTheDocument()
        expect(screen.queryByRole('button')).not.toBeInTheDocument()
    })

    it('有待處理人時包成可聚焦的 trigger（鍵盤可達，不是純 hover-only）', () => {
        render(
            <PendingOwnerBadge owner={{ ...base, candidates: ['王大明'] }}>
                <span>待核准</span>
            </PendingOwnerBadge>
        )
        const trigger = screen.getByRole('button')
        expect(trigger).toContainElement(screen.getByText('待核准'))
        expect(trigger).toHaveAttribute('tabindex', '0')
    })

    /**
     * 釘住 `asChild` + `<div>` 包裝（CodeRabbit 於 PR #30 指出）。
     *
     * 拿掉 `asChild` → Radix 自己渲染真的 `<button>`，而 `Badge` 渲染 `<div>`，
     * 變成 `<button><div>` 無效巢狀；把包裝換成 `<span>` 也一樣容不下 `<div>`。
     * React 對這種巢狀只印 console 警告、不丟例外，所以要直接斷言 trigger 的標籤名。
     */
    it('trigger 必須是 div 而非 button，否則 Badge 的 div 會被包進 button', () => {
        render(
            <PendingOwnerBadge owner={{ ...base, candidates: ['王大明'] }}>
                <div>待核准</div>
            </PendingOwnerBadge>
        )
        const trigger = screen.getByRole('button')
        expect(trigger.tagName).toBe('DIV')
        expect(trigger).toContainElement(screen.getByText('待核准'))
    })
})
