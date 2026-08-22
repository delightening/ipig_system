import { render, screen, fireEvent } from '@testing-library/react'
import { vi } from 'vitest'
import i18n from '@/lib/i18n'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

/**
 * 迴歸測試：欄位編號下拉不可對 pen code 做字串裁切。
 *
 * 舊寫法 `value={pen.code.slice(1)}` 假設 code 一定是「一個區碼字母 + 編號」。
 * 欄位 code 只要不是那個形狀（例如檢疫舍的 `Q`、羊舍的 `羊` 這種單字元），
 * slice(1) 就會得到空字串。Radix Select 把空字串當成「未選取」，該選項雖然畫得出來、
 * 點下去卻不會觸發 onValueChange——欄位編號永遠是空的、送出鈕一直 disabled，
 * 結果是「新增動物」表單在那些棟舍底下根本無法完成。
 */

const BUILDING: BuildingWithFacility = {
  id: 'building-1',
  facility_id: 'facility-1',
  facility_code: 'PIGMODEL',
  facility_name: '國模中心',
  code: 'Q',
  name: '檢疫舍',
  description: null,
  is_active: true,
  config: null,
  sort_order: 1,
}

const ZONE: ZoneWithBuilding = {
  id: 'zone-1',
  building_id: BUILDING.id,
  building_code: BUILDING.code,
  building_name: BUILDING.name,
  facility_id: BUILDING.facility_id,
  facility_name: BUILDING.facility_name,
  code: 'Q',
  name: 'Quarantine',
  color: null,
  is_active: true,
  layout_config: null,
  sort_order: 1,
}

/** 關鍵：單字元 code，正是舊寫法會裁成空字串的那種 */
const SINGLE_CHAR_PEN: PenDetails = {
  id: 'pen-1',
  code: 'Q',
  name: null,
  capacity: 1,
  current_count: 0,
  status: 'active',
  row_index: null,
  col_index: null,
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

vi.mock('../hooks/useFacilityLayout', () => ({
  useFacilityLayout: () => ({
    buildings: [BUILDING],
    zonesByBuilding: { [BUILDING.id]: [ZONE] },
    pensByZone: { [ZONE.id]: [SINGLE_CHAR_PEN] },
    isLoading: false,
  }),
}))

vi.mock('../hooks/useBreedSpecies', () => ({
  useBreedSpecies: () => ({ breedSpecies: [] }),
}))

// 於 vi.mock 之後 import，確保元件拿到的是 mock 版本
const { AnimalAddDialog } = await import('../components/AnimalAddDialog')
type NewAnimalForm = React.ComponentProps<typeof AnimalAddDialog>['newAnimal']

let previousLanguage: string

beforeAll(async () => {
  previousLanguage = i18n.language
  await i18n.changeLanguage('zh-TW')
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

const EMPTY_FORM: NewAnimalForm = {
  ear_tag: '',
  species_id: '',
  gender: 'male',
  source_id: '',
  entry_date: '2026-07-27',
  entry_weight: '',
  birth_date: '',
  pre_experiment_code: '',
  remark: '',
  breed_other: '',
}

function renderDialog(onPenCodeChange = vi.fn()) {
  render(
    <AnimalAddDialog
      open
      onOpenChange={() => {}}
      newAnimal={EMPTY_FORM}
      onNewAnimalChange={() => {}}
      penBuilding={BUILDING.code}
      onPenBuildingChange={() => {}}
      penZone={ZONE.code}
      onPenZoneChange={() => {}}
      penCode=""
      onPenCodeChange={onPenCodeChange}
      sourcesData={[]}
      onSubmit={() => {}}
      isPending={false}
    />
  )
  return { onPenCodeChange }
}

describe('新增動物 — 欄位編號下拉', () => {
  it('單字元 pen code 的選項可以被渲染出來', () => {
    renderDialog()

    fireEvent.click(screen.getByText('選擇編號'))

    expect(screen.getByRole('option', { name: 'Q' })).toBeInTheDocument()
  })

  it('選取後回傳完整 pen code，供 pen_location 直接使用', () => {
    const { onPenCodeChange } = renderDialog()

    fireEvent.click(screen.getByText('選擇編號'))
    fireEvent.click(screen.getByRole('option', { name: 'Q' }))

    // 這是真正會紅的那一條：舊寫法下 value 是 ''，Radix 視為未選取，
    // 點擊完全不會觸發 onValueChange（呼叫次數 0），欄位編號永遠填不起來。
    expect(onPenCodeChange).toHaveBeenCalledWith('Q')
  })
})
