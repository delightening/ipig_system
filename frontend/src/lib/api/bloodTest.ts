import api, { deleteResource } from './client'

import type {
  BloodTestListItem, AnimalBloodTestWithItems, CreateBloodTestRequest,
  UpdateBloodTestRequest, BloodTestTemplate, CreateBloodTestTemplateRequest,
  UpdateBloodTestTemplateRequest, BloodTestPanel, CreateBloodTestPanelRequest,
  UpdateBloodTestPanelRequest, UpdateBloodTestPanelItemsRequest,
  BloodTestPreset, CreateBloodTestPresetRequest, UpdateBloodTestPresetRequest,
  BloodTestAnalysisRow,
} from '@/types'

export const bloodTestApi = {
  listByAnimal: (animalId: string) =>
    api.get<BloodTestListItem[]>(`/animals/${animalId}/blood-tests`),
  getById: (id: string) =>
    api.get<AnimalBloodTestWithItems>(`/blood-tests/${id}`),
  create: (animalId: string, data: CreateBloodTestRequest) =>
    api.post<AnimalBloodTestWithItems>(`/animals/${animalId}/blood-tests`, data),
  update: (id: string, data: UpdateBloodTestRequest) =>
    api.put<AnimalBloodTestWithItems>(`/blood-tests/${id}`, data),
  delete: (id: string, reason: string) =>
    deleteResource(`/blood-tests/${id}`, { data: { reason } }),
}

export const bloodTestTemplateApi = {
  list: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates'),
  listAll: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates/all'),
  create: (data: CreateBloodTestTemplateRequest) =>
    api.post<BloodTestTemplate>('/blood-test-templates', data),
  update: (id: string, data: UpdateBloodTestTemplateRequest) =>
    api.put<BloodTestTemplate>(`/blood-test-templates/${id}`, data),
  delete: (id: string) =>
    deleteResource(`/blood-test-templates/${id}`),
}

export const bloodTestPanelApi = {
  list: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels'),
  listAll: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels/all'),
  create: (data: CreateBloodTestPanelRequest) =>
    api.post<BloodTestPanel>('/blood-test-panels', data),
  update: (id: string, data: UpdateBloodTestPanelRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}`, data),
  updateItems: (id: string, data: UpdateBloodTestPanelItemsRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}/items`, data),
  delete: (id: string) =>
    deleteResource(`/blood-test-panels/${id}`),
}

export const bloodTestPresetApi = {
  list: () =>
    api.get<BloodTestPreset[]>('/blood-test-presets'),
  listAll: () =>
    api.get<BloodTestPreset[]>('/blood-test-presets/all'),
  create: (data: CreateBloodTestPresetRequest) =>
    api.post<BloodTestPreset>('/blood-test-presets', data),
  update: (id: string, data: UpdateBloodTestPresetRequest) =>
    api.put<BloodTestPreset>(`/blood-test-presets/${id}`, data),
  delete: (id: string) =>
    deleteResource(`/blood-test-presets/${id}`),
}

export const bloodTestAnalysisApi = {
  query: (params: string) =>
    api.get<BloodTestAnalysisRow[]>(`/reports/blood-test-analysis?${params}`),
}
