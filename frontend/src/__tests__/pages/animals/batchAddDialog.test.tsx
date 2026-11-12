/**
 * 批次新增動物對話框。
 *
 * 要釘住的契約：
 * 1. 貼上耳號範圍／清單並按「加入列表」→ 預覽依序列出各隻，計數與上限顯示正確。
 * 2. 無法辨識的片段與重複的耳號會提示，不會悄悄吞掉。
 * 3. 可移除單隻。
 * 4. 共用的必填欄位（品種、出生日期、欄位、實驗前代號）沒填齊、或體重沒填時，送出鈕保持停用。
 *
 * Radix Select 在 jsdom 裡難以操作，所以「選了品種／欄位之後能送出」由純邏輯測試
 * （batchAddLogic.test.ts）與後端整合測試覆蓋，這裡只驗證畫面行為。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// t() 回傳 key；有插值時把 JSON 接在後面，才斷言得到計數與內容
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => (opts ? `${key}|${JSON.stringify(opts)}` : key),
    i18n: { language: 'zh-TW' },
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { apiPost } = vi.hoisted(() => ({ apiPost: vi.fn() }))
vi.mock('@/lib/api', () => ({ default: { post: apiPost }, isAxiosError: () => false }))

vi.mock('@/pages/animals/hooks/useFacilityLayout', () => ({
  useFacilityLayout: () => ({
    buildings: [],
    zonesByBuilding: {},
    pensByZone: {
      z1: [
        {
          id: 'pen-a',
          code: 'A01',
          name: null,
          capacity: 0,
          current_count: 0,
          status: 'active',
          zone_id: 'z1',
          zone_code: 'Z',
          zone_name: '測試區',
          building_code: 'B',
          building_name: '測試棟',
        },
      ],
    },
    isLoading: false,
  }),
}))
vi.mock('@/pages/animals/hooks/useBreedSpecies', () => ({
  useBreedSpecies: () => ({ breedSpecies: [{ id: 'sp-mini', code: 'minipig', name: '迷你豬' }] }),
}))

import { BatchAddAnimalsDialog } from '@/pages/animals/batchAdd/BatchAddAnimalsDialog'

function renderDialog() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <BatchAddAnimalsDialog open onOpenChange={() => {}} sourcesData={[]} />
    </QueryClientProvider>,
  )
}

function addTags(text: string) {
  fireEvent.change(screen.getByLabelText(/animals\.earTag/), { target: { value: text } })
  fireEvent.click(screen.getByRole('button', { name: 'animalPages.batchAdd.addToList' }))
}

/** 預覽表格（容器查詢在 jsdom 不生效，表格與卡片都會渲染，只取表格那份）。 */
function previewTable() {
  return screen.getByRole('table')
}

describe('BatchAddAnimalsDialog', () => {
  beforeEach(() => {
    apiPost.mockReset()
  })

  it('貼上範圍後依序列出各隻，並顯示計數與上限', () => {
    renderDialog()
    addTags('021-023')

    const rows = within(previewTable()).getAllByRole('row')
    // 第一列是表頭
    expect(rows).toHaveLength(4)
    expect(within(rows[1]).getByText('021')).toBeTruthy()
    expect(within(rows[3]).getByText('023')).toBeTruthy()
    expect(
      screen.getByText(/animalPages\.batchAdd\.listTitle\|\{"count":3,"max":50\}/),
    ).toBeTruthy()
  })

  it('無法辨識的片段與重複的耳號會提示', () => {
    renderDialog()
    addTags('021 abc 021')

    expect(screen.getByText(/animalPages\.batchAdd\.noteInvalid/)).toBeTruthy()
    expect(screen.getByText(/animalPages\.batchAdd\.noteDuplicates/)).toBeTruthy()
    expect(within(previewTable()).getAllByRole('row')).toHaveLength(2)
  })

  it('可移除單隻', () => {
    renderDialog()
    addTags('021-022')

    fireEvent.click(
      within(previewTable()).getByRole('button', {
        name: /animalPages\.batchAdd\.removeRow\|\{"earTag":"021"\}/,
      }),
    )
    const rows = within(previewTable()).getAllByRole('row')
    expect(rows).toHaveLength(2)
    expect(within(rows[1]).getByText('022')).toBeTruthy()
  })

  it('共用必填欄位與體重沒填齊時，送出鈕保持停用且不送出任何請求', () => {
    renderDialog()
    addTags('021')

    const submit = screen.getByRole('button', { name: /animalPages\.batchAdd\.submit/ })
    expect((submit as HTMLButtonElement).disabled).toBe(true)

    // 就算補上實驗前代號與體重，品種／出生日期／欄位仍未選 → 仍不可送出
    fireEvent.change(screen.getByLabelText(/animalPages\.shared\.preExperimentCode/), {
      target: { value: 'PRE-001' },
    })
    fireEvent.change(within(previewTable()).getByLabelText(/animalPages\.shared\.entryWeightKg 021/), {
      target: { value: '25.5' },
    })
    expect((submit as HTMLButtonElement).disabled).toBe(true)
    expect(apiPost).not.toHaveBeenCalled()
  })

  it('尚未加入耳號時顯示空狀態，送出鈕停用', () => {
    renderDialog()
    expect(screen.getByText('animalPages.batchAdd.emptyList')).toBeTruthy()
    const submit = screen.getByRole('button', { name: /animalPages\.batchAdd\.submit/ })
    expect((submit as HTMLButtonElement).disabled).toBe(true)
  })
})
