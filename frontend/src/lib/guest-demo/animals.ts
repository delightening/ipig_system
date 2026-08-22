import type { PaginatedResponse } from '@/types/common'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

export interface DemoAnimalListItem {
  id: string
  ear_tag: string
  status: string
  breed: string
  gender: string
  birth_date?: string
  entry_date: string
  entry_weight?: string
  source_id?: string
  source_name?: string
  pen_location?: string
  species_name?: string
  latest_weight?: number
  latest_weight_date?: string
  is_on_medication?: boolean
  has_abnormal_record?: boolean
  iacuc_no?: string
  remark?: string
  created_at: string
  updated_at: string
}

// 民間豬場（demo source；範例不指名道姓）
export const DEMO_ANIMAL_SOURCES = [
  {
    id: 'demo-source-1', name: '民間豬場 A', code: 'FARM-A',
    contact_person: '範例聯絡人', contact_phone: '範例電話',
    address: '範例地址', remark: 'SPF 級迷你豬供應商（範例）',
    is_active: true, created_at: '2024-01-01T08:00:00Z',
  },
  {
    id: 'demo-source-2', name: '民間豬場 B', code: 'FARM-B',
    contact_person: '範例聯絡人', contact_phone: '範例電話',
    address: '範例地址', remark: 'LYD / 白豬供應商（範例）',
    is_active: true, created_at: '2024-01-01T08:00:00Z',
  },
]

