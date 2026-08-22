import api from './client'

import type {
  Amendment,
  AmendmentReviewAssignment,
  AmendmentStatusHistory,
  RecordAmendmentDecisionRequest,
  ClassifyAmendmentRequest,
  ChangeAmendmentStatusRequest,
} from '@/types/amendment'

/**
 * 計畫變更申請（Amendment）API。對齊後端 backend/src/routes/protocol.rs 的 `/amendments/*`。
 * 投票結算規則見 services/amendment/workflow.rs::check_all_decisions_tx：
 * 全體委員投完才結算，precedence REVISION > REJECT > APPROVE。
 */
const base = (id: string) => `/amendments/${id}`

export const amendmentApi = {
  get: (id: string) => api.get<Amendment>(base(id)),
  getAssignments: (id: string) =>
    api.get<AmendmentReviewAssignment[]>(`${base(id)}/assignments`),
  getHistory: (id: string) =>
    api.get<AmendmentStatusHistory[]>(`${base(id)}/history`),
  recordDecision: (id: string, data: RecordAmendmentDecisionRequest) =>
    api.post<AmendmentReviewAssignment>(`${base(id)}/decision`, data),
  classify: (id: string, data: ClassifyAmendmentRequest) =>
    api.post<Amendment>(`${base(id)}/classify`, data),
  startReview: (id: string) =>
    api.post<Amendment>(`${base(id)}/start-review`),
  changeStatus: (id: string, data: ChangeAmendmentStatusRequest) =>
    api.post<Amendment>(`${base(id)}/status`, data),
}
