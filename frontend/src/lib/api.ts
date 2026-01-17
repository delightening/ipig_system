import axios, { AxiosError } from 'axios'

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor - add auth token
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('access_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor - handle errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as typeof error.config & { _retry?: boolean }

    // If 401 and not already retrying, try to refresh token
    if (error.response?.status === 401 && !originalRequest?._retry) {
      originalRequest._retry = true

      const refreshToken = localStorage.getItem('refresh_token')
      if (refreshToken) {
        try {
          const response = await api.post('/auth/refresh', {
            refresh_token: refreshToken,
          })

          const { access_token, refresh_token: newRefreshToken } = response.data
          localStorage.setItem('access_token', access_token)
          localStorage.setItem('refresh_token', newRefreshToken)

          if (originalRequest) {
            originalRequest.headers.Authorization = `Bearer ${access_token}`
            return api(originalRequest)
          }
        } catch {
          // Refresh failed, clear tokens and redirect to login (only once)
          if (!sessionStorage.getItem('auth_redirecting')) {
            sessionStorage.setItem('auth_redirecting', 'true')
            localStorage.removeItem('access_token')
            localStorage.removeItem('refresh_token')
            // Redirect immediately without clearing flag (cleared on login page)
            window.location.href = '/login'
          }
        }
      } else {
        // No refresh token, redirect to login (only once)
        if (!sessionStorage.getItem('auth_redirecting')) {
          sessionStorage.setItem('auth_redirecting', 'true')
          localStorage.removeItem('access_token')
          localStorage.removeItem('refresh_token')
          // Redirect immediately without clearing flag (cleared on login page)
          window.location.href = '/login'
        }
      }
    }

    return Promise.reject(error)
  }
)

// Utility function to format ear tag: if it's a number < 100, pad to 3 digits
export function formatEarTag(earTag: string): string {
  if (!earTag) return earTag
  // Check if it's a pure number
  if (/^\d+$/.test(earTag)) {
    const num = parseInt(earTag, 10)
    if (num < 100) {
      return earTag.padStart(3, '0')
    }
  }
  return earTag
}

export default api

// API Types
export interface User {
  id: string
  email: string
  display_name: string
  is_active: boolean
  roles: string[]
  permissions: string[]
  must_change_password?: boolean
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  token_type: string
  expires_in: number
  user: User
}

