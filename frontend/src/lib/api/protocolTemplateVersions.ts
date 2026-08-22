import api from './client'

/** 計畫書範本版本（院區層級，admin 管理） */
export interface ProtocolTemplateVersion {
  id: string
  version_label: string
  effective_date?: string | null
  is_current: boolean
  notes?: string | null
  created_by: string
  created_by_name?: string | null
  created_at: string
  updated_at: string
}

export interface TemplateVersionDocument {
  id: string
  file_name: string
  file_path: string
  file_size: number
  mime_type: string
  created_at: string
}

export interface CreateTemplateVersionRequest {
  version_label: string
  effective_date?: string | null
  notes?: string | null
}

export interface UpdateTemplateVersionRequest {
  version_label?: string | null
  effective_date?: string | null
  notes?: string | null
}

const BASE = '/protocol-template-versions'

export const listTemplateVersions = async () =>
  (await api.get<ProtocolTemplateVersion[]>(BASE)).data

export const createTemplateVersion = async (data: CreateTemplateVersionRequest) =>
  (await api.post<ProtocolTemplateVersion>(BASE, data)).data

export const updateTemplateVersion = async (id: string, data: UpdateTemplateVersionRequest) =>
  (await api.put<ProtocolTemplateVersion>(`${BASE}/${id}`, data)).data

export const setCurrentTemplateVersion = async (id: string) =>
  (await api.post<ProtocolTemplateVersion>(`${BASE}/${id}/set-current`)).data

export const deleteTemplateVersion = async (id: string) => {
  await api.delete(`${BASE}/${id}`)
}

export const listTemplateVersionDocuments = async (id: string) =>
  (await api.get<TemplateVersionDocument[]>(`${BASE}/${id}/documents`)).data

export const uploadTemplateVersionDocument = async (id: string, file: File) => {
  const formData = new FormData()
  formData.append('file', file)
  return (await api.post(`${BASE}/${id}/documents`, formData)).data
}

export const deleteTemplateVersionDocument = async (id: string, docId: string) => {
  await api.delete(`${BASE}/${id}/documents/${docId}`)
}
