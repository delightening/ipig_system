import api from './client'

import type {
  AnimalFieldCorrectionRequest,
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
}
