import api from './client'

// 動物預約與試驗規劃（Phase 4）API client
// 對應 backend GET /reservation-planning、/animals/reservable、/animals/batch/{reserve,unreserve,assign}

export type PlanningGroupType = 'approved' | 'planned' | 'orphan'
export type PlanningRelation = 'reserved' | 'in_experiment' | 'completed'

/** 規劃分組中的一隻動物（GET /reservation-planning 的 animals[] 元素） */
export interface PlanningAnimalRow {
  id: string
  ear_tag: string
  animal_no: string | null
  gender: 'male' | 'female'
  birth_date: string | null
  latest_weight_kg: string | null
  weight_measured_at: string | null
  remark: string | null
  status: string
  relation: PlanningRelation
}

/** 試驗群組（已核准 protocol 或規劃中 planned_experiment） */
export interface ReservationPlanningGroup {
  group_type: PlanningGroupType
  id: string
  iacuc_no: string | null
  unit: string | null
  pi_name: string | null
  description: string | null
  demand: number
  reserved_count: number
  /** 已分配、實驗中 */
  in_experiment_count: number
  /** 實驗完成（待淘汰）— 仍計入缺口 */
  completed_count: number
  animals: PlanningAnimalRow[]
}

/** 可預約動物（GET /animals/reservable） */
export interface ReservableAnimalRow {
  id: string
  ear_tag: string
  animal_no: string | null
  gender: 'male' | 'female'
  birth_date: string | null
  age_months: number | null
  pen_location: string | null
  latest_weight_kg: string | null
  weight_measured_at: string | null
  remark: string | null
}

export interface ReservableQuery {
  gender?: 'male' | 'female' | null
  age_months_min?: number | null
  age_months_max?: number | null
  weight_min?: number | null
  weight_max?: number | null
}

export interface ReserveBody {
  animal_ids: string[]
  planned_experiment_id?: string
  protocol_id?: string
}

/** 移除 null / undefined / '' 後組 query params */
function pruneParams(q: ReservableQuery): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [k, v] of Object.entries(q)) {
    if (v !== null && v !== undefined && v !== '') out[k] = String(v)
  }
  return out
}

export interface CreatePlannedExperimentBody {
  unit: string
  description?: string | null
  demand_count: number
}

export const reservationPlanningApi = {
  getPlanning: () => api.get<ReservationPlanningGroup[]>('/reservation-planning'),
  createPlannedExperiment: (body: CreatePlannedExperimentBody) => api.post('/planned-experiments', body),
  searchReservable: (q: ReservableQuery) =>
    api.get<ReservableAnimalRow[]>('/animals/reservable', { params: pruneParams(q) }),
  reserve: (body: ReserveBody) => api.post('/animals/batch/reserve', body),
  unreserve: (animalIds: string[]) => api.post('/animals/batch/unreserve', { animal_ids: animalIds }),
  assign: (animalIds: string[], iacucNo: string) =>
    api.post('/animals/batch/assign', { animal_ids: animalIds, iacuc_no: iacucNo }),
  /** 備註 inline 編輯（Phase 5）：空字串 → 後端正規化為 NULL。 */
  updateRemark: (animalId: string, remark: string) =>
    api.patch(`/animals/${animalId}/remark`, { remark }),
}
