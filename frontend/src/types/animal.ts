/**
 * 實驗動物管理型別
 */

// 基本列舉
export type AnimalStatus = 'unassigned' | 'in_experiment' | 'completed' | 'euthanized' | 'sudden_death' | 'transferred'
export type AnimalBreed = 'minipig' | 'white' | 'lyd' | 'other'
export type AnimalGender = 'male' | 'female'
export type RecordType = 'abnormal' | 'experiment' | 'observation'

// 狀態名稱映射
export const animalStatusNames: Record<AnimalStatus, string> = {
    unassigned: '未分配',
    in_experiment: '實驗中',
    completed: '實驗完成',
    euthanized: '已安樂死',
    sudden_death: '猝死',
    transferred: '已轉讓',
}

// 全部狀態名稱（保留向後相容性）
export const allAnimalStatusNames: Record<AnimalStatus, string> = {
    unassigned: '未分配',
    in_experiment: '實驗中',
    completed: '實驗完成',
    euthanized: '已安樂死',
    sudden_death: '猝死',
    transferred: '已轉讓',
}

export const animalBreedNames: Record<AnimalBreed, string> = {
    minipig: '迷你豬',
    white: '白豬',
    lyd: 'LYD',
    other: '其他',
}

export const animalGenderNames: Record<AnimalGender, string> = {
    male: '公',
    female: '母',
}

export const recordTypeNames: Record<RecordType, string> = {
    abnormal: '異常紀錄',
    experiment: '試驗紀錄',
    observation: '觀察紀錄',
}

// 動物來源
export interface AnimalSource {
    id: string
    code: string
    name: string
    address?: string
    contact?: string
    phone?: string
    is_active: boolean
    sort_order: number
}

