import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { PERMISSIONS } from '@/lib/permissions.generated'

/**
 * 轉讓頁籤的六個動作閘（P0-3）。
 *
 * ## 為什麼補這支測試
 *
 * 這個元件原本用角色硬判，且 admin 那格寫的是 `roles.includes('ADMIN')`——
 * 全 codebase 的角色代碼是 `'admin'` / `'SYSTEM_ADMIN'`（`constants.rs:188-189`），
 * **沒有任何一處用大寫**，所以那個變數恆為 `false`：「完成轉讓」按鈕從不出現、
 * 執行秘書看不到任何轉讓操作。
 *
 * 這個打錯字能潛伏到 2026-09-06 才被發現，直接原因是**本元件當時零測試覆蓋**——
 * 後端的 11 個整合 case 驗的是 API 行為，擋不住前端把角色字串打錯。
 * 本檔就是補上那一層：閘的判準改了會紅，角色字串再打錯也會紅。
 *
 * ## 測法
 *
 * 三個子元件都 mock 成把 `can*` props 攤平到 `data-*` 的探針，
 * 直接斷言 `TransferTab` 算出來的布林，不繞道 UI 細節。
 */

// ── i18n 替身：t(key) 回 key，斷言比對 key（專案慣例）────────────────────────────
vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key }),
    initReactI18next: { type: '3rdParty', init: () => {} },
}))

// 「發起轉讓」按鈕的 i18n key（t() 在測試中回傳 key 本身）
const INITIATE_BUTTON = /animalActions\.transfer\.tab\.initiate/

// ── 授權替身：每個測試自行決定「這個使用者有哪些權限碼、哪些角色」 ──────────────
const granted = new Set<string>()
let roles: string[] = []

vi.mock('@/stores/auth', () => ({
    useAuthUser: () => ({ id: 'u-1', roles }),
    useAuthHasPermission: () => (p: string) => granted.has(p),
}))

// ── 資料替身 ────────────────────────────────────────────────────────────────
type FakeTransfer = { id: string; status: string }
let transfers: FakeTransfer[] = []

vi.mock('@/lib/api', () => ({
    transferApi: { list: () => Promise.resolve({ data: transfers }) },
}))

