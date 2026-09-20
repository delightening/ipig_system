import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import type { OvertimeWithUser } from '@/types/hr'

/**
 * R86-2 作廢通道的前端閘。
 *
 * 釘住三件事（後端各有對應守衛，這裡只確保 UI 不顯示注定失敗的按鈕）：
 * 1. 只有負責人、只有已核准、且不是自己的單，才看得到「作廢」鈕。
 * 2. 作廢理由必填且有長度下限，未過關不得送出 API。
 * 3. 已作廢的列要看得到作廢原因——否則使用者只看到狀態變了卻不知為何。
 */

const apiPost = vi.fn().mockResolvedValue({ data: {} })
vi.mock('@/lib/api', () => ({
    default: { post: (...args: unknown[]) => apiPost(...args) },
    deleteResource: vi.fn(),
}))

vi.mock('@/components/ui/use-toast', () => ({ toast: vi.fn() }))

// 元件文字全走 i18n：t(key) 回 key（有插值參數時把參數附在後面，讓「作廢原因」等內插值仍可斷言）。
// lib/utils 會拉進 lib/i18n（.use(initReactI18next)），mock 整個模組就得補上這個 plugin 形狀。
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: (key: string, opts?: Record<string, unknown>) =>
            opts ? `${key} ${JSON.stringify(opts)}` : key,
    }),
    initReactI18next: { type: '3rdParty', init: () => {} },
}))

let isAdmin = true
let currentUserId = 'admin-1'
vi.mock('@/stores/auth', () => ({
    useAuthIsAdmin: () => isAdmin,
    useAuthUser: () => ({ id: currentUserId }),
}))

const { AllRecordsTable } = await import('../components/AllRecordsTable')

function mkRow(over: Partial<OvertimeWithUser> = {}): OvertimeWithUser {
    return {
        id: 'ot-1',
        user_id: 'staff-1',
        user_email: 'staff@example.com',
        user_name: '測試同仁',
        overtime_date: '2026-07-20',
        start_time: '2026-07-20T07:30:00Z',
        end_time: '2026-07-20T12:00:00Z',
        hours: 4.5,
        overtime_type: 'C',
        multiplier: 1.66,
        comp_time_hours: 8,
        comp_time_expires_at: '2027-07-20',
        status: 'approved',
        reason: '國定假日值班（補登）',
        can_approve: false,
        ...over,
    }
}

function renderTable(rows: OvertimeWithUser[]) {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return render(
        <QueryClientProvider client={queryClient}>
            <AllRecordsTable data={rows} isLoading={false} />
        </QueryClientProvider>,
    )
}

/** 桌機表格與窄版卡片在 jsdom 會同時渲染（Tailwind 斷點不生效），故一律取第一顆 */
const voidButtons = () =>
    screen.queryAllByRole('button', { name: /hrPages\.overtime\.void\.button/ })

describe('加班單作廢（R86-2）', () => {
    beforeEach(() => {
        apiPost.mockClear()
        isAdmin = true
        currentUserId = 'admin-1'
    })

    it('負責人看得到他人已核准單的作廢鈕', () => {
        renderTable([mkRow()])
        expect(voidButtons().length).toBeGreaterThan(0)
    })

    it('非負責人看不到作廢鈕', () => {
        isAdmin = false
        renderTable([mkRow()])
        expect(voidButtons()).toHaveLength(0)
    })

    it('自己的加班單看不到作廢鈕', () => {
        currentUserId = 'staff-1'
        renderTable([mkRow()])
        expect(voidButtons()).toHaveLength(0)
    })

    it('尚未核准的單看不到作廢鈕', () => {
        renderTable([mkRow({ status: 'pending_admin' })])
        expect(voidButtons()).toHaveLength(0)
    })

    it('已作廢的列顯示狀態與作廢原因', () => {
        renderTable([mkRow({ status: 'voided', void_reason: '補登重跑造成的重複單' })])
        expect(screen.getAllByText('hrPages.shared.overtimeStatus.voided').length).toBeGreaterThan(0)
        expect(screen.getAllByText(/補登重跑造成的重複單/).length).toBeGreaterThan(0)
        expect(voidButtons()).toHaveLength(0)
    })

    it('理由過短不送出，填妥後以 trim 後的理由呼叫作廢 API', async () => {
        renderTable([mkRow()])
        fireEvent.click(voidButtons()[0])

        // DeleteReasonDialog 的文案（noun 內插由 hrPages.overtime.void.actionNoun 帶入）
        const textarea = screen.getByPlaceholderText(/ui\.deleteReasonDialog\.reasonPlaceholder/)
        const confirm = screen.getByRole('button', { name: /^ui\.deleteReasonDialog\.confirm / })

        fireEvent.change(textarea, { target: { value: '重複' } })
        fireEvent.click(confirm)
        expect(await screen.findByText(/ui\.deleteReasonDialog\.reasonTooShort/)).toBeInTheDocument()
        expect(apiPost).not.toHaveBeenCalled()

        fireEvent.change(textarea, { target: { value: '  補登重跑造成的重複單  ' } })
        fireEvent.click(confirm)
        await waitFor(() =>
            expect(apiPost).toHaveBeenCalledWith('/hr/overtime/ot-1/void', {
                reason: '補登重跑造成的重複單',
            }),
        )
    })
})
