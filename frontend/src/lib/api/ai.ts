import api from './client'

export interface AiApiKeyInfo {
  id: string
  name: string
  key_prefix: string
  scopes: string[]
  is_active: boolean
  expires_at: string | null
  last_used_at: string | null
  usage_count: number
  rate_limit_per_minute: number
  created_at: string
}

export interface CreateAiApiKeyRequest {
  name: string
  scopes: string[]
  expires_at?: string | null
  rate_limit_per_minute: number
}

export interface CreateAiApiKeyResponse {
  id: string
  name: string
  api_key: string
  scopes: string[]
  expires_at: string | null
  rate_limit_per_minute: number
  created_at: string
}

export const aiApi = {
  listKeys: () =>
    api.get<AiApiKeyInfo[]>('/ai/admin/keys').then(r => r.data),

  createKey: (req: CreateAiApiKeyRequest) =>
    api.post<CreateAiApiKeyResponse>('/ai/admin/keys', req).then(r => r.data),

  toggleKey: (id: string, is_active: boolean) =>
    api.put(`/ai/admin/keys/${id}/toggle`, { is_active }),

  deleteKey: (id: string) =>
    api.delete(`/ai/admin/keys/${id}`),
}
