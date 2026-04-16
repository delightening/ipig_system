import type { PaginatedResponse } from '@/types/common'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

export interface DemoAnimalListItem {
  id: string
  ear_tag: string
  status: string
  breed: string
  gender: string
  entry_date: string
  pen_location?: string
  species_name?: string
  latest_weight?: number
  latest_weight_date?: string
  is_on_medication?: boolean
  has_abnormal_record?: boolean
  iacuc_no?: string
  created_at: string
  updated_at: string
}

const demoAnimals: DemoAnimalListItem[] = [
  {
    id: 'demo-a1', ear_tag: 'D-001', status: 'in_experiment', breed: 'minipig',
    gender: 'female', entry_date: '2025-11-15', pen_location: 'A-01',
    species_name: '迷你豬', latest_weight: 28.5, latest_weight_date: '2026-03-28',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a2', ear_tag: 'D-002', status: 'in_experiment', breed: 'minipig',
    gender: 'male', entry_date: '2025-11-15', pen_location: 'A-02',
    species_name: '迷你豬', latest_weight: 32.1, latest_weight_date: '2026-03-28',
    is_on_medication: true, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a3', ear_tag: 'D-003', status: 'unassigned', breed: 'lyd',
    gender: 'female', entry_date: '2026-01-10', pen_location: 'B-03',
    species_name: 'LYD', latest_weight: 45.0, latest_weight_date: '2026-03-25',
    is_on_medication: false, has_abnormal_record: false,
    created_at: '2026-01-10T08:00:00Z', updated_at: '2026-03-25T10:00:00Z',
  },
  {
    id: 'demo-a4', ear_tag: 'D-004', status: 'completed', breed: 'white',
    gender: 'male', entry_date: '2025-09-01', pen_location: 'C-01',
    species_name: '白豬', latest_weight: 55.2, latest_weight_date: '2026-02-20',
    is_on_medication: false, has_abnormal_record: true, iacuc_no: 'IACUC-2024-012',
    created_at: '2025-09-01T08:00:00Z', updated_at: '2026-02-20T10:00:00Z',
  },
  {
    id: 'demo-a5', ear_tag: 'D-005', status: 'in_experiment', breed: 'minipig',
    gender: 'female', entry_date: '2026-02-01', pen_location: 'A-03',
    species_name: '迷你豬', latest_weight: 22.8, latest_weight_date: '2026-03-30',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-003',
    created_at: '2026-02-01T08:00:00Z', updated_at: '2026-03-30T10:00:00Z',
  },
]

export const DEMO_ANIMALS_PAGINATED: PaginatedResponse<DemoAnimalListItem> = {
  data: demoAnimals,
  total: 5,
  page: 1,
  per_page: 20,
  total_pages: 1,
}

export const DEMO_ANIMALS_ALL = demoAnimals

export const DEMO_ANIMAL_STATS = {
  status_counts: {
    in_experiment: 3,
    unassigned: 1,
    completed: 1,
  } as Record<string, number>,
  pen_animals_count: 5,
  total: 5,
}

// ============================================
// 設施 Demo 資料（建物 / 區域 / 欄位）
// 供 useFacilityLayout 在訪客模式下使用
// ============================================

const DEMO_FACILITY_ID = 'demo-facility-1'
const DEMO_FACILITY_CODE = 'DEMO'
const DEMO_FACILITY_NAME = '範例動物實驗設施'
const DEMO_BUILDING_ID = 'demo-building-1'
const DEMO_BUILDING_CODE = 'A'
const DEMO_BUILDING_NAME = '範例動物房'

export const DEMO_BUILDINGS: BuildingWithFacility[] = [
  {
    id: DEMO_BUILDING_ID,
    facility_id: DEMO_FACILITY_ID,
    facility_code: DEMO_FACILITY_CODE,
    facility_name: DEMO_FACILITY_NAME,
    code: DEMO_BUILDING_CODE,
    name: DEMO_BUILDING_NAME,
    description: '訪客 Demo 動物房',
    is_active: true,
    config: null,
    sort_order: 1,
  },
]

