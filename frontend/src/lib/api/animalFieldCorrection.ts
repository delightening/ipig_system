import api from './client'

import type {
  AnimalFieldCorrectionRequest,
  BackfillIacucEventDateRequest,
  BackfillIacucEventDateResponse,
  CreateAnimalFieldCorrectionRequest,
  ReviewAnimalFieldCorrectionRequest,
} from '@/types/animal'

export const animalFieldCorrectionApi = {
  listByAnimal: (animalId: string) =>
    api.get<AnimalFieldCorrectionRequest[]>(`/animals/${animalId}/field-corrections`),
  create: (animalId: string, data: CreateAnimalFieldCorrectionRequest) =>
    api.post<{ id: string }>(`/animals/${animalId}/field-corrections`, data),
  listPending: () =>
    api.get<AnimalFieldCorrectionRequest[]>('/animals/animal-field-corrections/pending'),
  review: (requestId: string, data: ReviewAnimalFieldCorrectionRequest) =>
    api.post<{ message: string }>(`/animals/animal-field-corrections/${requestId}/review`, data),
  /** 為 IACUC 變更事件補登實際日期（權限 animal.field_correction.review；只新增不覆蓋）。 */
  backfillIacucEventDate: (
    animalId: string,
    eventId: string,
    data: BackfillIacucEventDateRequest,
  ) =>
    api.post<BackfillIacucEventDateResponse>(
      `/animals/${animalId}/events/${eventId}/actual-date`,
      data,
    ),
}
