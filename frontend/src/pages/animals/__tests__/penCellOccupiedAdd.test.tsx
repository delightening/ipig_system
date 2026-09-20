import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { vi } from 'vitest'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'
import type { AnimalListItem } from '@/types/animal'

/**
 * 迴歸測試：有豬的欄位可以點編號再加入豬隻。
 *
 * 背景：先前有豬的欄位（`penAnimals.length > 0`）完全沒有掛任何互動，使用者只能對空欄位
 * 用滑入觸發的輸入框加豬。這支測試涵蓋新行為：點擊欄位編號（只有第一列的那個按鈕，
 * 不含耳號連結）→ 在最上方插入輸入框、既有動物往下推一列 → 送出呼叫 onQuickMove。
 */

const BUILDING: BuildingWithFacility = {
  id: 'building-1',
  facility_id: 'facility-1',
  facility_code: 'PIGMODEL',
  facility_name: '國模中心',
  code: 'A',
  name: 'A 棟',
  description: null,
  is_active: true,
  config: null,
  sort_order: 1,
}

const ZONE: ZoneWithBuilding = {
  id: 'zone-a',
  building_id: BUILDING.id,
  building_code: BUILDING.code,
  building_name: BUILDING.name,
  facility_id: BUILDING.facility_id,
  facility_name: BUILDING.facility_name,
  code: 'A',
  name: 'A 區',
  color: null,
  is_active: true,
  layout_config: null,
  sort_order: 1,
}

const PEN_A05: PenDetails = {
  id: 'pen-a05',
  code: 'A05',
  name: null,
  capacity: 4,
  current_count: 1,
  status: 'active',
  row_index: 0,
  col_index: 0,
  zone_id: ZONE.id,
  zone_code: ZONE.code,
  zone_name: ZONE.name,
  zone_color: null,
  zone_layout_config: null,
  building_id: BUILDING.id,
  building_code: BUILDING.code,
  building_name: BUILDING.name,
  facility_id: BUILDING.facility_id,
  facility_code: BUILDING.facility_code,
  facility_name: BUILDING.facility_name,
}

const PEN_A06: PenDetails = {
  ...PEN_A05,
  id: 'pen-a06',
  code: 'A06',
  row_index: 1,
}

const EXISTING_ANIMAL: AnimalListItem = {
  id: 'animal-009',
  ear_tag: '009',
  status: 'unassigned',
  breed: 'other',
  gender: 'female',
  entry_date: '2026-01-01',
  pen_location: 'A05',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
}

const EXISTING_ANIMAL_2: AnimalListItem = {
  ...EXISTING_ANIMAL,
  id: 'animal-020',
  ear_tag: '020',
  pen_location: 'A06',
}

const mockLayout = vi.fn()

vi.mock('../hooks/useFacilityLayout', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../hooks/useFacilityLayout')>()
  return {
    ...actual,
    useFacilityLayout: () => mockLayout(),
  }
})

// R89-5：PenCell 的欄位編號按鈕改用 <Can permission={ANIMAL_ANIMAL_EDIT}> 包住，
// 這支測試專注在加豬互動流程本身，不是權限行為，故一律放行。
vi.mock('@/stores/auth', () => ({
  useAuthHasPermission: () => () => true,
}))

// PenCell 的文字改走 i18n：t(key) 回 key，斷言比對 key（不綁語言包內容）。
// initReactI18next 是 lib/i18n（經 lib/utils 被拉進來）在 import 階段需要的 plugin 形狀。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { AnimalPenView } = await import('../components/AnimalPenView')

function renderOccupiedPen() {
  mockLayout.mockReturnValue({
    buildings: [BUILDING],
    zonesByBuilding: { [BUILDING.id]: [ZONE] },
    pensByZone: { [ZONE.id]: [PEN_A05] },
    isLoading: false,
  })
  const onQuickMove = vi.fn()
  render(
    <MemoryRouter>
      <AnimalPenView
        groupedData={[{ pen_location: 'A05', animals: [EXISTING_ANIMAL] }]}
        isLoading={false}
        activeBuildingCode={null}
        onQuickMove={onQuickMove}
        isQuickMovePending={false}
      />
    </MemoryRouter>
  )
  return { onQuickMove }
}

