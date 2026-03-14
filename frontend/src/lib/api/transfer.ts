import api from './client'

import type {
  AnimalTransfer, TransferVetEvaluation,
  CreateTransferRequest, VetEvaluateTransferRequest,
  AssignTransferPlanRequest, RejectTransferRequest,
} from '@/types/animal'

export const transferApi = {
  getDataBoundary: (animalId: string) =>
    api.get<{ boundary: string | null }>(`/animals/${animalId}/data-boundary`),
  list: (animalId: string) =>
    api.get<AnimalTransfer[]>(`/animals/${animalId}/transfers`),
  get: (transferId: string) =>
    api.get<AnimalTransfer>(`/transfers/${transferId}`),
  initiate: (animalId: string, data: CreateTransferRequest) =>
    api.post<AnimalTransfer>(`/animals/${animalId}/transfers`, data),
  vetEvaluate: (transferId: string, data: VetEvaluateTransferRequest) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/vet-evaluate`, data),
  getVetEvaluation: (transferId: string) =>
    api.get<TransferVetEvaluation | null>(`/transfers/${transferId}/vet-evaluation`),
  assignPlan: (transferId: string, data: AssignTransferPlanRequest) =>
    api.put<AnimalTransfer>(`/transfers/${transferId}/assign-plan`, data),
  approve: (transferId: string) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/approve`),
  complete: (transferId: string) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/complete`),
  reject: (transferId: string, data: RejectTransferRequest) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/reject`, data),
}
