// 治療方式藥物選項型別定義

/** 藥物選項 */
export interface TreatmentDrugOption {
    id: string
    name: string
    display_name: string | null
    default_dosage_unit: string | null
    available_units: string[] | null
    default_dosage_value: string | null
    erp_product_id: string | null
    category: string | null
    sort_order: number
    is_active: boolean
    created_by: string | null
    created_at: string
    updated_at: string
}

/** 建立藥物選項請求 */
export interface CreateTreatmentDrugRequest {
    name: string
    display_name?: string
    default_dosage_unit?: string
    available_units?: string[]
    default_dosage_value?: string
    erp_product_id?: string
    category?: string
    sort_order?: number
}

/** 更新藥物選項請求 */
export interface UpdateTreatmentDrugRequest {
    name?: string
    display_name?: string
    default_dosage_unit?: string
    available_units?: string[]
    default_dosage_value?: string
    erp_product_id?: string
    category?: string
    sort_order?: number
    is_active?: boolean
}

/** ERP 匯入請求 */
export interface ImportFromErpRequest {
    product_ids: string[]
    category?: string
}

/** 藥物分類 */
export const DRUG_CATEGORIES = ['麻醉', '止痛', '抗生素', '鎮靜', '其他'] as const

/** 劑量單位 */
export const DOSAGE_UNITS = ['mg', 'ml', 'mg/kg', 'cap', 'tab', 'cc', 'L/min', '%', 'cm', 'g', 'pcs'] as const