function renderTwoOccupiedPens() {
  mockLayout.mockReturnValue({
    buildings: [BUILDING],
    zonesByBuilding: { [BUILDING.id]: [ZONE] },
    pensByZone: { [ZONE.id]: [PEN_A05, PEN_A06] },
    isLoading: false,
  })
  const onQuickMove = vi.fn()
  render(
    <MemoryRouter>
      <AnimalPenView
        groupedData={[
          { pen_location: 'A05', animals: [EXISTING_ANIMAL] },
          { pen_location: 'A06', animals: [EXISTING_ANIMAL_2] },
        ]}
        isLoading={false}
        activeBuildingCode={null}
        onQuickMove={onQuickMove}
        isQuickMovePending={false}
      />
    </MemoryRouter>
  )
  return { onQuickMove }
}

describe('有豬的欄位點編號加入豬隻', () => {
  it('預設顯示既有動物、欄位編號是可點按鈕', () => {
    renderOccupiedPen()

    expect(screen.getByText('009')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'A05' })).toBeInTheDocument()
  })

  it('點擊欄位編號後在最上方出現輸入框，既有動物仍在畫面上、排在輸入框之後', () => {
    renderOccupiedPen()

    fireEvent.click(screen.getByRole('button', { name: 'A05' }))

    const input = screen.getByPlaceholderText('animalPages.shared.enterEarTag')
    expect(input).toBeInTheDocument()
    // 按鈕本身在編輯狀態下應該消失（換成純文字 + 輸入框）
    expect(screen.queryByRole('button', { name: 'A05' })).not.toBeInTheDocument()

    const earTagEl = screen.getByText('009')
    expect(input.compareDocumentPosition(earTagEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })

  it('輸入耳號按 Enter 送出，呼叫 onQuickMove 帶正確的耳號與欄位', () => {
    const { onQuickMove } = renderOccupiedPen()

    fireEvent.click(screen.getByRole('button', { name: 'A05' }))
    const input = screen.getByPlaceholderText('animalPages.shared.enterEarTag')
    fireEvent.change(input, { target: { value: '010' } })
    fireEvent.keyDown(input, { key: 'Enter' })

    expect(onQuickMove).toHaveBeenCalledWith('010', 'A05')
  })

  it('點擊 009 的連結不會觸發新增輸入框（事件不應該冒泡到欄位按鈕）', () => {
    renderOccupiedPen()

    fireEvent.click(screen.getByText('009'))

    expect(screen.queryByPlaceholderText('animalPages.shared.enterEarTag')).not.toBeInTheDocument()
  })

  it('在 A05 輸入到一半就切到 A06，A05 失焦後 150ms 的延遲送出不會誤送舊文字、也不會蓋掉 A06 剛開始的編輯', async () => {
    // 用 fake timer 精準跳過 150ms，不用真的等——避免 CI 上因為排程延遲造成時間margin 不夠而偶發失敗
    vi.useFakeTimers()
    try {
      const { onQuickMove } = renderTwoOccupiedPens()

      fireEvent.click(screen.getByRole('button', { name: 'A05' }))
      const inputA05 = screen.getByPlaceholderText('animalPages.shared.enterEarTag')
      fireEvent.change(inputA05, { target: { value: '999' } })
      fireEvent.blur(inputA05) // 排入 150ms 後的延遲 submit/cancel

      // 在延遲觸發前就切到另一個欄位（不用等，立即操作）
      fireEvent.click(screen.getByRole('button', { name: 'A06' }))
      expect(screen.getByPlaceholderText('animalPages.shared.enterEarTag')).toBeInTheDocument()

      // 精準跳過 A05 那筆延遲 callback 原本會觸發的時間點
      await vi.advanceTimersByTimeAsync(200)

      // A05 的殘留文字「999」不該被送出
      expect(onQuickMove).not.toHaveBeenCalledWith('999', 'A05')
      // A06 剛開始的編輯不該被 A05 那筆遲到的 callback 清掉
      expect(screen.getByPlaceholderText('animalPages.shared.enterEarTag')).toBeInTheDocument()
    } finally {
      vi.useRealTimers()
    }
  })
})