// 故事線：AUP-2025-001 心血管藥物安全性評估（APPROVED 2025-11-01）
// → 2025-11-15 D-001/D-002 進場（5/6 月齡，22/28 kg）
// → 2025-11-20 ~ 12-05 隔離疫苗驅蟲
// → 2026-01-05 in_experiment
// → 2026-01-15 D-001 左眼手術
const demoAnimals: DemoAnimalListItem[] = [
  {
    id: 'demo-a1', ear_tag: 'D-001', status: 'in_experiment', breed: 'minipig',
    gender: 'female',
    birth_date: '2025-06-15',  // 5 個月齡進場
    entry_date: '2025-11-15',
    entry_weight: '22.0',
    source_id: 'demo-source-1', source_name: '民間豬場 A',
    pen_location: 'A-01',
    species_name: '迷你豬', latest_weight: 28.5, latest_weight_date: '2026-03-28',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    remark: '心血管藥物試驗主受試動物。性情溫馴，採血順利',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a2', ear_tag: 'D-002', status: 'in_experiment', breed: 'minipig',
    gender: 'male',
    birth_date: '2025-05-15',  // 6 個月齡進場
    entry_date: '2025-11-15',
    entry_weight: '28.0',
    source_id: 'demo-source-1', source_name: '民間豬場 A',
    pen_location: 'A-02',
    species_name: '迷你豬', latest_weight: 32.1, latest_weight_date: '2026-03-28',
    is_on_medication: true, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    remark: '同窩 D-001。長期投予試驗藥物中，每週採血',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a3', ear_tag: 'D-003', status: 'unassigned', breed: 'lyd',
    gender: 'female',
    birth_date: '2025-09-10',  // 4 個月齡進場
    entry_date: '2026-01-10',
    entry_weight: '24.5',
    source_id: 'demo-source-2', source_name: '民間豬場 B',
    pen_location: 'B-03',
    species_name: 'LYD', latest_weight: 28.0, latest_weight_date: '2026-03-25',
    is_on_medication: false, has_abnormal_record: false,
    remark: '尚未指派計畫，目前隔離觀察中',
    created_at: '2026-01-10T08:00:00Z', updated_at: '2026-03-25T10:00:00Z',
  },
  {
    id: 'demo-a4', ear_tag: 'D-004', status: 'completed', breed: 'white',
    gender: 'male',
    birth_date: '2025-04-01',  // 5 月齡進場（IACUC-2024-012 試驗已結束）
    entry_date: '2025-09-01',
    entry_weight: '25.0',
    source_id: 'demo-source-2', source_name: '民間豬場 B',
    pen_location: 'C-01',
    species_name: '白豬', latest_weight: 55.2, latest_weight_date: '2026-02-20',
    is_on_medication: false, has_abnormal_record: true, iacuc_no: 'IACUC-2024-012',
    remark: '骨科植入物試驗已結案 (2026-02-20)，保留歷史紀錄供參考',
    created_at: '2025-09-01T08:00:00Z', updated_at: '2026-02-20T10:00:00Z',
  },
  {
    id: 'demo-a5', ear_tag: 'D-005', status: 'in_experiment', breed: 'minipig',
    gender: 'female',
    birth_date: '2025-09-25',  // 4 月齡進場
    entry_date: '2026-02-01',
    entry_weight: '20.5',
    source_id: 'demo-source-1', source_name: '民間豬場 A',
    pen_location: 'A-03',
    species_name: '迷你豬', latest_weight: 22.8, latest_weight_date: '2026-03-30',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-003',
    remark: '新型疫苗免疫反應研究受試動物（IACUC-2025-003）',
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
  // demo 只有一棟（code 'A'），5 隻全在該棟
  pen_counts_by_building: { A: 5 } as Record<string, number>,
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

// ============================================
// Demo 詳細紀錄 — 病歷 / 手術 / 體重 / 疫苗 / 觀察
// 供 animal detail 頁面顯示豐富內容（每個動物有完整 timeline）
// ============================================

// D-001/D-002 觀察 timeline（依時序排）：
// 進場 11-15 → 隔離期 11-20~12-31 → 進入實驗 1-05 → 手術後恢復觀察 → 持續追蹤
export const DEMO_ANIMAL_OBSERVATIONS = [
  // 入場隔離期觀察
  {
    id: 'demo-obs-1', animal_id: 'demo-a1',
    event_date: '2025-11-20', record_type: 'normal',
    content: '進場第 5 天，適應新環境良好，食慾正常，無腹瀉/咳嗽',
    no_medication_needed: true, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2025-11-20T09:00:00Z',
    updated_at: '2025-11-20T09:00:00Z',
  },
  {
    id: 'demo-obs-2', animal_id: 'demo-a1',
    event_date: '2025-12-15', record_type: 'normal',
    content: '隔離觀察期滿一個月，所有臨床檢查（HR/RR/體溫）正常',
    no_medication_needed: true, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2025-12-15T10:00:00Z',
    updated_at: '2025-12-15T10:00:00Z',
  },
  // 術後觀察
  {
    id: 'demo-obs-3', animal_id: 'demo-a1',
    event_date: '2026-01-20', record_type: 'normal',
    content: '左眼術後第 5 天，傷口癒合良好，無紅腫滲液。已恢復正常活動',
    no_medication_needed: true, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2026-01-20T09:00:00Z',
    updated_at: '2026-01-20T09:00:00Z',
  },
  {
    id: 'demo-obs-4', animal_id: 'demo-a1',
    event_date: '2026-03-15', record_type: 'normal',
    content: '左眼角膜清澈無分泌物；皮毛光澤；步態平穩。每日活動量正常',
    no_medication_needed: true, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2026-03-15T09:00:00Z',
    updated_at: '2026-03-15T09:00:00Z',
  },
  {
    id: 'demo-obs-5', animal_id: 'demo-a1',
    event_date: '2026-03-28', record_type: 'normal',
    content: '精神狀態良好，食慾正常，體重增加符合預期曲線（22→28.5 kg）',
    no_medication_needed: true, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2026-03-28T10:30:00Z',
    updated_at: '2026-03-28T10:30:00Z',
  },
  // D-002 異常事件
  {
    id: 'demo-obs-6', animal_id: 'demo-a2',
    event_date: '2026-03-25', record_type: 'abnormal',
    content: '輕微跛行（左後肢），可能與昨日採血保定有關。觀察中，未投藥',
    no_medication_needed: false, stop_medication: false, vet_read: true,
    created_by_name: '範例獸醫', created_at: '2026-03-25T14:00:00Z',
    updated_at: '2026-03-25T14:00:00Z',
  },
]

// D-001 體重曲線：進場 22 kg → 4.5 個月增長到 28.5 kg（迷你豬正常曲線）
export const DEMO_ANIMAL_WEIGHTS = [
  { id: 'demo-w-1', animal_id: 'demo-a1', measure_date: '2026-03-28', weight: '28.5',
    created_by_name: '範例實驗員', created_at: '2026-03-28T10:00:00Z' },
  { id: 'demo-w-2', animal_id: 'demo-a1', measure_date: '2026-03-14', weight: '27.8',
    created_by_name: '範例實驗員', created_at: '2026-03-14T10:00:00Z' },
  { id: 'demo-w-3', animal_id: 'demo-a1', measure_date: '2026-02-28', weight: '26.5',
    created_by_name: '範例實驗員', created_at: '2026-02-28T10:00:00Z' },
  { id: 'demo-w-4', animal_id: 'demo-a1', measure_date: '2026-02-14', weight: '25.2',
    created_by_name: '範例實驗員', created_at: '2026-02-14T10:00:00Z' },
  { id: 'demo-w-5', animal_id: 'demo-a1', measure_date: '2026-01-15', weight: '24.0',
    created_by_name: '範例實驗員', created_at: '2026-01-15T08:00:00Z' },
  { id: 'demo-w-6', animal_id: 'demo-a1', measure_date: '2025-12-15', weight: '23.0',
    created_by_name: '範例實驗員', created_at: '2025-12-15T10:00:00Z' },
  { id: 'demo-w-7', animal_id: 'demo-a1', measure_date: '2025-11-15', weight: '22.0',
    created_by_name: '範例實驗員', created_at: '2025-11-15T10:00:00Z' },
]

export const DEMO_ANIMAL_VACCINATIONS = [
  { id: 'demo-vac-1', animal_id: 'demo-a1', administered_date: '2025-11-20',
    vaccine: '豬瘟活毒疫苗（PRRS）', deworming_dose: '',
    created_by_name: '範例獸醫', created_at: '2025-11-20T09:00:00Z' },
  { id: 'demo-vac-2', animal_id: 'demo-a1', administered_date: '2025-12-05',
    vaccine: '', deworming_dose: 'Ivermectin 0.3 mg/kg 皮下注射',
    created_by_name: '範例獸醫', created_at: '2025-12-05T09:00:00Z' },
  { id: 'demo-vac-3', animal_id: 'demo-a1', administered_date: '2026-02-10',
    vaccine: '', deworming_dose: 'Doramectin 0.2 mg/kg 肌肉注射',
    created_by_name: '範例獸醫', created_at: '2026-02-10T09:00:00Z' },
]

export const DEMO_ANIMAL_SURGERIES = [
  {
    id: 'demo-surg-1', animal_id: 'demo-a1',
    is_first_experiment: true, surgery_date: '2026-01-15',
    surgery_site: '左眼',
    induction_anesthesia: { atropine: { enabled: true, dose: '0.05 ml/kg IM' }, zoletil50: { enabled: true, dose: '5 mg/kg IM' } },
    pre_surgery_medication: [
      { name: 'Cefazolin', dose: '20 mg/kg IV' },
    ],
    positioning: '右側臥姿，頭部抬高 15 度',
    anesthesia_maintenance: { isoflurane: { enabled: true, dose: '1.5–2% in 100% O₂' } },
    anesthesia_observation: '麻醉誘導平穩，維持期心率 80–95 bpm，SpO₂ 98%+',
    vital_signs: [
      { time: '09:00', breathing_method: '自主呼吸', heart_rate: '92', respiration_rate: '18', temperature: '38.5', spo2: '99' },
      { time: '09:15', breathing_method: '自主呼吸', heart_rate: '88', respiration_rate: '16', temperature: '38.4', spo2: '99' },
      { time: '09:30', breathing_method: '自主呼吸', heart_rate: '85', respiration_rate: '15', temperature: '38.3', spo2: '98' },
      { time: '09:45', breathing_method: '自主呼吸', heart_rate: '90', respiration_rate: '17', temperature: '38.4', spo2: '99' },
    ],
    reflex_recovery: '麻醉結束 12 分鐘恢復吞嚥反射，20 分鐘恢復角膜反射',
    respiration_rate: 18,
    post_surgery_medication: [
      { name: 'Meloxicam', dose: '0.4 mg/kg SC' },
      { name: 'Cefazolin', dose: '20 mg/kg IV q12h × 3 天' },
    ],
    remark: '手術順利，術中無大量出血，術野清潔縫合',
    no_medication_needed: false, vet_read: true,
    created_by_name: '範例獸醫',
    created_at: '2026-01-15T09:00:00Z', updated_at: '2026-01-15T11:00:00Z',
  },
]

export const DEMO_ANIMAL_PAIN_ASSESSMENTS = [
  {
    id: 'demo-pain-1', record_type: 'surgery', record_id: 'demo-surg-1',
    record_mode: 'pain_assessment',
    post_op_days: 1, time_period: 'morning',
    incision: 1, attitude_behavior: 1, appetite: 0, feces: 0, urine: 0, pain_score: 1,
    injection_ketorolac: false, injection_meloxicam: true, oral_meloxicam: false,
    post_medications: [],
    vet_read: true, created_at: '2026-01-16T09:00:00Z',
  },
  {
    id: 'demo-pain-2', record_type: 'surgery', record_id: 'demo-surg-1',
    record_mode: 'pain_assessment',
    post_op_days: 2, time_period: 'morning',
    incision: 1, attitude_behavior: 0, appetite: 0, feces: 0, urine: 0, pain_score: 1,
    injection_ketorolac: false, injection_meloxicam: false, oral_meloxicam: true,
    post_medications: [],
    vet_read: true, created_at: '2026-01-17T09:00:00Z',
  },
  {
    id: 'demo-pain-3', record_type: 'surgery', record_id: 'demo-surg-1',
    record_mode: 'pain_assessment',
    post_op_days: 3, time_period: 'morning',
    incision: 0, attitude_behavior: 0, appetite: 0, feces: 0, urine: 0, pain_score: 0,
    injection_ketorolac: false, injection_meloxicam: false, oral_meloxicam: false,
    post_medications: [],
    vet_read: true, created_at: '2026-01-18T09:00:00Z',
  },
]

// R47 可用豬隻快速查詢（庫存盤點）demo 資料
export const DEMO_AVAILABLE_PIGS = {
  animals: [
    {
      id: 'demo-ap-1', ear_tag: 'A-101', animal_no: 'AP-2026-001',
      breed: 'minipig', gender: 'male',
      birth_date: '2024-01-15', age_months: 28,
      latest_weight_kg: '32.5', weight_measured_at: '2026-05-08',
      pen_location: 'A03',
    },
    {
      id: 'demo-ap-2', ear_tag: 'A-102', animal_no: 'AP-2026-002',
      breed: 'minipig', gender: 'female',
      birth_date: '2024-02-20', age_months: 27,
      latest_weight_kg: '28.7', weight_measured_at: '2026-05-08',
      pen_location: 'A03',
    },
    {
      id: 'demo-ap-3', ear_tag: 'B-201', animal_no: 'AP-2026-003',
      breed: 'minipig', gender: 'male',
      birth_date: '2024-04-10', age_months: 25,
      latest_weight_kg: '30.1', weight_measured_at: '2026-05-10',
      pen_location: 'B12',
    },
    {
      id: 'demo-ap-4', ear_tag: 'B-202', animal_no: 'AP-2026-004',
      breed: 'minipig', gender: 'female',
      birth_date: '2024-05-05', age_months: 24,
      latest_weight_kg: '26.4', weight_measured_at: '2026-05-10',
      pen_location: 'B12',
    },
    {
      id: 'demo-ap-5', ear_tag: 'D-301', animal_no: 'AP-2026-005',
      breed: 'white', gender: 'male',
      birth_date: '2023-09-12', age_months: 32,
      latest_weight_kg: '45.8', weight_measured_at: '2026-05-12',
      pen_location: 'D08',
    },
    {
      id: 'demo-ap-6', ear_tag: 'D-302', animal_no: 'AP-2026-006',
      breed: 'white', gender: 'female',
      birth_date: '2023-10-01', age_months: 31,
      latest_weight_kg: '42.3', weight_measured_at: '2026-05-12',
      pen_location: 'D08',
    },
    {
      id: 'demo-ap-7', ear_tag: 'C-401', animal_no: 'AP-2026-007',
      breed: 'lyd', gender: 'male',
      birth_date: '2024-06-18', age_months: 23,
      latest_weight_kg: '38.5', weight_measured_at: '2026-05-11',
      pen_location: 'C05',
    },
    {
      id: 'demo-ap-8', ear_tag: 'C-402', animal_no: 'AP-2026-008',
      breed: 'lyd', gender: 'female',
      birth_date: '2024-07-22', age_months: 22,
      latest_weight_kg: '35.2', weight_measured_at: '2026-05-11',
      pen_location: 'C05',
    },
  ],
  summary: {
    total: 8, male: 4, female: 4,
    by_breed: { '迷你豬': 4, '白豬': 2, 'LYD': 2 },
    excluded_weight_expired: 0,
  },
  page: 1, per_page: 20,
}

