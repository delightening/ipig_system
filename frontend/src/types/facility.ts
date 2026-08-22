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

/** 物種清單項目：Species 外加「被幾隻動物引用」（含軟刪除的動物）。使用中的物種不可停用或刪除。 */
export interface SpeciesListItem extends Species {
  animal_count: number
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
  layout_config?: Record<string, unknown> | null
}

export interface UpdateZoneRequest {
  name?: string
  color?: string
  is_active?: boolean
  sort_order?: number
  layout_config?: Record<string, unknown> | null
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

// 值為 i18n key（非顯示字串）；consumer 須以 t() 取顯示文字。
export const PEN_STATUS_NAMES: Record<string, string> = {
  active: 'facility.penStatus.active',
  empty: 'facility.penStatus.empty',
  maintenance: 'facility.penStatus.maintenance',
  disabled: 'facility.penStatus.disabled',
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

/** 部門成員（成員清單 / 組織圖用）。department_id 供組織圖依部門分組。 */
export interface DepartmentMember {
  id: string
  display_name: string
  email: string
  department_id: string | null
  /**
   * 是否為本場受僱人員。
   *
   * 部門可以合法地同時有內部與外部成員（IACUC 外聘委員）；成員名冊與組織圖
   * 若不標示，人一旦編進去就再也分不出誰是外聘的。
   */
  is_internal: boolean
}

/**
 * IACUC 委員名冊的一列。
 *
 * **與部門編制無關**——委員身分來自角色（IACUC_CHAIR / REVIEWER / IACUC_STAFF），
 * 人事歸屬來自 department_id，兩者正交。委員會是跨部門的橫切面（獸醫可以兼任
 * 審查委員），而 department_id 是單值的，裝不下這種雙重身分；故同一個人可以
 * 同時出現在部門樹與這份名冊裡，各自表達不同的事實。
 */
export interface CommitteeMember {
  id: string
  display_name: string
  /** 人事歸屬部門名稱；未編入任何部門時為 null */
  department_name: string | null
  /** 是否為本場受僱人員（外聘委員為 false） */
  is_internal: boolean
}
