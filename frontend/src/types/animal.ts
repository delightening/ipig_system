/**
 * 實驗動物管理型別
 */

// 基本列舉
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

// 全部狀態名稱（保留向後相容性）
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

// 豬源
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

// 豬隻
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

// 觀察記錄
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

// 手術記錄
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

// 體重記錄
export interface PigWeight {
    id: number
    pig_id: string
    measure_date: string
    weight: number
    created_by?: string
    created_by_name?: string
    created_at: string
}

// 疫苗記錄
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

// 犧牲記錄
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

// 病理報告
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

// 豬隻請求型別
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