export const DEMO_ZONES: ZoneWithBuilding[] = [
  {
    id: 'demo-zone-a',
    building_id: DEMO_BUILDING_ID,
    building_code: DEMO_BUILDING_CODE,
    building_name: DEMO_BUILDING_NAME,
    facility_id: DEMO_FACILITY_ID,
    facility_name: DEMO_FACILITY_NAME,
    code: 'A',
    name: 'A 區',
    color: 'blue',
    is_active: true,
    layout_config: null,
    sort_order: 1,
  },
  {
    id: 'demo-zone-b',
    building_id: DEMO_BUILDING_ID,
    building_code: DEMO_BUILDING_CODE,
    building_name: DEMO_BUILDING_NAME,
    facility_id: DEMO_FACILITY_ID,
    facility_name: DEMO_FACILITY_NAME,
    code: 'B',
    name: 'B 區',
    color: 'green',
    is_active: true,
    layout_config: null,
    sort_order: 2,
  },
  {
    id: 'demo-zone-c',
    building_id: DEMO_BUILDING_ID,
    building_code: DEMO_BUILDING_CODE,
    building_name: DEMO_BUILDING_NAME,
    facility_id: DEMO_FACILITY_ID,
    facility_name: DEMO_FACILITY_NAME,
    code: 'C',
    name: 'C 區',
    color: 'amber',
    is_active: true,
    layout_config: null,
    sort_order: 3,
  },
]

/** 建立欄位 helper */
function makePen(
  id: string,
  code: string,
  zoneId: string,
  zoneCode: string,
  zoneName: string,
  zoneColor: string,
  rowIndex: number,
  colIndex: number,
  currentCount: number,
): PenDetails {
  return {
    id,
    code,
    name: null,
    capacity: 0,
    current_count: currentCount,
    status: currentCount > 0 ? 'active' : 'empty',
    row_index: rowIndex,
    col_index: colIndex,
    zone_id: zoneId,
    zone_code: zoneCode,
    zone_name: zoneName,
    zone_color: zoneColor,
    zone_layout_config: null,
    building_id: DEMO_BUILDING_ID,
    building_code: DEMO_BUILDING_CODE,
    building_name: DEMO_BUILDING_NAME,
    facility_id: DEMO_FACILITY_ID,
    facility_code: DEMO_FACILITY_CODE,
    facility_name: DEMO_FACILITY_NAME,
  }
}

// Zone A (2 rows × 2 cols)：A-01 A-02 A-03 A-04
// demo-a1 → A-01, demo-a2 → A-02, demo-a5 → A-03
export const DEMO_PENS: PenDetails[] = [
  makePen('demo-pen-a01', 'A-01', 'demo-zone-a', 'A', 'A 區', 'blue', 0, 0, 1),
  makePen('demo-pen-a02', 'A-02', 'demo-zone-a', 'A', 'A 區', 'blue', 0, 1, 1),
  makePen('demo-pen-a03', 'A-03', 'demo-zone-a', 'A', 'A 區', 'blue', 1, 0, 1),
  makePen('demo-pen-a04', 'A-04', 'demo-zone-a', 'A', 'A 區', 'blue', 1, 1, 0),
  // Zone B (2 rows × 2 cols)：B-01 B-02 B-03 B-04
  // demo-a3 → B-03
  makePen('demo-pen-b01', 'B-01', 'demo-zone-b', 'B', 'B 區', 'green', 0, 0, 0),
  makePen('demo-pen-b02', 'B-02', 'demo-zone-b', 'B', 'B 區', 'green', 0, 1, 0),
  makePen('demo-pen-b03', 'B-03', 'demo-zone-b', 'B', 'B 區', 'green', 1, 0, 1),
  makePen('demo-pen-b04', 'B-04', 'demo-zone-b', 'B', 'B 區', 'green', 1, 1, 0),
  // Zone C (2 rows × 2 cols)：C-01 C-02 C-03 C-04
  // demo-a4 → C-01
  makePen('demo-pen-c01', 'C-01', 'demo-zone-c', 'C', 'C 區', 'amber', 0, 0, 1),
  makePen('demo-pen-c02', 'C-02', 'demo-zone-c', 'C', 'C 區', 'amber', 0, 1, 0),
  makePen('demo-pen-c03', 'C-03', 'demo-zone-c', 'C', 'C 區', 'amber', 1, 0, 0),
  makePen('demo-pen-c04', 'C-04', 'demo-zone-c', 'C', 'C 區', 'amber', 1, 1, 0),
]

// ============================================

export const DEMO_ANIMALS_BY_PEN = [
  {
    pen_location: 'A-01',
    animals: [demoAnimals[0]],
  },
  {
    pen_location: 'A-02',
    animals: [demoAnimals[1]],
  },
  {
    pen_location: 'A-03',
    animals: [demoAnimals[4]],
  },
  {
    pen_location: 'B-03',
    animals: [demoAnimals[2]],
  },
  {
    pen_location: 'C-01',
    animals: [demoAnimals[3]],
  },
]
