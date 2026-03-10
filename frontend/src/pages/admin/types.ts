/**
 * 設備與校正紀錄頁面共用型別
 */

export interface Equipment {
  id: string
  name: string
  model: string | null
  serial_number: string | null
  location: string | null
  notes: string | null
  is_active: boolean
}

export interface CalibrationWithEquipment {
  id: string
  equipment_id: string
  equipment_name: string
  calibrated_at: string
  next_due_at: string | null
  result: string | null
  notes: string | null
  created_at: string
}

export interface EquipmentForm {
  name: string
  model: string
  serial_number: string
  location: string
  notes: string
}

export interface CalibrationForm {
  equipment_id: string
  calibrated_at: string
  next_due_at: string
  result: string
  notes: string
}
