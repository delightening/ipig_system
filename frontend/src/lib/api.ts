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
export interface UserTraining {
  code: string
  certificate_no?: string
  received_date?: string
}

export interface User {
  id: string
  email: string
  display_name: string
  phone?: string
  organization?: string
  is_active: boolean
  roles: string[]
  permissions: string[]
  must_change_password?: boolean
  // AUP 第 8 節人員資料
  entry_date?: string | null
  position?: string | null
  aup_roles?: string[]
  years_experience?: number
  trainings?: UserTraining[]
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

// Storage Location Types (儲位/貨架)
export type StorageLocationType = 'shelf' | 'rack' | 'zone' | 'bin'

export interface StorageLocation {
  id: string
  warehouse_id: string
  code: string
  name?: string
  location_type: StorageLocationType
  row_index: number
  col_index: number
  width: number
  height: number
  capacity?: number
  current_count: number
  color?: string
  is_active: boolean
  config?: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface StorageLocationWithWarehouse extends StorageLocation {
  warehouse_code: string
  warehouse_name: string
}

export interface StorageLayoutItem {
  id: string
  row_index: number
  col_index: number
  width: number
  height: number
}

export interface UpdateStorageLayoutRequest {
  items: StorageLayoutItem[]
}

export const storageLocationTypeNames: Record<StorageLocationType, string> = {
  shelf: '貨架',
  rack: '儲物架',
  zone: '區域',
  bin: '儲物格',
}

export interface StorageLocationInventoryItem {
  id: string
  storage_location_id: string
  product_id: string
  product_sku: string
  product_name: string
  on_hand_qty: string
  base_uom: string
  batch_no?: string
  expiry_date?: string
  updated_at: string
}

export interface UpdateStorageLocationInventoryItemRequest {
  on_hand_qty: string
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
  phone?: string
  organization?: string
  is_active?: boolean
  role_ids?: string[]
  // AUP 第 8 節人員資料
  entry_date?: string | null
  position?: string | null
  aup_roles?: string[]
  years_experience?: number
  trainings?: UserTraining[]
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
  | 'PRE_REVIEW_REVISION_REQUIRED'
  | 'VET_REVIEW'
  | 'VET_REVISION_REQUIRED'
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
  PRE_REVIEW_REVISION_REQUIRED: '行政預審補件',
  VET_REVIEW: '獸醫審查',
  VET_REVISION_REQUIRED: '獸醫要求修訂',
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

import { ProtocolWorkingContent } from '@/types/protocol'

export interface Protocol {
  id: string
  protocol_no: string
  iacuc_no?: string
  title: string
  status: ProtocolStatus
  pi_user_id: string
  working_content?: ProtocolWorkingContent
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
  content_snapshot: ProtocolWorkingContent
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
  /** 是否為正式審查委員（可撰寫意見） */
  is_primary_reviewer?: boolean
  /** 審查階段 */
  review_stage?: 'PRE_REVIEW' | 'VET_REVIEW' | 'UNDER_REVIEW'
}

export interface ReviewComment {
  id: string
  protocol_version_id?: string
  protocol_id?: string
  reviewer_id: string
  content: string
  is_resolved: boolean
  resolved_by?: string
  resolved_at?: string
  /** 審查階段 */
  review_stage?: 'PRE_REVIEW' | 'VET_REVIEW' | 'UNDER_REVIEW'
  created_at: string
  updated_at: string
}

export interface ReviewCommentResponse extends ReviewComment {
  reviewer_name: string
  reviewer_email: string
  parent_comment_id?: string
  replied_by?: string
  replied_by_name?: string
  replied_by_email?: string
}

export interface CreateProtocolRequest {
  title: string
  pi_user_id?: string
  working_content?: ProtocolWorkingContent
  start_date?: string
  end_date?: string
}

export interface UpdateProtocolRequest {
  title?: string
  working_content?: ProtocolWorkingContent
  start_date?: string
  end_date?: string
}

export interface ChangeStatusRequest {
  to_status: ProtocolStatus
  remark?: string
  /** 審查委員 ID 列表（當目標狀態為 UNDER_REVIEW 時必填 2-3 位） */
  reviewer_ids?: string[]
  /** 獸醫師 ID（當目標狀態為 VET_REVIEW 時可選，未設定則使用預設獸醫） */
  vet_id?: string
}

export interface CreateCommentRequest {
  protocol_version_id: string
  content: string
}

export interface ReplyCommentRequest {
  parent_comment_id: string
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
// Activity Log Types
// ============================================

export interface UserActivityLog {
  id: string
  actor_user_id: string
  actor_email: string
  actor_display_name: string
  actor_roles: string[]
  event_category: string
  event_type: string
  event_severity: string
  entity_type?: string
  entity_id?: string
  entity_display_name?: string
  before_data?: any
  after_data?: any
  changed_fields?: string[]
  ip_address?: string
  user_agent?: string
  request_path?: string
  request_method?: string
  response_status?: number
  created_at: string
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

export type PigStatus = 'unassigned' | 'in_experiment' | 'completed'
export type PigBreed = 'minipig' | 'white' | 'lyd' | 'other'
export type PigGender = 'male' | 'female'
export type RecordType = 'abnormal' | 'experiment' | 'observation'

// 狀態名稱映射
export const pigStatusNames: Record<PigStatus, string> = {
  unassigned: '未分配',
  in_experiment: '實驗中',
  completed: '實驗完成',
}

// 全部狀態名稱（與 pigStatusNames 相同，保留向後相容性）
export const allPigStatusNames: Record<PigStatus, string> = {
  unassigned: '未分配',
  in_experiment: '實驗中',
  completed: '實驗完成',
}

export const pigBreedNames: Record<PigBreed, string> = {
  minipig: '迷你豬',
  white: '白豬',
  lyd: 'LYD',
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
  id: string
  animal_no?: string
  animal_id?: string
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
  experiment_assigned_by?: string
  experiment_assigned_by_name?: string
  remark?: string
  deletion_reason?: string
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
  pig_id: string
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
  pig_id: string
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
  pig_id: string
  measure_date: string
  weight: number
  created_by?: string
  created_by_name?: string
  created_at: string
}

export interface PigVaccination {
  id: number
  pig_id: string
  administered_date: string
  vaccine?: string
  deworming_dose?: string
  created_by?: string
  created_by_name?: string
  created_at: string
}

export interface PigSacrifice {
  id: number
  pig_id: string
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
  pig_id: string
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
  is_urgent: boolean
  attachments?: Record<string, unknown>
  created_by?: string
  created_by_name?: string
  created_at: string
}



export interface BloodTestTemplate {
  id: string
  code: string
  name: string
  default_unit?: string
  reference_range?: string
  default_price?: number
  sort_order: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface BloodTestListItem {
  id: string
  pig_id: string
  test_date: string
  lab_name?: string
  remark?: string
  vet_read: boolean
  created_at: string
  created_by_name?: string
  item_count: number
  abnormal_count: number
}

export interface PigBloodTestItem {
  id: string
  blood_test_id: string
  template_id?: string
  item_name: string
  result_value?: string
  result_unit?: string
  reference_range?: string
  is_abnormal: boolean
  remark?: string
  sort_order: number
  created_at: string
}

export interface PigBloodTestWithItems {
  blood_test: {
    id: string
    pig_id: string
    test_date: string
    lab_name?: string
    remark?: string
    vet_read: boolean
    is_deleted: boolean
    created_by?: string
    created_at: string
    updated_at: string
  }
  items: PigBloodTestItem[]
  created_by_name?: string
}

export interface BloodTestItemInput {
  template_id?: string
  item_name: string
  result_value?: string
  result_unit?: string
  reference_range?: string
  is_abnormal: boolean
  remark?: string
  sort_order: number
}

export interface CreateBloodTestRequest {
  test_date: string
  lab_name?: string
  remark?: string
  items: BloodTestItemInput[]
}

export interface UpdateBloodTestRequest {
  test_date?: string
  lab_name?: string
  remark?: string
  items?: BloodTestItemInput[]
}

export interface CreateBloodTestTemplateRequest {
  code: string
  name: string
  default_unit?: string
  reference_range?: string
  default_price?: number
  sort_order: number
}

export interface UpdateBloodTestTemplateRequest {
  name?: string
  default_unit?: string
  reference_range?: string
  default_price?: number
  sort_order?: number
  is_active?: boolean
}

// 血液檢查 API 函數
export const bloodTestApi = {
  listByPig: (pigId: string) =>
    api.get<BloodTestListItem[]>(`/pigs/${pigId}/blood-tests`),
  getById: (id: string) =>
    api.get<PigBloodTestWithItems>(`/blood-tests/${id}`),
  create: (pigId: string, data: CreateBloodTestRequest) =>
    api.post<PigBloodTestWithItems>(`/pigs/${pigId}/blood-tests`, data),
  update: (id: string, data: UpdateBloodTestRequest) =>
    api.put<PigBloodTestWithItems>(`/blood-tests/${id}`, data),
  delete: (id: string, reason: string) =>
    api.delete(`/blood-tests/${id}`, { data: { reason } }),
}

// 血液檢查項目模板 API 函數
export const bloodTestTemplateApi = {
  list: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates'),
  listAll: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates/all'),
  create: (data: CreateBloodTestTemplateRequest) =>
    api.post<BloodTestTemplate>('/blood-test-templates', data),
  update: (id: string, data: UpdateBloodTestTemplateRequest) =>
    api.put<BloodTestTemplate>(`/blood-test-templates/${id}`, data),
  delete: (id: string) =>
    api.delete(`/blood-test-templates/${id}`),
}

// ============================================
// 血液檢查組合 (Panel)
// ============================================

export interface BloodTestPanel {
  id: string
  key: string
  name: string
  icon?: string
  sort_order: number
  is_active: boolean
  items: BloodTestTemplate[]
  created_at: string
  updated_at: string
}

export interface CreateBloodTestPanelRequest {
  key: string
  name: string
  icon?: string
  sort_order?: number
  template_ids?: string[]
}

export interface UpdateBloodTestPanelRequest {
  name?: string
  icon?: string
  sort_order?: number
  is_active?: boolean
}

export interface UpdateBloodTestPanelItemsRequest {
  template_ids: string[]
}

// 血液檢查組合 API 函數
export const bloodTestPanelApi = {
  list: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels'),
  listAll: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels/all'),
  create: (data: CreateBloodTestPanelRequest) =>
    api.post<BloodTestPanel>('/blood-test-panels', data),
  update: (id: string, data: UpdateBloodTestPanelRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}`, data),
  updateItems: (id: string, data: UpdateBloodTestPanelItemsRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}/items`, data),
  delete: (id: string) =>
    api.delete(`/blood-test-panels/${id}`),
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

// ============================================
// Amendment System Types
// ============================================

export type AmendmentType = 'MAJOR' | 'MINOR' | 'PENDING'
export type AmendmentStatus =
  | 'DRAFT'
  | 'SUBMITTED'
  | 'CLASSIFIED'
  | 'UNDER_REVIEW'
  | 'REVISION_REQUIRED'
  | 'RESUBMITTED'
  | 'APPROVED'
  | 'REJECTED'
  | 'ADMIN_APPROVED'

export const amendmentStatusNames: Record<AmendmentStatus, string> = {
  DRAFT: '草稿',
  SUBMITTED: '已提交',
  CLASSIFIED: '已分類',
  UNDER_REVIEW: '審查中',
  REVISION_REQUIRED: '需修訂',
  RESUBMITTED: '已重送',
  APPROVED: '已核准',
  REJECTED: '已否決',
  ADMIN_APPROVED: '行政核准',
}

// Status colors: DRAFT-gray, SUBMITTED-blue, CLASSIFIED-orange, UNDER_REVIEW-purple, APPROVED-green, REJECTED-red
export const amendmentStatusColors: Record<AmendmentStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  CLASSIFIED: 'warning',
  UNDER_REVIEW: 'outline',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  REJECTED: 'destructive',
  ADMIN_APPROVED: 'success',
}

export const amendmentTypeNames: Record<AmendmentType, string> = {
  MAJOR: '重大變更',
  MINOR: '小變更',
  PENDING: '待分類',
}

// 變更項目選項（多選）
export const AMENDMENT_CHANGE_ITEM_OPTIONS = [
  { value: 'ANIMAL_COUNT', label: '動物數量' },
  { value: 'PROCEDURE', label: '實驗程序' },
  { value: 'PERSONNEL', label: '試驗工作人員' },
  { value: 'DURATION', label: '執行期間' },
  { value: 'FUNDING', label: '經費來源' },
  { value: 'FACILITY', label: '設施/場地' },
  { value: 'SPECIES', label: '動物種類/品系' },
  { value: 'ANESTHESIA', label: '麻醉方式' },
  { value: 'EUTHANASIA', label: '安樂死方法' },
  { value: 'OTHER', label: '其他' },
] as const

export interface Amendment {
  id: string
  protocol_id: string
  amendment_no: string
  revision_number: number
  amendment_type: AmendmentType
  status: AmendmentStatus
  title: string
  description?: string
  change_items?: string[]
  changes_content?: Record<string, unknown>
  submitted_by?: string
  submitted_at?: string
  classified_by?: string
  classified_at?: string
  classification_remark?: string
  created_by: string
  created_at: string
  updated_at: string
}

export interface AmendmentListItem extends Amendment {
  protocol_iacuc_no?: string
  protocol_title?: string
  submitted_by_name?: string
  classified_by_name?: string
}

export interface CreateAmendmentRequest {
  protocol_id: string
  title: string
  description?: string
  change_items?: string[]
  changes_content?: Record<string, unknown>
}

export interface UpdateAmendmentRequest {
  title?: string
  description?: string
  change_items?: string[]
  changes_content?: Record<string, unknown>
}

export interface ClassifyAmendmentRequest {
  amendment_type: AmendmentType
  remark?: string
}

export interface ChangeAmendmentStatusRequest {
  to_status: AmendmentStatus
  remark?: string
}

export interface RecordAmendmentDecisionRequest {
  decision: 'APPROVE' | 'REJECT' | 'REVISION'
  comment?: string
}

export interface AmendmentVersion {
  id: string
  amendment_id: string
  version_no: number
  content_snapshot: Record<string, unknown>
  submitted_at: string
  submitted_by: string
}

export interface AmendmentStatusHistory {
  id: string
  amendment_id: string
  from_status?: AmendmentStatus
  to_status: AmendmentStatus
  changed_by: string
  remark?: string
  created_at: string
}

export interface AmendmentReviewAssignment {
  id: string
  amendment_id: string
  reviewer_id: string
  assigned_by: string
  assigned_at: string
  decision?: string
  decided_at?: string
  comment?: string
  reviewer_name?: string
  reviewer_email?: string
}

// ============================================
// Protocol Activity Types (新增)
// ============================================

export type ProtocolActivityType =
  // 生命週期
  | 'CREATED'
  | 'UPDATED'
  | 'SUBMITTED'
  | 'RESUBMITTED'
  | 'APPROVED'
  | 'APPROVED_WITH_CONDITIONS'
  | 'CLOSED'
  | 'REJECTED'
  | 'SUSPENDED'
  | 'DELETED'
  // 審查流程
  | 'STATUS_CHANGED'
  | 'REVIEWER_ASSIGNED'
  | 'VET_ASSIGNED'
  | 'COEDITOR_ASSIGNED'
  | 'COEDITOR_REMOVED'
  // 審查意見
  | 'COMMENT_ADDED'
  | 'COMMENT_REPLIED'
  | 'COMMENT_RESOLVED'
  // 附件
  | 'ATTACHMENT_UPLOADED'
  | 'ATTACHMENT_DELETED'
  // 版本
  | 'VERSION_CREATED'
  | 'VERSION_RECOVERED'
  // 修正案
  | 'AMENDMENT_CREATED'
  | 'AMENDMENT_SUBMITTED'
  // 動物管理
  | 'PIG_ASSIGNED'
  | 'PIG_UNASSIGNED'

export interface ProtocolActivity {
  id: string
  protocol_id: string
  activity_type: ProtocolActivityType
  activity_type_display: string
  actor_id: string
  actor_name: string
  actor_email: string
  from_value?: string
  to_value?: string
  target_entity_type?: string
  target_entity_id?: string
  target_entity_name?: string
  remark?: string
  extra_data?: Record<string, unknown>
  created_at: string
}

export const getProtocolActivities = async (id: string): Promise<ProtocolActivity[]> => {
  const response = await api.get<ProtocolActivity[]>(`/protocols/${id}/activities`)
  return response.data
}
