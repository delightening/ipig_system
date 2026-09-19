import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent 而非 user-event：@testing-library/user-event 不在專案依賴內
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const apiPut = vi.fn()
vi.mock('@/lib/api', () => ({
  default: { put: (...args: unknown[]) => apiPut(...args) },
}))

const toastFn = vi.fn()
vi.mock('@/components/ui/use-toast', () => ({
  toast: (...args: unknown[]) => toastFn(...args),
}))

// 元件文字走 t()：mock 成回傳 key，斷言比對 key 而非任何語言的譯文。
// `lib/utils` 會拉進 `lib/i18n`（後者 `.use(initReactI18next)`），故要補上 plugin 形狀。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

import { WarehouseInactiveDialog } from '@/components/warehouse/WarehouseInactiveDialog'
import type { Warehouse } from '@/types/erp'

/** 「復原」按鈕的 i18n key（mock 的 t() 直接回傳 key，按鈕的可存取名稱就是它） */
const RESTORE_KEY = 'erpDocs.warehouse.inactive.restore'

function makeWarehouse(overrides: Partial<Warehouse> = {}): Warehouse {
  return {
    id: 'wh-1',
    code: '2',
    name: '儲藏室',
    is_active: false,
    // migration 014/015 的政策旗標。這支測試的樣本剛好是儲藏室——現實中三項都會是
    // true，但本測試驗的是停用／復原流程，與政策無關，用欄位預設值即可。
    exclude_from_alerts: false,
    skip_routine_stocktake: false,
    is_default_issue_source: false,
    created_at: '2026-03-18T02:20:11Z',
    updated_at: '2026-08-05T08:06:55Z',
    ...overrides,
  }
}

function renderDialog(warehouses: Warehouse[]) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <WarehouseInactiveDialog open onOpenChange={() => {}} warehouses={warehouses} />
    </QueryClientProvider>,
  )
}

describe('WarehouseInactiveDialog', () => {
  beforeEach(() => {
    apiPut.mockReset()
    toastFn.mockReset()
  })

  it('列出已停用的倉庫', () => {
    renderDialog([makeWarehouse()])

    expect(screen.getByText('2 - 儲藏室')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: RESTORE_KEY })).toBeInTheDocument()
  })

  // 這支釘住復原的請求形狀：誤送成 DELETE 或漏帶 is_active 會讓倉庫救不回來
  it('點復原送出 PUT /warehouses/{id} 與 is_active: true', async () => {
    apiPut.mockResolvedValue({ data: makeWarehouse({ is_active: true }) })
    renderDialog([makeWarehouse()])

    fireEvent.click(screen.getByRole('button', { name: RESTORE_KEY }))

    await waitFor(() => {
      expect(apiPut).toHaveBeenCalledWith('/warehouses/wh-1', { is_active: true })
    })
  })

  it('復原失敗時顯示錯誤 toast，不靜默吞掉', async () => {
    apiPut.mockRejectedValue(new Error('boom'))
    renderDialog([makeWarehouse()])

    fireEvent.click(screen.getByRole('button', { name: RESTORE_KEY }))

    await waitFor(() => {
      expect(toastFn).toHaveBeenCalledWith(
        expect.objectContaining({ variant: 'destructive' }),
      )
    })
  })

  it('沒有停用倉庫時顯示空狀態', () => {
    renderDialog([])

    expect(screen.getByText('erpDocs.warehouse.inactive.empty')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: RESTORE_KEY })).not.toBeInTheDocument()
  })
})
