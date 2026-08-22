/**
 * 設備維護管理頁面共用型別
 */
import type { StatusVariant } from '@/components/ui/status-badge'

export type EquipmentStatus = 'active' | 'inactive' | 'under_repair' | 'decommissioned'
export type CalibrationType = 'calibration' | 'validation' | 'inspection'
export type CalibrationCycle = 'monthly' | 'quarterly' | 'semi_annual' | 'annual'
export type MaintenanceType = 'repair' | 'maintenance'
export type MaintenanceStatus = 'pending' | 'in_progress' | 'completed' | 'unrepairable' | 'pending_review'
export type DisposalStatus = 'pending' | 'approved' | 'rejected'
/** GMP/GLP 確效階段：IQ安裝確效 / OQ作業確效 / PQ效能確效 */
export type ValidationPhase = 'IQ' | 'OQ' | 'PQ'

// 值為 i18n key（非顯示字串）；consumer 須以 t() 取顯示文字。
export const EQUIPMENT_STATUS_LABELS: Record<EquipmentStatus, string> = {
  active: 'admin.equipmentLabels.status.active',
  inactive: 'admin.equipmentLabels.status.inactive',
  under_repair: 'admin.equipmentLabels.status.under_repair',
  decommissioned: 'admin.equipmentLabels.status.decommissioned',
}

export const CALIBRATION_TYPE_LABELS: Record<CalibrationType, string> = {
  calibration: 'admin.calibrationLabels.type.calibration',
  validation: 'admin.calibrationLabels.type.validation',
  inspection: 'admin.calibrationLabels.type.inspection',
}

export const CALIBRATION_CYCLE_LABELS: Record<CalibrationCycle, string> = {
  monthly: 'admin.calibrationLabels.cycle.monthly',
  quarterly: 'admin.calibrationLabels.cycle.quarterly',
  semi_annual: 'admin.calibrationLabels.cycle.semi_annual',
  annual: 'admin.calibrationLabels.cycle.annual',
}

// 值為 i18n key（非顯示字串）；consumer 須以 t() 取顯示文字。
export const MAINTENANCE_TYPE_LABELS: Record<MaintenanceType, string> = {
  repair: 'admin.maintenanceLabels.type.repair',
  maintenance: 'admin.maintenanceLabels.type.maintenance',
}

export const MAINTENANCE_STATUS_LABELS: Record<MaintenanceStatus, string> = {
  pending: 'admin.maintenanceLabels.status.pending',
  in_progress: 'admin.maintenanceLabels.status.in_progress',
  pending_review: 'admin.maintenanceLabels.status.pending_review',
  completed: 'admin.maintenanceLabels.status.completed',
  unrepairable: 'admin.maintenanceLabels.status.unrepairable',
}

/** 維修狀態 → StatusBadge 色彩。保養不分狀態，故僅維修使用此表。 */
const REPAIR_STATUS_VARIANT: Record<MaintenanceStatus, StatusVariant> = {
  completed: 'success',
  unrepairable: 'error',
  pending_review: 'purple',
  in_progress: 'neutral',
  pending: 'warning',
}

/**
 * 維修/保養紀錄合併「類型+狀態」徽章規則（Dashboard widget 與設備管理頁共用）：
 * - 保養：恆顯示藍色「保養」，不分狀態（完成與否由「完修日期」欄判讀）。
 * - 維修：依狀態上色（完修綠／無法維修紅／待驗收紫／進行中灰／待處理亮黃）。
 * 回傳 labelKey 為 i18n key，consumer 須以 t() 取顯示文字。
 */
export function getMaintenanceBadge(
  type: MaintenanceType,
  status: MaintenanceStatus,
): { variant: StatusVariant; labelKey: string } {
  if (type === 'maintenance') {
    return { variant: 'info', labelKey: MAINTENANCE_TYPE_LABELS.maintenance }
  }
  return { variant: REPAIR_STATUS_VARIANT[status] ?? 'neutral', labelKey: MAINTENANCE_STATUS_LABELS[status] }
}

export const DISPOSAL_STATUS_LABELS: Record<DisposalStatus, string> = {
  pending: 'admin.disposalLabels.status.pending',
  approved: 'admin.disposalLabels.status.approved',
  rejected: 'admin.disposalLabels.status.rejected',
}

export const VALIDATION_PHASE_LABELS: Record<ValidationPhase, string> = {
  IQ: 'admin.validationLabels.phase.IQ',
  OQ: 'admin.validationLabels.phase.OQ',
  PQ: 'admin.validationLabels.phase.PQ',
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
  status: EquipmentStatus
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