// ── 子元件替身：把 can* 攤平成 data-* 供斷言 ──────────────────────────────────
vi.mock('../TransferActiveCard', () => ({
    TransferActiveCard: (p: Record<string, unknown>) => (
        <div
            data-testid="active-card"
            data-can-vet-evaluate={String(p.canVetEvaluate)}
            data-can-assign-plan={String(p.canAssignPlan)}
            data-can-approve={String(p.canApprove)}
            data-can-complete={String(p.canComplete)}
            data-can-reject={String(p.canReject)}
        />
    ),
}))
vi.mock('../TransferInitiateForm', () => ({
    TransferInitiateForm: () => <div data-testid="initiate-form" />,
}))
vi.mock('../TransferHistoryList', () => ({
    TransferHistoryList: () => <div data-testid="history" />,
}))
// GuestHide 對非 guest 直接渲染 children；此處不測 guest demo 行為。
vi.mock('@/components/ui/guest-hide', () => ({
    GuestHide: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

const { TransferTab } = await import('../TransferTab')

/** render 並等 react-query 解完 loading。 */
async function renderTab(animalStatus = 'completed') {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(
        <QueryClientProvider client={queryClient}>
            <TransferTab animalId="a-1" animalStatus={animalStatus} earTag="001" />
        </QueryClientProvider>,
    )
    // 歷史清單一定會渲染，用它當「已離開 loading」的訊號。
    await screen.findByTestId('history')
}

function card() {
    return screen.getByTestId('active-card')
}

beforeEach(() => {
    granted.clear()
    roles = []
    transfers = []
})

describe('TransferTab 動作閘', () => {
    // ── 協調段：animal.transfer.manage ──────────────────────────────────────
    it('持 animal.transfer.manage 者看得到「發起轉讓」', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        await renderTab()
        expect(screen.getByRole('button', { name: INITIATE_BUTTON })).toBeInTheDocument()
    })

    // 核心回歸：後端已把協調段從 animal.record.create 換成 animal.transfer.manage，
    // 前端不得繼續認舊碼——否則試驗工作人員又看得到按鈕、按下去吃 403。
    it('只持 animal.record.create 者看不到「發起轉讓」', async () => {
        granted.add(PERMISSIONS.ANIMAL_RECORD_CREATE)
        await renderTab()
        expect(screen.queryByRole('button', { name: INITIATE_BUTTON })).not.toBeInTheDocument()
    })

    it('動物狀態不是 completed 時不得發起（即使有權限）', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        await renderTab('in_experiment')
        expect(screen.queryByRole('button', { name: INITIATE_BUTTON })).not.toBeInTheDocument()
    })

    it('已有進行中的轉讓時不得重複發起', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'pending' }]
        await renderTab()
        expect(screen.queryByRole('button', { name: INITIATE_BUTTON })).not.toBeInTheDocument()
    })

    // 🔴 這一條是那個打錯字的直接回歸：`canComplete` 曾經恆為 false，
    // 「完成轉讓」按鈕對任何人都不出現。
    it('狀態 pi_approved 時，持協調權者可完成轉讓', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'pi_approved' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-complete', 'true')
    })

    it('狀態 vet_evaluated 時，持協調權者可指定新計畫', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'vet_evaluated' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-assign-plan', 'true')
    })

    it('未結案的轉讓，持協調權者可拒絕', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'plan_assigned' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-reject', 'true')
    })

    // ── 獸醫評估：animal.vet.recommend（與後端 require_permission! 同一個碼）──
    it('狀態 pending 時，持 animal.vet.recommend 者可評估', async () => {
        granted.add(PERMISSIONS.ANIMAL_VET_RECOMMEND)
        transfers = [{ id: 't-1', status: 'pending' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-vet-evaluate', 'true')
    })

    it('只持協調權者不得做獸醫評估', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'pending' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-vet-evaluate', 'false')
    })

    // ── 核准：後端是 check_transfer_signing_authority（VET 或兩造 PI），非權限碼 ──
    it('VET 角色在 plan_assigned 時看得到核准', async () => {
        roles = ['VET']
        transfers = [{ id: 't-1', status: 'plan_assigned' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-approve', 'true')
    })

    // 協調權 ≠ 簽署權責：執秘推得動流程，但簽不了名。
    it('只持協調權（非 VET / 非 PI）者不得核准', async () => {
        granted.add(PERMISSIONS.ANIMAL_TRANSFER_MANAGE)
        transfers = [{ id: 't-1', status: 'plan_assigned' }]
        await renderTab()
        expect(card()).toHaveAttribute('data-can-approve', 'false')
    })

    // ── 那個打錯字本身 ──────────────────────────────────────────────────────
    //
    // 大寫 'ADMIN' 不是這個系統的角色代碼。舊版把它當 admin 判準，於是六個閘裡
    // 有四個永遠關著。這條釘住「角色字串不再是判準」：只帶大寫 ADMIN 而沒有
    // 任何權限碼的使用者，一個動作都不該看得到。
    it("大寫 'ADMIN' 角色不構成任何授權（它不是本系統的角色代碼）", async () => {
        roles = ['ADMIN']
        transfers = [{ id: 't-1', status: 'pi_approved' }]
        await renderTab()
        expect(screen.queryByRole('button', { name: INITIATE_BUTTON })).not.toBeInTheDocument()
        expect(card()).toHaveAttribute('data-can-complete', 'false')
        expect(card()).toHaveAttribute('data-can-assign-plan', 'false')
        expect(card()).toHaveAttribute('data-can-reject', 'false')
        expect(card()).toHaveAttribute('data-can-approve', 'false')
    })
})