export interface Warehouse {
  id: string
  code: string
  name: string
  address?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface Product {
  id: string
  sku: string
  name: string
  spec?: string
  category_id?: string
  base_uom: string
  track_batch: boolean
  track_expiry: boolean
  safety_stock?: string
  reorder_point?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface Partner {
  id: string
  partner_type: 'supplier' | 'customer'
  code: string
  name: string
  tax_id?: string
  phone?: string
  email?: string
  address?: string
  payment_terms?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export type DocType = 'PO' | 'GRN' | 'PR' | 'SO' | 'DO' | 'TR' | 'STK' | 'ADJ' | 'RM'
export type DocStatus = 'draft' | 'submitted' | 'approved' | 'cancelled'

export interface DocumentLine {
  id: string
  document_id: string
  line_no: number
  product_id: string
  product_sku: string
  product_name: string
  qty: string
  uom: string
  unit_price?: string
  batch_no?: string
  expiry_date?: string
  remark?: string
}

export interface Document {
  id: string
  doc_type: DocType
  doc_no: string
  status: DocStatus
  warehouse_id?: string
  warehouse_from_id?: string
  warehouse_to_id?: string
  partner_id?: string
  doc_date: string
  remark?: string
  created_by: string
  approved_by?: string
  created_at: string
  updated_at: string
  approved_at?: string
  lines: DocumentLine[]
  warehouse_name?: string
  warehouse_from_name?: string
  warehouse_to_name?: string
  partner_name?: string
  created_by_name: string
  approved_by_name?: string
}

export interface DocumentListItem {
  id: string
  doc_type: DocType
  doc_no: string
  status: DocStatus
  warehouse_name?: string
  partner_name?: string
  doc_date: string
  created_by_name: string
  approved_by_name?: string
  created_at: string
  approved_at?: string
  line_count: number
  total_amount?: string
}

export interface InventoryOnHand {
  warehouse_id: string
  warehouse_code: string
  warehouse_name: string
  product_id: string
  product_sku: string
  product_name: string
  base_uom: string
  qty_on_hand: string
  avg_cost?: string
  safety_stock?: string
  reorder_point?: string
  last_updated_at?: string
}

export interface StockLedgerDetail {
  id: string
  warehouse_id: string
  warehouse_name: string
  product_id: string
  product_sku: string
  product_name: string
  trx_date: string
  doc_type: DocType
  doc_id: string
  doc_no: string
  direction: string
  qty_base: string
  unit_cost?: string
  batch_no?: string
  expiry_date?: string
}

export interface LowStockAlert {
  warehouse_id: string
  warehouse_name: string
  product_id: string
  product_sku: string
  product_name: string
  qty_on_hand: string
  safety_stock: string
  reorder_point: string
  shortage: string
}

export interface Role {
  id: string
  code: string
  name: string
  description?: string
  is_internal: boolean
  is_system: boolean
  is_active: boolean
  permissions: Permission[]
  created_at: string
  updated_at: string
}

export interface Permission {
  id: string
  code: string
  name: string
  module?: string
  description?: string
  created_at: string
}

// Report Types
export interface StockOnHandReport {
  warehouse_id: string
  warehouse_code: string
  warehouse_name: string
  product_id: string
  product_sku: string
  product_name: string
  category_name?: string
  base_uom: string
  qty_on_hand: string
  avg_cost?: string
  total_value?: string
  safety_stock?: string
  reorder_point?: string
}

export interface StockLedgerReport {
  trx_date: string
  warehouse_code: string
  warehouse_name: string
  product_sku: string
  product_name: string
  doc_type: string
  doc_no: string
  direction: string
  qty_base: string
  unit_cost?: string
  batch_no?: string
  expiry_date?: string
}

export interface PurchaseLinesReport {
  doc_date: string
  doc_no: string
  status: string
  partner_code?: string
  partner_name?: string
  warehouse_name?: string
  product_sku: string
  product_name: string
  qty: string
  uom: string
  unit_price?: string
  line_total?: string
  created_by_name: string
  approved_by_name?: string
}

export interface SalesLinesReport {
  doc_date: string
  doc_no: string
  status: string
  partner_code?: string
  partner_name?: string
  warehouse_name?: string
  product_sku: string
  product_name: string
  qty: string
  uom: string
  unit_price?: string
  line_total?: string
  created_by_name: string
  approved_by_name?: string
}

export interface CostSummaryReport {
  warehouse_id: string
  warehouse_code: string
  warehouse_name: string
  product_id: string
  product_sku: string
  product_name: string
  category_name?: string
  qty_on_hand: string
  avg_cost?: string
  total_value?: string
}

export interface AuditLogWithActor {
  id: string
  actor_user_id: string
  actor_name: string
  action: string
  entity_type: string
  entity_id: string
  before_data?: Record<string, unknown>
  after_data?: Record<string, unknown>
  created_at: string
}

// Request types
export interface CreateUserRequest {
  email: string
  password: string
  display_name: string
  role_ids: string[]
}

export interface UpdateUserRequest {
  email?: string
  display_name?: string
  is_active?: boolean
  role_ids?: string[]
}

export interface CreateRoleRequest {
  code: string
  name: string
  permission_ids: string[]
}

export interface UpdateRoleRequest {
  name?: string
  permission_ids?: string[]
}

// Password Change Types
export interface ChangeOwnPasswordRequest {
  current_password: string
  new_password: string
}

export interface ResetPasswordRequest {
  new_password: string
}

// SKU Types
export interface SkuSegment {
  code: string
  label: string
  value: string
  source: string
}

export interface SkuPreviewRequest {
  org?: string
  cat: string
  sub: string
  attributes?: {
    generic_name?: string
    dose_value?: number
    dose_unit?: string
    dosage_form?: string
    sterile?: boolean
    [key: string]: unknown
  }
  pack: {
    uom: string
    qty: number
  }
  source: string
  rule_version_hint?: string
}

export interface SkuPreviewResponse {
  preview_sku: string
  segments: SkuSegment[]
  rule_version: string
  rule_updated_at?: string
}

export interface SkuPreviewError {
  code: 'E1' | 'E2' | 'E3' | 'E4' | 'E5'
  message: string
  suggestion?: string
  field?: string
}

// Extended Product creation with SKU generation
export interface CreateProductWithSkuRequest {
  name?: string
  spec?: string
  base_uom: string
  track_batch?: boolean
  track_expiry?: boolean
  safety_stock?: number | null
  reorder_point?: number | null
  category_code: string
  subcategory_code: string
  source_code: string
  pack_unit: string
  pack_qty: number
  attributes?: {
    generic_name?: string
    dose_value?: number
    dose_unit?: string
    dosage_form?: string
    sterile?: boolean
    [key: string]: unknown
  } | null
}

// ============================================
// AUP Protocol Types
// ============================================

export type ProtocolStatus =
  | 'DRAFT'
  | 'SUBMITTED'
  | 'PRE_REVIEW'
  | 'UNDER_REVIEW'
  | 'REVISION_REQUIRED'
  | 'RESUBMITTED'
  | 'APPROVED'
  | 'APPROVED_WITH_CONDITIONS'
  | 'DEFERRED'
  | 'REJECTED'
  | 'SUSPENDED'
  | 'CLOSED'
  | 'DELETED'

export const protocolStatusNames: Record<ProtocolStatus, string> = {
  DRAFT: '草稿',
  SUBMITTED: '已提交',
  PRE_REVIEW: '行政預審',
  UNDER_REVIEW: '審查中',
  REVISION_REQUIRED: '需修訂',
  RESUBMITTED: '已重送',
  APPROVED: '已核准',
  APPROVED_WITH_CONDITIONS: '附條件核准',
  DEFERRED: '延後審議',
  REJECTED: '已否決',
  SUSPENDED: '已暫停',
  CLOSED: '已結案',
  DELETED: '已刪除',
}

export interface Protocol {
  id: string
  protocol_no: string
  iacuc_no?: string
  title: string
  status: ProtocolStatus
  pi_user_id: string
  working_content?: Record<string, unknown>
  start_date?: string
  end_date?: string
  created_by: string
  created_at: string
  updated_at: string
}

export interface ProtocolListItem {
  id: string
  protocol_no: string
  iacuc_no?: string
  title: string
  status: ProtocolStatus
  pi_user_id: string
  pi_name: string
  pi_organization?: string
  start_date?: string
  end_date?: string
  created_at: string
  apply_study_number?: string
}

export interface ProtocolResponse extends Protocol {
  pi_name?: string
  pi_email?: string
  pi_organization?: string
  status_display: string
}

export interface ProtocolVersion {
  id: string
  protocol_id: string
  version_no: number
  content_snapshot: Record<string, unknown>
  submitted_at: string
  submitted_by: string
}

export interface ProtocolStatusHistory {
  id: string
  protocol_id: string
  from_status?: ProtocolStatus
  to_status: ProtocolStatus
  changed_by: string
  remark?: string
  created_at: string
}

export interface ReviewAssignment {
  id: string
  protocol_id: string
  reviewer_id: string
  assigned_by: string
  assigned_at: string
  completed_at?: string
}

export interface ReviewComment {
  id: string
  protocol_version_id: string
  reviewer_id: string
  content: string
  is_resolved: boolean
  resolved_by?: string
  resolved_at?: string
  created_at: string
  updated_at: string
}

export interface ReviewCommentResponse extends ReviewComment {
  reviewer_name: string
  reviewer_email: string
}

export interface CreateProtocolRequest {
  title: string
  pi_user_id?: string
  working_content?: Record<string, unknown>
  start_date?: string
  end_date?: string
}

export interface UpdateProtocolRequest {
  title?: string
  working_content?: Record<string, unknown>
  start_date?: string
  end_date?: string
}

export interface ChangeStatusRequest {
  to_status: ProtocolStatus
  remark?: string
}

export interface CreateCommentRequest {
  protocol_version_id: string
  content: string
}

export interface AssignReviewerRequest {
  protocol_id: string
  reviewer_id: string
}

export interface AssignCoEditorRequest {
  protocol_id: string
  user_id: string
}

export interface CoEditorAssignmentResponse {
  user_id: string
  protocol_id: string
  role_in_protocol: string
  granted_at: string
  granted_by?: string
  user_name: string
  user_email: string
  granted_by_name?: string
}

export interface ReviewAssignmentResponse extends ReviewAssignment {
  reviewer_name: string
  reviewer_email: string
  assigned_by_name: string
}

// ============================================
// 附件管理 Types
// ============================================

export interface ProtocolAttachment {
  id: string
  protocol_id?: string
  protocol_version_id?: string
  file_name: string
  file_path: string
  file_size: number
  mime_type: string
  uploaded_by: string
  uploaded_by_name?: string
  created_at: string
}

export interface UserSimple {
  id: string
  email: string
  display_name?: string
}

// ============================================
// 實驗動物管理 Types
// ============================================

export type PigStatus = 'unassigned' | 'assigned' | 'in_experiment' | 'completed'
export type PigBreed = 'minipig' | 'white' | 'other'
export type PigGender = 'male' | 'female'
export type RecordType = 'abnormal' | 'experiment' | 'observation'

// Status names for dropdown selection (excludes 'assigned' which is deprecated)
export const pigStatusNames: Partial<Record<PigStatus, string>> = {
  unassigned: '未分配',
  in_experiment: '實驗中',
  completed: '實驗完畢',
}

// All status names for display purposes (includes deprecated 'assigned')
export const allPigStatusNames: Record<PigStatus, string> = {
  unassigned: '未分配',
  assigned: '已分配',
  in_experiment: '實驗中',
  completed: '實驗完畢',
}

export const pigBreedNames: Record<PigBreed, string> = {
  minipig: '迷你豬',
  white: '白豬',
  other: '其他',
}

export const pigGenderNames: Record<PigGender, string> = {
  male: '公',
  female: '母',
}

export const recordTypeNames: Record<RecordType, string> = {
  abnormal: '異常紀錄',
  experiment: '試驗紀錄',
  observation: '觀察紀錄',
}

export interface PigSource {
  id: string
  code: string
  name: string
  address?: string
  contact?: string
  phone?: string
  is_active: boolean
  sort_order: number
}

export interface Pig {
  id: number
  ear_tag: string
  status: PigStatus
  breed: PigBreed
  breed_other?: string
  source_id?: string
  source_name?: string
  gender: PigGender
  birth_date?: string
  entry_date: string
  entry_weight?: number
  pen_location?: string
  pre_experiment_code?: string
  iacuc_no?: string
  experiment_date?: string
  remark?: string
  vet_last_viewed_at?: string
  created_by?: string
  created_at: string
  updated_at: string
}

export interface PigListItem extends Pig {
  latest_weight?: number
  latest_weight_date?: string
  breed_other?: string
  has_abnormal_record?: boolean
  vet_recommendation_date?: string
  is_on_medication?: boolean
  last_medication_date?: string
}

export interface PigObservation {
  id: number
  pig_id: number
  event_date: string
  record_type: RecordType
  equipment_used?: string[]
  anesthesia_start?: string
  anesthesia_end?: string
  content: string
  no_medication_needed: boolean
  treatments?: {
    drug: string
    dosage: string
    end_date?: string
  }[]
  remark?: string
  vet_read: boolean
  vet_read_at?: string
  created_by?: string
  created_by_name?: string
  created_at: string
  updated_at: string
}

export interface PigSurgery {
  id: number
  pig_id: number
  is_first_experiment: boolean
  surgery_date: string
  surgery_site: string
  induction_anesthesia?: Record<string, unknown>
  pre_surgery_medication?: Record<string, unknown>
  positioning?: string
  anesthesia_maintenance?: Record<string, unknown>
  anesthesia_observation?: string
  vital_signs?: {
    time: string
    heart_rate: number
    respiration_rate: number
    temperature: number
    spo2: number
  }[]
  reflex_recovery?: string
  respiration_rate?: number
  post_surgery_medication?: Record<string, unknown>
  remark?: string
  no_medication_needed: boolean
  vet_read: boolean
  vet_read_at?: string
  created_by?: string
  created_by_name?: string
  created_at: string
  updated_at: string
}

export interface PigWeight {
  id: number
  pig_id: number
  measure_date: string
  weight: number
  created_by?: string
  created_by_name?: string
  created_at: string
}

export interface PigVaccination {
  id: number
  pig_id: number
  administered_date: string
  vaccine?: string
  deworming_dose?: string
  created_by?: string
  created_by_name?: string
  created_at: string
}

export interface PigSacrifice {
  id: number
  pig_id: number
  sacrifice_date?: string
  zoletil_dose?: string
  method_electrocution: boolean
  method_bloodletting: boolean
  method_other?: string
  sampling?: string
  sampling_other?: string
  blood_volume_ml?: number
  confirmed_sacrifice: boolean
  created_by?: string
  created_by_name?: string
  created_at: string
  updated_at: string
}

export interface PigPathologyReport {
  id: number
  pig_id: number
  attachments?: {
    id: string
    file_name: string
    file_path: string
    file_size: number
    created_at: string
  }[]
  created_by?: string
  created_by_name?: string
  created_at: string
  updated_at: string
}

export interface VetRecommendation {
  id: number
  record_type: 'observation' | 'surgery'
  record_id: number
  content: string
  attachments?: Record<string, unknown>
  created_by?: string
  created_by_name?: string
  created_at: string
}

// Pig API Request Types
export interface CreatePigRequest {
  ear_tag: string
  breed: PigBreed
  gender: PigGender
  source_id?: string
  birth_date?: string
  entry_date: string
  entry_weight?: number
  pen_location?: string
  pre_experiment_code?: string
  remark?: string
}

export interface UpdatePigRequest {
  ear_tag?: string
  status?: PigStatus
  breed?: PigBreed
  gender?: PigGender
  source_id?: string
  birth_date?: string
  entry_date?: string
  entry_weight?: number
  pen_location?: string
  pre_experiment_code?: string
  iacuc_no?: string
  experiment_date?: string
  remark?: string
}

export interface BatchAssignPigsRequest {
  pig_ids: number[]
  iacuc_no: string
}

export interface BatchStartExperimentRequest {
  pig_ids: number[]
}

// Password Reset Types
export interface ForgotPasswordRequest {
  email: string
}

export interface ResetPasswordWithTokenRequest {
  token: string
  new_password: string
}

// ============================================
// Notification Types
// ============================================

export type NotificationType =
  | 'low_stock'
  | 'expiry_warning'
  | 'document_approval'
  | 'protocol_status'
  | 'vet_recommendation'
  | 'system_alert'
  | 'monthly_report'

export const notificationTypeNames: Record<NotificationType, string> = {
  low_stock: '低庫存預警',
  expiry_warning: '效期預警',
  document_approval: '單據審核',
  protocol_status: '計畫狀態',
  vet_recommendation: '獸醫師建議',
  system_alert: '系統通知',
  monthly_report: '月報',
}

export interface NotificationItem {
  id: string
  type: NotificationType
  title: string
  content?: string
  is_read: boolean
  read_at?: string
  related_entity_type?: string
  related_entity_id?: string
  created_at: string
}

export interface NotificationListResponse {
  data: NotificationItem[]
  total: number
  page: number
  per_page: number
}

export interface UnreadNotificationCount {
  count: number
}

export interface MarkNotificationsReadRequest {
  notification_ids: string[]
}

// ============================================
// Notification Settings Types
// ============================================

export interface NotificationSettings {
  user_id: string
  email_low_stock: boolean
  email_expiry_warning: boolean
  email_document_approval: boolean
  email_protocol_status: boolean
  email_monthly_report: boolean
  expiry_warning_days: number
  low_stock_notify_immediately: boolean
  updated_at: string
}

export interface UpdateNotificationSettingsRequest {
  email_low_stock?: boolean
  email_expiry_warning?: boolean
  email_document_approval?: boolean
  email_protocol_status?: boolean
  email_monthly_report?: boolean
  expiry_warning_days?: number
  low_stock_notify_immediately?: boolean
}

// ============================================
// File Upload Types
// ============================================

export interface UploadResponse {
  id: string
  file_name: string
  file_path: string
  file_size: number
  mime_type: string
}

export interface Attachment {
  id: string
  entity_type: string
  entity_id: string
  file_name: string
  file_path: string
  file_size: number
  mime_type: string
  uploaded_by: string
  created_at: string
}
