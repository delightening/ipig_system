import { render, screen } from '@testing-library/react'
import { vi } from 'vitest'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

/**
 * 迴歸測試：區域卡片的欄數要由資料決定。
 *
 * 先前 `renderZoneCard` 寫死 `grid-cols-2`，無論該區域實際有沒有右排欄位都畫兩欄。
 * 單排的區域（羊舍 S、檢疫舍 Q——欄位的 col_index 是 NULL，全歸左欄）右半邊
 * 就變成一整排空白格，看起來像「有一排不存在的欄位」。
 *
 * 真正雙排的區域（A/B/C/D，col_index 0 與 1 各半）必須維持兩欄，故兩種情境都測。
 */

const BUILDING: BuildingWithFacility = {
  id: 'building-1',
  facility_id: 'facility-1',
  facility_code: 'PIGMODEL',
  facility_name: '國模中心',
  code: 'S',
  name: '羊舍',
  description: null,
  is_active: true,
  config: null,
  sort_order: 1,
}

function makeZone(id: string, code: string, name: string): ZoneWithBuilding {
  return {
    id,
    building_id: BUILDING.id,
    building_code: BUILDING.code,
    building_name: BUILDING.name,
    facility_id: BUILDING.facility_id,
    facility_name: BUILDING.facility_name,
    code,
    name,
    color: null,
    is_active: true,
    layout_config: null,
    sort_order: 1,
  }
}

function makePen(id: string, code: string, zone: ZoneWithBuilding, colIndex: number | null, rowIndex: number): PenDetails {
  return {
    id,
    code,
    name: null,
    capacity: 4,
    current_count: 0,
    status: 'active',
    row_index: rowIndex,
    col_index: colIndex,
    zone_id: zone.id,
    zone_code: zone.code,
    zone_name: zone.name,
    zone_color: null,
    zone_layout_config: null,
    building_id: BUILDING.id,
    building_code: BUILDING.code,
    building_name: BUILDING.name,
    facility_id: BUILDING.facility_id,
    facility_code: BUILDING.facility_code,
    facility_name: BUILDING.facility_name,
  }
}

const SINGLE_ZONE = makeZone('zone-single', 'S', '羊舍區')
const DOUBLE_ZONE = makeZone('zone-double', 'A', 'A 區')

/** 單排：col_index 為 NULL，正是單筆「新增欄位」建出來的樣子 */
const SINGLE_PENS = [makePen('pen-s1', 'S01', SINGLE_ZONE, null, 0)]

/** 雙排：左右各一 */
const DOUBLE_PENS = [
  makePen('pen-a1', 'A01', DOUBLE_ZONE, 0, 0),
  makePen('pen-a2', 'A02', DOUBLE_ZONE, 1, 0),
]

const mockLayout = vi.fn()

vi.mock('../hooks/useFacilityLayout', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../hooks/useFacilityLayout')>()
  return {
    ...actual,
    useFacilityLayout: () => mockLayout(),
  }
})

const { AnimalPenView } = await import('../components/AnimalPenView')

function renderWithZone(zone: ZoneWithBuilding, pens: PenDetails[]) {
  mockLayout.mockReturnValue({
    buildings: [BUILDING],
    zonesByBuilding: { [BUILDING.id]: [zone] },
    pensByZone: { [zone.id]: pens },
    isLoading: false,
  })
  return render(
    <AnimalPenView
      groupedData={[]}
      isLoading={false}
      // null = 未指定棟別，fallback 到 buildings[0]（本測試 mock 的唯一一棟）
      activeBuildingCode={null}
      onQuickMove={() => {}}
      isQuickMovePending={false}
    />
  )
}

describe('AnimalPenView 區域卡片欄數', () => {
  it('單排區域只畫一欄，右半邊不會出現空白欄', () => {
    renderWithZone(SINGLE_ZONE, SINGLE_PENS)

    // PenGridHeader 每欄各有一個「欄位」表頭
    expect(screen.getAllByText('欄位')).toHaveLength(1)
    expect(screen.getByText('S01')).toBeInTheDocument()
  })

  it('真正雙排的區域維持兩欄（不因此次修正而退化）', () => {
    renderWithZone(DOUBLE_ZONE, DOUBLE_PENS)

    expect(screen.getAllByText('欄位')).toHaveLength(2)
    expect(screen.getByText('A01')).toBeInTheDocument()
    expect(screen.getByText('A02')).toBeInTheDocument()
  })
})
