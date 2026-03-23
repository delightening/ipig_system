// 設施管理型別定義
// 對應 backend/src/models/facility.rs

// ============================================
// Species（物種）
// ============================================

export interface Species {
  id: string
  code: string
  name: string
  name_en: string | null
  icon: string | null
  is_active: boolean
  config: Record<string, unknown> | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface CreateSpeciesRequest {
  code: string
  name: string
  name_en?: string
  icon?: string
  parent_id?: string
  sort_order?: number
}

export interface UpdateSpeciesRequest {
  name?: string
  name_en?: string
  icon?: string
  is_active?: boolean
  parent_id?: string
  sort_order?: number
}

// ============================================
// Facility（設施）
// ============================================

export interface Facility {
  id: string
  code: string
  name: string
  address: string | null
  phone: string | null
  contact_person: string | null
  is_active: boolean
  config: Record<string, unknown> | null
  created_at: string
  updated_at: string
}

export interface CreateFacilityRequest {
  code: string
  name: string
  address?: string
  phone?: string
  contact_person?: string
}

export interface UpdateFacilityRequest {
  name?: string
  address?: string
  phone?: string
  contact_person?: string
  is_active?: boolean
}

// ============================================
// Building（棟舍）
// ============================================

export interface BuildingWithFacility {
  id: string
  facility_id: string
  facility_code: string
  facility_name: string
  code: string
  name: string
  description: string | null
  is_active: boolean
  config: Record<string, unknown> | null
  sort_order: number
}

export interface CreateBuildingRequest {
  facility_id: string
  code: string
  name: string
  description?: string
  sort_order?: number
}

export interface UpdateBuildingRequest {
  name?: string
  description?: string
  is_active?: boolean
  sort_order?: number
}

// ============================================
// Zone（區域）
// ============================================

export interface ZoneWithBuilding {
  id: string
  building_id: string
  building_code: string
  building_name: string
  facility_id: string
  facility_name: string
  code: string
  name: string | null
  color: string | null
  is_active: boolean
  layout_config: Record<string, unknown> | null
  sort_order: number
}

export interface CreateZoneRequest {
  building_id: string
  code: string
  name?: string
  color?: string
  sort_order?: number
}

export interface UpdateZoneRequest {
  name?: string
  color?: string
  is_active?: boolean
  sort_order?: number
}

// ============================================
// Pen（欄位）
// ============================================

export interface PenDetails {
  id: string
  code: string
  name: string | null
  capacity: number
  current_count: number
  status: string
  row_index: number | null
  col_index: number | null
  zone_id: string
  zone_code: string
  zone_name: string | null
  zone_color: string | null
  zone_layout_config: Record<string, unknown> | null
  building_id: string
  building_code: string
  building_name: string
  facility_id: string
  facility_code: string
  facility_name: string
}

export interface CreatePenRequest {
  zone_id: string
  code: string
  name?: string
  capacity?: number
  row_index?: number
  col_index?: number
}

export interface UpdatePenRequest {
  name?: string
  capacity?: number
  status?: string
  row_index?: number
  col_index?: number
  is_active?: boolean
}

export const PEN_STATUS_NAMES: Record<string, string> = {
  active: '使用中',
  empty: '空欄',
  maintenance: '維修中',
  disabled: '停用',
}

// ============================================
// Department（部門）
// ============================================

export interface DepartmentWithManager {
  id: string
  code: string
  name: string
  parent_id: string | null
  parent_name: string | null
  manager_id: string | null
  manager_name: string | null
  is_active: boolean
  sort_order: number
}

export interface CreateDepartmentRequest {
  code: string
  name: string
  parent_id?: string
  manager_id?: string
  sort_order?: number
}

export interface UpdateDepartmentRequest {
  name?: string
  parent_id?: string
  manager_id?: string
  is_active?: boolean
  sort_order?: number
}
