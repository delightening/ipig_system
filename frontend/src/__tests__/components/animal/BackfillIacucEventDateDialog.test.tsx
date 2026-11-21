/**
 * 補登 IACUC 變更事件實際日期的對話框。
 *
 * 要釘住的契約：
 * 1. 日期或原因沒填、日期晚於今天 → 送出鈕停用。
 * 2. 送出帶上事件 id、日期與去空白的原因；成功後重新載入事件列表、顯示成功訊息並關閉。
 * 3. 後端拒絕（例如日期超出範圍）→ 後端訊息顯示在對話框內，對話框保持開啟，不顯示成功訊息。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent/純渲染而非 user-event：@testing-library/user-event 不在專案依賴內
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AxiosError, type AxiosResponse } from 'axios'

// t() 回傳 key 本身，斷言比對 key，不受文案改寫與語系影響。
// 整個 react-i18next 被 mock，要補上 initReactI18next 的 plugin 形狀，否則 lib/i18n 在 import 階段就炸。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const K = 'animalActions.timeline'

const backfill = vi.fn()
vi.mock('@/lib/api/animalFieldCorrection', () => ({
  animalFieldCorrectionApi: {
    backfillIacucEventDate: (...args: unknown[]) => backfill(...args),
  },
}))

const toast = vi.fn()
vi.mock('@/components/ui/use-toast', () => ({
  toast: (...args: unknown[]) => toast(...args),
}))

import { BackfillIacucEventDateDialog } from '@/components/animal/BackfillIacucEventDateDialog'
import type { AnimalEvent } from '@/types/animal'

const event: AnimalEvent = {
  id: 'evt-1',
  event_type: 'IACUC_CHANGE',
  before_data: { iacuc_no: null },
  after_data: { iacuc_no: 'PIG-114005' },
  created_at: '2026-11-12T03:00:00Z',
}

function renderDialog() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const invalidate = vi.spyOn(queryClient, 'invalidateQueries')
  const onOpenChange = vi.fn()
  render(
    <QueryClientProvider client={queryClient}>
      <BackfillIacucEventDateDialog
        open
        onOpenChange={onOpenChange}
        animalId="animal-1"
        event={event}
      />
    </QueryClientProvider>,
  )
  return { invalidate, onOpenChange }
}

const submitButton = () => screen.getByRole('button', { name: `${K}.backfillSubmit` })

function fill(date: string, reason: string) {
  fireEvent.change(screen.getByLabelText(`${K}.backfillDateLabel`), { target: { value: date } })
  fireEvent.change(screen.getByLabelText(`${K}.backfillReasonLabel`), { target: { value: reason } })
}

describe('BackfillIacucEventDateDialog', () => {
  beforeEach(() => {
    backfill.mockReset()
    toast.mockReset()
  })

  it('日期或原因沒填時不能送出', () => {
    renderDialog()
    expect(submitButton()).toBeDisabled()

    fill('2026-07-20', '')
    expect(submitButton()).toBeDisabled()

    fill('', '依紙本申請單補登')
    expect(submitButton()).toBeDisabled()

    fill('2026-07-20', '   ')
    expect(submitButton()).toBeDisabled()

    fill('2026-07-20', '依紙本申請單補登')
    expect(submitButton()).toBeEnabled()
  })

  it('日期晚於今天：顯示提示並停用送出', () => {
    renderDialog()
    fill('2999-01-01', '依紙本申請單補登')

    expect(screen.getByText(`${K}.backfillDateFuture`)).toBeInTheDocument()
    expect(submitButton()).toBeDisabled()
  })

  it('送出帶上事件 id、日期與去空白的原因；成功後重新載入事件、顯示成功訊息並關閉', async () => {
    backfill.mockResolvedValue({
      data: {
        event_id: 'evt-1',
        actual_date: '2026-07-20',
        backfill: { by_name: '管理員甲', at: '2026-11-21T02:00:00Z', reason: '依紙本申請單補登' },
      },
    })
    const { invalidate, onOpenChange } = renderDialog()
    fill('2026-07-20', '  依紙本申請單補登  ')

    fireEvent.click(submitButton())

    await waitFor(() => expect(backfill).toHaveBeenCalledTimes(1))
    expect(backfill).toHaveBeenCalledWith('animal-1', 'evt-1', {
      actual_date: '2026-07-20',
      reason: '依紙本申請單補登',
    })
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false))
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['animal-iacuc-events', 'animal-1'] })
    expect(toast).toHaveBeenCalledWith({ title: `${K}.backfillSuccess` })
  })

  it('後端拒絕：訊息顯示在對話框內，對話框保持開啟，不顯示成功訊息', async () => {
    const message = '實際日期須介於 2026-07-13 與 2026-08-20 之間（含）'
    backfill.mockRejectedValue(
      new AxiosError('request failed', 'ERR_BAD_REQUEST', undefined, undefined, {
        status: 400,
        data: { error: { message } },
      } as AxiosResponse),
    )
    const { invalidate, onOpenChange } = renderDialog()
    fill('2026-09-01', '依紙本申請單補登')

    fireEvent.click(submitButton())

    expect(await screen.findByText(message)).toBeInTheDocument()
    expect(onOpenChange).not.toHaveBeenCalledWith(false)
    expect(toast).not.toHaveBeenCalled()
    expect(invalidate).not.toHaveBeenCalled()
    // 可以改日期再送
    expect(submitButton()).toBeEnabled()
  })
})
