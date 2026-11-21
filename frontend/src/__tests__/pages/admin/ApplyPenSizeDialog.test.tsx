import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent/純渲染而非 user-event：@testing-library/user-event 不在專案依賴內
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AxiosError, type AxiosResponse } from 'axios'

// t() 回傳 key 本身（帶 count 時附上 `:count`），斷言比對 key，不受文案改寫與語系影響。
// 整個 react-i18next 被 mock，要補上 initReactI18next 的 plugin 形狀，否則 lib/i18n 在 import 階段就炸。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { count?: number }) =>
      opts && opts.count !== undefined ? `${key}:${opts.count}` : key,
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const K = 'admin.penLayoutPreview'

const applyPenSize = vi.fn()
vi.mock('@/lib/api/facility', () => ({
  facilityApi: {
    applyPenSize: (...args: unknown[]) => applyPenSize(...args),
  },
}))

const toast = vi.fn()
vi.mock('@/components/ui/use-toast', () => ({
  toast: (...args: unknown[]) => toast(...args),
}))

import { ApplyPenSizeDialog } from '@/pages/admin/components/ApplyPenSizeDialog'
import type { PenDetails, ZoneWithBuilding } from '@/types/facility'

// 只填元件實際會讀的欄位，其餘用型別斷言略過
function makePen(id: string): PenDetails {
  return { id, code: id, length_m: null, width_m: null, zone_id: 'zone-1' } as unknown as PenDetails
}

const zone = { id: 'zone-1', code: 'Z1', building_code: 'B1' } as unknown as ZoneWithBuilding

function axiosError(status: number) {
  return new AxiosError('request failed', 'ERR_BAD_REQUEST', undefined, undefined, {
    status,
    data: {},
  } as AxiosResponse)
}

function renderDialog(pens: PenDetails[]) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const invalidate = vi.spyOn(queryClient, 'invalidateQueries')
  const onOpenChange = vi.fn()
  render(
    <QueryClientProvider client={queryClient}>
      <ApplyPenSizeDialog open onOpenChange={onOpenChange} zone={zone} pens={pens} />
    </QueryClientProvider>,
  )
  return { invalidate, onOpenChange }
}

function fillSizeAndApply() {
  fireEvent.change(screen.getByPlaceholderText('2.40'), { target: { value: ' 1.20 ' } })
  fireEvent.change(screen.getByPlaceholderText('3.00'), { target: { value: '2.40' } })
  fireEvent.click(screen.getByRole('button', { name: `${K}.applySizeSubmit` }))
}

describe('ApplyPenSizeDialog 的欄位名單檢查', () => {
  beforeEach(() => {
    applyPenSize.mockReset()
    toast.mockReset()
  })

  it('送出時帶上畫面上看到的欄位 ID，長寬去除前後空白', async () => {
    applyPenSize.mockResolvedValue({ data: { updated_count: 2 } })
    renderDialog([makePen('pen-a'), makePen('pen-b')])

    fillSizeAndApply()

    await waitFor(() => expect(applyPenSize).toHaveBeenCalledTimes(1))
    expect(applyPenSize).toHaveBeenCalledWith('zone-1', {
      length_m: '1.20',
      width_m: '2.40',
      expected_pen_ids: ['pen-a', 'pen-b'],
    })
  })

  it('收到 409：重新載入欄位資料、顯示說明，對話框保持開啟', async () => {
    applyPenSize.mockRejectedValue(axiosError(409))
    const { invalidate, onOpenChange } = renderDialog([makePen('pen-a'), makePen('pen-b')])

    fillSizeAndApply()

    await waitFor(() => expect(toast).toHaveBeenCalledTimes(1))
    expect(toast).toHaveBeenCalledWith({
      title: `${K}.applySizeConflict`,
      variant: 'destructive',
    })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['pens'] })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['facility-pens'] })
    // 沒有關閉，也沒有走「結果不確定」的訊息
    expect(onOpenChange).not.toHaveBeenCalledWith(false)
    expect(screen.getByText(`${K}.applySizeTitle`)).toBeInTheDocument()
    expect(toast).not.toHaveBeenCalledWith(expect.objectContaining({ title: `${K}.applySizeUnknown` }))
  })

  it('其他 4xx 仍走一般失敗訊息，不重新載入欄位資料', async () => {
    applyPenSize.mockRejectedValue(axiosError(400))
    const { invalidate } = renderDialog([makePen('pen-a'), makePen('pen-b')])

    fillSizeAndApply()

    await waitFor(() => expect(toast).toHaveBeenCalledTimes(1))
    expect(toast).toHaveBeenCalledWith(expect.objectContaining({ title: `${K}.applySizeFailed` }))
    expect(invalidate).not.toHaveBeenCalled()
  })
})