// 動物
export interface Animal {
    id: string
    animal_no?: string
    animal_id?: string
    ear_tag: string
    status: AnimalStatus
    breed: AnimalBreed
    breed_other?: string
    source_id?: string
    source_name?: string
    gender: AnimalGender
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

export interface AnimalListItem extends Animal {
    latest_weight?: number
    latest_weight_date?: string
    breed_other?: string
    has_abnormal_record?: boolean
    vet_recommendation_date?: string
    is_on_medication?: boolean
    last_medication_date?: string
}

// 觀察記錄
export interface AnimalObservation {
    id: number
    animal_id: string
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

// 手術記錄
export interface AnimalSurgery {
    id: number
    animal_id: string
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

// 體重記錄
export interface AnimalWeight {
    id: number
    animal_id: string
    measure_date: string
    weight: number
    created_by?: string
    created_by_name?: string
    created_at: string
}

// 疫苗記錄
export interface AnimalVaccination {
    id: string
    animal_id: string
    administered_date: string
    vaccine?: string
    deworming_dose?: string
    created_by?: string
    created_by_name?: string
    created_at: string
}

// 犧牲記錄
export interface AnimalSacrifice {
    id: number
    animal_id: string
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

// 病理報告
export interface AnimalPathologyReport {
    id: number
    animal_id: string
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

// 猝死記錄
export interface AnimalSuddenDeath {
    id: string
    animal_id: string
    discovered_at: string
    discovered_by: string
    probable_cause?: string
    iacuc_no?: string
    location?: string
    remark?: string
    requires_pathology: boolean
    created_at: string
}

// 獸醫建議
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

// 血液檢查
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
    animal_id: string
    test_date: string
    lab_name?: string
    remark?: string
    vet_read: boolean
    created_at: string
    created_by_name?: string
    item_count: number
    abnormal_count: number
}

export interface AnimalBloodTestItem {
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

export interface AnimalBloodTestWithItems {
    blood_test: {
        id: string
        animal_id: string
        test_date: string
        lab_name?: string
        remark?: string
        vet_read: boolean
        is_deleted: boolean
        created_by?: string
        created_at: string
        updated_at: string
    }
    items: AnimalBloodTestItem[]
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
    panel_id?: string
}

export interface UpdateBloodTestTemplateRequest {
    name?: string
    default_unit?: string
    reference_range?: string
    default_price?: number
    sort_order?: number
    is_active?: boolean
    panel_id?: string
}

// 血液檢查組合
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

// 動物請求型別
export interface CreateAnimalRequest {
    ear_tag: string
    breed: AnimalBreed
    gender: AnimalGender
    source_id?: string
    birth_date?: string
    entry_date: string
    entry_weight?: number
    pen_location?: string
    pre_experiment_code?: string
    remark?: string
}

export interface UpdateAnimalRequest {
    ear_tag?: string
    status?: AnimalStatus
    breed?: AnimalBreed
    gender?: AnimalGender
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

/** P2-R4-13: Timeline 可編輯的紀錄類型 */
export type AnimalTimelineRecord = AnimalObservation | AnimalSurgery

/** UpdateAnimalRequest 欄位值（依 key 而異） */
export type UpdateAnimalRequestValue = UpdateAnimalRequest[keyof UpdateAnimalRequest]

/** 可申請修正的欄位 */
export const CORRECTABLE_FIELDS = ['ear_tag', 'birth_date', 'gender', 'breed'] as const
export type CorrectableField = (typeof CORRECTABLE_FIELDS)[number]

/** 動物欄位修正申請 */
export interface AnimalFieldCorrectionRequest {
  id: string
  animal_id: string
  field_name: string
  old_value: string | null
  new_value: string
  reason: string
  status: 'pending' | 'approved' | 'rejected'
  requested_by: string
  requested_by_name: string | null
  reviewed_by: string | null
  reviewed_at: string | null
  created_at: string
  animal_ear_tag: string | null
}

/** 建立動物欄位修正申請 */
export interface CreateAnimalFieldCorrectionRequest {
  field_name: string
  new_value: string
  reason: string
}

/** 審核動物欄位修正申請 */
export interface ReviewAnimalFieldCorrectionRequest {
  approved: boolean
  reject_reason?: string
}

export interface BatchAssignAnimalsRequest {
    animal_ids: number[]
    iacuc_no: string
}

export interface BatchStartExperimentRequest {
    animal_ids: number[]
}

// 血液檢查分析原始列
export interface BloodTestAnalysisRow {
    animal_id: string
    ear_tag: string
    iacuc_no?: string
    test_date: string
    lab_name?: string
    item_name: string
    template_code?: string
    result_value?: string
    result_unit?: string
    reference_range?: string
    is_abnormal: boolean
}

// ============================================
// 轉讓流程
// ============================================

export type AnimalTransferStatus = 'pending' | 'vet_evaluated' | 'plan_assigned' | 'pi_approved' | 'completed' | 'rejected'

export const transferStatusNames: Record<AnimalTransferStatus, string> = {
    pending: '待審',
    vet_evaluated: '獸醫已評估',
    plan_assigned: '已指定新計劃',
    pi_approved: 'PI 已同意',
    completed: '轉讓完成',
    rejected: '已拒絕',
}

export interface AnimalTransfer {
    id: string
    animal_id: string
    from_iacuc_no: string
    to_iacuc_no?: string
    status: AnimalTransferStatus
    initiated_by: string
    reason: string
    remark?: string
    rejected_by?: string
    rejected_reason?: string
    completed_at?: string
    created_at: string
    updated_at: string
}

export interface TransferVetEvaluation {
    id: string
    transfer_id: string
    vet_id: string
    health_status: string
    is_fit_for_transfer: boolean
    conditions?: string
    evaluated_at: string
}

// 轉讓 DTO
export interface CreateTransferRequest {
    reason: string
    remark?: string
}

export interface VetEvaluateTransferRequest {
    health_status: string
    is_fit_for_transfer: boolean
    conditions?: string
}

export interface AssignTransferPlanRequest {
    to_iacuc_no: string
}

export interface RejectTransferRequest {
    reason: string
}

// IACUC 變更事件（時間軸用）
export interface AnimalEvent {
    id: string
    event_type: string
    actor_name?: string
    before_data?: { iacuc_no?: string }
    after_data?: { iacuc_no?: string }
    created_at: string
}
