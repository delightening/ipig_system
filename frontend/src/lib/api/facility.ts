import api, { deleteResource } from './client'

import type {
  Species, SpeciesListItem, CreateSpeciesRequest, UpdateSpeciesRequest,
  Facility, CreateFacilityRequest, UpdateFacilityRequest,
  BuildingWithFacility, CreateBuildingRequest, UpdateBuildingRequest,
  ZoneWithBuilding, CreateZoneRequest, UpdateZoneRequest,
  PenDetails, CreatePenRequest, UpdatePenRequest,
  DepartmentWithManager, CreateDepartmentRequest, UpdateDepartmentRequest,
  CommitteeMember,
  DepartmentMember,
} from '@/types/facility'

export const facilityApi = {
  // ── Species ──────────────────────────────
  /** 預設只回啟用中的物種；物種管理頁要重新啟用停用物種時才帶 includeInactive */
  listSpecies: (includeInactive = false) =>
    api.get<SpeciesListItem[]>('/facilities/species', {
      params: includeInactive ? { include_inactive: true } : undefined,
    }),
  createSpecies: (data: CreateSpeciesRequest) =>
    api.post<Species>('/facilities/species', data),
  updateSpecies: (id: string, data: UpdateSpeciesRequest) =>
    api.put<Species>(`/facilities/species/${id}`, data),
  deleteSpecies: (id: string) =>
    deleteResource(`/facilities/species/${id}`),

  // ── Facility ─────────────────────────────
  listFacilities: () =>
    api.get<Facility[]>('/facilities'),
  createFacility: (data: CreateFacilityRequest) =>
    api.post<Facility>('/facilities', data),
  updateFacility: (id: string, data: UpdateFacilityRequest) =>
    api.put<Facility>(`/facilities/${id}`, data),
  deleteFacility: (id: string) =>
    deleteResource(`/facilities/${id}`),

  // ── Building ─────────────────────────────
  listBuildings: (facility_id?: string) =>
    api.get<BuildingWithFacility[]>('/facilities/buildings', { params: facility_id ? { facility_id } : undefined }),
  createBuilding: (data: CreateBuildingRequest) =>
    api.post<BuildingWithFacility>('/facilities/buildings', data),
  updateBuilding: (id: string, data: UpdateBuildingRequest) =>
    api.put<BuildingWithFacility>(`/facilities/buildings/${id}`, data),
  deleteBuilding: (id: string) =>
    deleteResource(`/facilities/buildings/${id}`),

  // ── Zone ─────────────────────────────────
  listZones: (building_id?: string) =>
    api.get<ZoneWithBuilding[]>('/facilities/zones', { params: building_id ? { building_id } : undefined }),
  createZone: (data: CreateZoneRequest) =>
    api.post<ZoneWithBuilding>('/facilities/zones', data),
  updateZone: (id: string, data: UpdateZoneRequest) =>
    api.put<ZoneWithBuilding>(`/facilities/zones/${id}`, data),
  deleteZone: (id: string) =>
    deleteResource(`/facilities/zones/${id}`),

  // ── Pen ──────────────────────────────────
  listPens: (params?: { zone_id?: string; building_id?: string; facility_id?: string }) =>
    api.get<PenDetails[]>('/facilities/pens', { params }),
  createPen: (data: CreatePenRequest) =>
    api.post<PenDetails>('/facilities/pens', data),
  updatePen: (id: string, data: UpdatePenRequest) =>
    api.put<PenDetails>(`/facilities/pens/${id}`, data),
  deletePen: (id: string) =>
    deleteResource(`/facilities/pens/${id}`),

  // ── Department ───────────────────────────
  listDepartments: () =>
    api.get<DepartmentWithManager[]>('/facilities/departments'),
  createDepartment: (data: CreateDepartmentRequest) =>
    api.post<DepartmentWithManager>('/facilities/departments', data),
  updateDepartment: (id: string, data: UpdateDepartmentRequest) =>
    api.put<DepartmentWithManager>(`/facilities/departments/${id}`, data),
  deleteDepartment: (id: string) =>
    deleteResource(`/facilities/departments/${id}`),

  // ── Department Members（成員指派）─────────
  listDepartmentMembers: (departmentId: string) =>
    api.get<DepartmentMember[]>(`/facilities/departments/${departmentId}/members`),
  listAllDepartmentMembers: () =>
    api.get<DepartmentMember[]>('/facilities/department-members'),
  /** IACUC 委員名冊（依角色，與部門編制無關） */
  listCommitteeMembers: () =>
    api.get<CommitteeMember[]>('/facilities/committee-members'),
  /**
   * 指派使用者到部門。
   *
   * `allowExternal`：明知對方是外部人員仍要編入。後端對外部人員要求這個明示
   * 意圖，否則回 400——UI 的「一併顯示外部人員」開關只是客戶端閘門，擋不住
   * 過期前端或其他呼叫端。
   */
  assignDepartmentMember: (departmentId: string, userId: string, allowExternal = false) =>
    api.post(`/facilities/departments/${departmentId}/members`, {
      user_id: userId,
      allow_external: allowExternal,
    }),
  removeDepartmentMember: (departmentId: string, userId: string) =>
    deleteResource(`/facilities/departments/${departmentId}/members/${userId}`),
}
