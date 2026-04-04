/**
 * 設備維護管理頁面共用型別
 */

export type EquipmentStatus = 'active' | 'inactive' | 'under_repair' | 'decommissioned'
export type CalibrationType = 'calibration' | 'validation' | 'inspection'
export type CalibrationCycle = 'monthly' | 'quarterly' | 'semi_annual' | 'annual'
export type MaintenanceType = 'repair' | 'maintenance'
export type MaintenanceStatus = 'pending' | 'in_progress' | 'completed' | 'unrepairable' | 'pending_review'
export type DisposalStatus = 'pending' | 'approved' | 'rejected'
/** GMP/GLP 確效階段：IQ安裝確效 / OQ作業確效 / PQ效能確效 */
export type ValidationPhase = 'IQ' | 'OQ' | 'PQ'

export const EQUIPMENT_STATUS_LABELS: Record<EquipmentStatus, string> = {
  active: '啟用',
  inactive: '停用',
  under_repair: '維修',
  decommissioned: '報廢',
}

export const CALIBRATION_TYPE_LABELS: Record<CalibrationType, string> = {
  calibration: '校正',
  validation: '確效',
  inspection: '查核',
}

export const CALIBRATION_CYCLE_LABELS: Record<CalibrationCycle, string> = {
  monthly: '每月',
  quarterly: '每季',
  semi_annual: '每半年',
  annual: '每年',
}

export const MAINTENANCE_TYPE_LABELS: Record<MaintenanceType, string> = {
  repair: '維修',
  maintenance: '保養',
}

export const MAINTENANCE_STATUS_LABELS: Record<MaintenanceStatus, string> = {
  pending: '待處理',
  in_progress: '進行中',
  pending_review: '待驗收',
  completed: '已完成',
  unrepairable: '無法維修',
}

export const DISPOSAL_STATUS_LABELS: Record<DisposalStatus, string> = {
  pending: '待核准',
  approved: '已核准',
  rejected: '已駁回',
}

export const VALIDATION_PHASE_LABELS: Record<ValidationPhase, string> = {
  IQ: 'IQ 安裝確效',
  OQ: 'OQ 作業確效',
  PQ: 'PQ 效能確效',
}

export interface Equipment {
  id: string
  name: string
  model: string | null
  serial_number: string | null
  location: string | null
  department: string | null
  purchase_date: string | null
  warranty_expiry: string | null
  notes: string | null
  is_active: boolean
  status: EquipmentStatus
  calibration_type: CalibrationType | null
  calibration_cycle: CalibrationCycle | null
  inspection_cycle: CalibrationCycle | null
}

export interface CalibrationWithEquipment {
  id: string
  equipment_id: string
  equipment_name: string
  equipment_serial_number: string | null
  calibration_type: CalibrationType
  calibrated_at: string
  next_due_at: string | null
  result: string | null
  notes: string | null
  partner_id: string | null
  partner_name: string | null
  report_number: string | null
  inspector: string | null
  // ISO 17025 合規欄位
  certificate_number: string | null
  performed_by: string | null
  acceptance_criteria: string | null
  measurement_uncertainty: string | null
  // GMP/GLP 確效欄位
  validation_phase: ValidationPhase | null
  protocol_number: string | null
  created_at: string
}

export interface EquipmentSupplierWithPartner {
  id: string
  equipment_id: string
  partner_id: string
  partner_name: string
  contact_person: string | null
  contact_phone: string | null
  contact_email: string | null
  notes: string | null
  partner_phone: string | null
  partner_phone_ext: string | null
  partner_email: string | null
  partner_address: string | null
}

export interface MaintenanceRecordWithDetails {
  id: string
  equipment_id: string
  equipment_name: string
  maintenance_type: MaintenanceType
  status: MaintenanceStatus
  reported_at: string
  completed_at: string | null
  problem_description: string | null
  repair_content: string | null
  repair_partner_id: string | null
  repair_partner_name: string | null
  maintenance_items: string | null
  performed_by: string | null
  notes: string | null
  created_by: string
  reviewed_by: string | null
  reviewer_name: string | null
  reviewed_at: string | null
  review_notes: string | null
  created_at: string
}

export interface DisposalWithDetails {
  id: string
  equipment_id: string
  equipment_name: string
  status: DisposalStatus
  disposal_date: string | null
  reason: string
  disposal_method: string | null
  applied_by: string
  applicant_name: string
  applied_at: string
  approved_by: string | null
  approver_name: string | null
  approved_at: string | null
  rejection_reason: string | null
  notes: string | null
}

export interface AnnualPlanWithEquipment {
  id: string
  year: number
  equipment_id: string
  equipment_name: string
  equipment_serial_number: string | null
  calibration_type: CalibrationType
  cycle: CalibrationCycle
  month_1: boolean
  month_2: boolean
  month_3: boolean
  month_4: boolean
  month_5: boolean
  month_6: boolean
  month_7: boolean
  month_8: boolean
  month_9: boolean
  month_10: boolean
  month_11: boolean
  month_12: boolean
}

export type MonthExecutionStatus = 'unplanned' | 'planned_pending' | 'completed' | 'overdue'

export interface MonthExecutionDetail {
  month: number
  planned: boolean
  status: MonthExecutionStatus
  calibration_id: string | null
  calibrated_at: string | null
  result: string | null
}

export interface AnnualPlanExecutionRow {
  plan_id: string
  year: number
  equipment_id: string
  equipment_name: string
  equipment_serial_number: string | null
  calibration_type: CalibrationType
  cycle: CalibrationCycle
  months: MonthExecutionDetail[]
  planned_count: number
  completed_count: number
  overdue_count: number
}

export interface AnnualPlanExecutionSummary {
  year: number
  total_planned: number
  total_completed: number
  total_overdue: number
  completion_rate: number
  rows: AnnualPlanExecutionRow[]
}

export type TimelineEventType = 'maintenance' | 'calibration' | 'status_change'

export interface EquipmentTimelineEntry {
  id: string
  event_type: TimelineEventType
  occurred_at: string
  title: string
  subtitle: string | null
  detail: Record<string, unknown>
}

export interface EquipmentForm {
  name: string
  model: string
  serial_number: string
  location: string
  department: string
  purchase_date: string
  warranty_expiry: string
  notes: string
  calibration_type: CalibrationType | ''
  calibration_cycle: CalibrationCycle | ''
  inspection_cycle: CalibrationCycle | ''
}

export interface CalibrationForm {
  equipment_id: string
  calibration_type: CalibrationType
  calibrated_at: string
  next_due_at: string
  result: string
  notes: string
  partner_id: string
  report_number: string
  inspector: string
  // ISO 17025 合規欄位
  certificate_number: string
  performed_by: string
  acceptance_criteria: string
  measurement_uncertainty: string
  // GMP/GLP 確效欄位
  validation_phase: ValidationPhase | ''
  protocol_number: string
}
