import type { PaginatedResponse } from '@/types/common'

export interface DemoEquipment {
  id: string
  name: string
  model: string | null
  serial_number: string | null
  location: string | null
  notes: string | null
  is_active: boolean
  status: string
  calibration_type: string | null
  calibration_cycle: string | null
  inspection_cycle: string | null
}

export interface DemoCalibration {
  id: string
  equipment_id: string
  equipment_name: string
  equipment_serial_number: string | null
  calibration_type: string
  calibrated_at: string
  next_due_at: string | null
  result: string | null
  notes: string | null
  partner_id: string | null
  partner_name: string | null
  report_number: string | null
  inspector: string | null
  created_at: string
}

const demoEquipment: DemoEquipment[] = [
  {
    id: 'demo-eq1', name: '範例電子秤', model: 'XS-205', serial_number: 'EQ-001',
    location: '實驗室 A', notes: null, is_active: true, status: 'active',
    calibration_type: 'calibration', calibration_cycle: 'annual', inspection_cycle: null,
  },
  {
    id: 'demo-eq2', name: '範例氣體麻醉機', model: 'AS-100', serial_number: 'EQ-002',
    location: '手術室', notes: null, is_active: true, status: 'active',
    calibration_type: 'validation', calibration_cycle: 'annual', inspection_cycle: null,
  },
  {
    id: 'demo-eq3', name: '範例低溫冷凍櫃', model: 'FR-80', serial_number: 'EQ-003',
    location: '儲藏室', notes: null, is_active: true, status: 'active',
    calibration_type: 'validation', calibration_cycle: 'semi_annual', inspection_cycle: null,
  },
  {
    id: 'demo-eq4', name: '範例動物磅秤', model: 'ASC-50', serial_number: 'EQ-004',
    location: '動物房', notes: null, is_active: true, status: 'active',
    calibration_type: 'inspection', calibration_cycle: 'quarterly', inspection_cycle: 'quarterly',
  },
]

export const DEMO_EQUIPMENT_PAGINATED: PaginatedResponse<DemoEquipment> = {
  data: demoEquipment,
  total: 4, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_EQUIPMENT_ALL: PaginatedResponse<DemoEquipment> = {
  data: demoEquipment,
  total: 4, page: 1, per_page: 500, total_pages: 1,
}

export const DEMO_CALIBRATIONS: PaginatedResponse<DemoCalibration> = {
  data: [
    {
      id: 'demo-cal1', equipment_id: 'demo-eq1', equipment_name: '範例電子秤',
      equipment_serial_number: 'EQ-001', calibration_type: 'calibration',
      calibrated_at: '2026-01-15T08:00:00Z', next_due_at: '2027-01-15T08:00:00Z',
      result: '合格', notes: null, partner_id: 'demo-partner1',
      partner_name: '範例校正廠商', report_number: 'CAL-2026-001',
      inspector: '範例校正員', created_at: '2026-01-15T08:00:00Z',
    },
    {
      id: 'demo-cal2', equipment_id: 'demo-eq4', equipment_name: '範例動物磅秤',
      equipment_serial_number: 'EQ-004', calibration_type: 'inspection',
      calibrated_at: '2026-03-01T08:00:00Z', next_due_at: '2026-06-01T08:00:00Z',
      result: '合格', notes: null, partner_id: null, partner_name: null,
      report_number: null, inspector: '範例內部查核員', created_at: '2026-03-01T08:00:00Z',
    },
  ],
  total: 2, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_ANNUAL_PLANS = [
  {
    id: 'demo-plan1', year: 2026, equipment_id: 'demo-eq1',
    equipment_name: '範例電子秤', equipment_serial_number: 'EQ-001',
    calibration_type: 'calibration' as const, cycle: 'annual' as const,
    month_1: true, month_2: false, month_3: false, month_4: false,
    month_5: false, month_6: false, month_7: false, month_8: false,
    month_9: false, month_10: false, month_11: false, month_12: false,
    generated_at: '2026-01-01T08:00:00Z',
  },
  {
    id: 'demo-plan2', year: 2026, equipment_id: 'demo-eq4',
    equipment_name: '範例動物磅秤', equipment_serial_number: 'EQ-004',
    calibration_type: 'inspection' as const, cycle: 'quarterly' as const,
    month_1: false, month_2: true, month_3: false, month_4: false,
    month_5: true, month_6: false, month_7: false, month_8: true,
    month_9: false, month_10: false, month_11: true, month_12: false,
    generated_at: '2026-01-01T08:00:00Z',
  },
]
