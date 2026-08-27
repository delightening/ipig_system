import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import type { AttendanceWithUser } from '@/types/hr'

/**
 * 補卡對話框的前端閘。
 *
 * 釘住三件事（後端各有對應守衛，這裡只確保 UI 不送出注定失敗的請求、
 * 以及**送出的時間值是對的**）：
 * 1. 理由有長度下限，未過關不得送出 API。
 * 2. 送出的時間必須是 UTC——表單填的是台灣時間，換算方向寫反會整整差 8 小時，
 *    而畫面上看起來完全正常（顯示時又轉回去了），只有這種測試抓得到。
 * 3. 補登模式在還沒選人員／日期之前，送出鈕必須是 disabled。
 */

const apiPost = vi.fn().mockResolvedValue({ data: { success: true, id: 'att-new' } })
const apiPut = vi.fn().mockResolvedValue({ data: { success: true } })
vi.mock('@/lib/api', () => ({
    default: {
        post: (...args: unknown[]) => apiPost(...args),
        put: (...args: unknown[]) => apiPut(...args),
    },
}))

vi.mock('@/components/ui/use-toast', () => ({ toast: vi.fn() }))

const { AttendanceCorrectionDialog } = await import('../components/AttendanceCorrectionDialog')

function mkRecord(over: Partial<AttendanceWithUser> = {}): AttendanceWithUser {
    return {
        id: 'att-1',
        user_id: 'staff-1',
        user_email: 'staff@example.com',
        user_name: '測試同仁',
        work_date: '2026-08-25',
        // 台灣時間 08:30 / 17:30
        clock_in_time: '2026-08-25T00:30:00Z',
        clock_out_time: '2026-08-25T09:30:00Z',
        regular_hours: 8,
        overtime_hours: 0,
        status: 'normal',
        remark: null,
        is_corrected: false,
        ...over,
    }
}

function renderDialog(record: AttendanceWithUser | null) {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return render(
        <QueryClientProvider client={queryClient}>
            <AttendanceCorrectionDialog
                open
                onOpenChange={() => {}}
                record={record}
                staffList={[{ id: 'staff-1', display_name: '測試同仁', email: 'staff@example.com' }]}
            />
        </QueryClientProvider>,
    )
}

const reasonBox = () => screen.getByPlaceholderText(/忘記打卡/)

describe('補卡對話框', () => {
    beforeEach(() => {
        apiPost.mockClear()
        apiPut.mockClear()
    })

    it('更正模式帶入既有紀錄的台灣時間，不是 UTC 原值', () => {
        renderDialog(mkRecord())
        expect(screen.getByLabelText('上班時間')).toHaveValue('08:30')
        expect(screen.getByLabelText('下班時間')).toHaveValue('17:30')
    })

    it('理由過短時送出鈕 disabled，不呼叫 API', () => {
        renderDialog(mkRecord())
        const confirm = screen.getByRole('button', { name: '確認更正' })

        expect(confirm).toBeDisabled()

        fireEvent.change(reasonBox(), { target: { value: '忘記' } })
        expect(confirm).toBeDisabled()

        fireEvent.click(confirm)
        expect(apiPut).not.toHaveBeenCalled()
    })

    it('更正送出的時間是 UTC，理由已 trim', async () => {
        renderDialog(mkRecord())

        fireEvent.change(screen.getByLabelText('上班時間'), { target: { value: '09:00' } })
        fireEvent.change(reasonBox(), { target: { value: '  忘記打卡，主管確認  ' } })
        fireEvent.click(screen.getByRole('button', { name: '確認更正' }))

        await waitFor(() =>
            expect(apiPut).toHaveBeenCalledWith('/hr/attendance/att-1', {
                // 台灣 09:00 → UTC 01:00（差 8 小時，方向不可寫反）
                clock_in_time: '2026-08-25T01:00:00.000Z',
                clock_out_time: '2026-08-25T09:30:00.000Z',
                reason: '忘記打卡，主管確認',
            }),
        )
    })

    it('補登模式未選人員與日期時，送出鈕 disabled', () => {
        renderDialog(null)
        fireEvent.change(reasonBox(), { target: { value: '忘記打卡' } })
        expect(screen.getByRole('button', { name: '確認補登' })).toBeDisabled()
        expect(apiPost).not.toHaveBeenCalled()
    })

    it('兩種模式都告知不得補自己的紀錄', () => {
        const { unmount } = renderDialog(mkRecord())
        expect(screen.getByText(/不得補登或更正自己的紀錄/)).toBeInTheDocument()
        unmount()

        renderDialog(null)
        expect(screen.getByText(/不得補登或更正自己的紀錄/)).toBeInTheDocument()
    })

    it('更正模式顯示對象與日期，且不出現人員下拉', () => {
        renderDialog(mkRecord())
        expect(screen.getByText('測試同仁')).toBeInTheDocument()
        expect(screen.queryByLabelText('人員')).not.toBeInTheDocument()
    })
})
