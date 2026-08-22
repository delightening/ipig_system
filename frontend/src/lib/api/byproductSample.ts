import api from './client'

// R53-3 backend `ByproductSample` entity (TIMESTAMPTZ → ISO string at JSON 邊界)
export interface ByproductSample {
  id: string
  // #445：強制安樂死單來源；null = 來自計劃內犧牲（無安樂死單）
  euthanasia_id: string | null
  animal_id: string
  source_protocol_id: string
  sampled_at: string                  // ISO 8601 UTC
  sample_content: string
  requester_user_id: string | null
  // R53-14: external requester 機構名 + 聯絡人（雙層，取代舊 requester_text 單欄）
  requester_org_name: string | null
  requester_contact_name: string | null
  collector_id: string
  notes: string | null
  created_at: string
  updated_at: string
  created_by: string
  deleted_at: string | null
}

// R53-4 HTTP `CreateByproductSampleHttpRequest`（euthanasia_id 走 path 不在 body）
//
// **IDOR 守衛**（R53-4 review-fix）：不送 `animal_id` / `source_protocol_id`，
// backend service 從 path `euthanasia_id` 內部推導
// （euthanasia_orders.animal_id → animals.iacuc_no → protocols.id）。
export interface CreateByproductSampleRequest {
  sampled_at: string
  sample_content: string
  requester_user_id?: string
  // external requester：機構名 + 聯絡人須同時提供（backend validate_requester）
  requester_org_name?: string
  requester_contact_name?: string
  notes?: string
}

// R53-3 backend `UpdateByproductSampleRequest`
// requester 群組整組覆寫語意：送了任一 requester 欄位即整組以送出值為準（含清空另一型別）。
export interface UpdateByproductSampleRequest {
  sampled_at?: string
  sample_content?: string
  requester_user_id?: string
  requester_org_name?: string
  requester_contact_name?: string
  notes?: string
}

export const byproductSampleApi = {
  listByEuthanasia: (euthanasiaId: string) =>
    api.get<ByproductSample[]>(`/euthanasia/${euthanasiaId}/byproduct-samples`),
  listByAnimal: (animalId: string) =>
    api.get<ByproductSample[]>(`/animals/${animalId}/byproduct-samples`),
  listByProtocol: (protocolId: string) =>
    api.get<ByproductSample[]>(`/protocols/${protocolId}/byproduct-samples`),
  get: (id: string) => api.get<ByproductSample>(`/byproduct-samples/${id}`),
  create: (euthanasiaId: string, data: CreateByproductSampleRequest) =>
    api.post<ByproductSample>(`/euthanasia/${euthanasiaId}/byproduct-samples`, data),
  // #445：計劃內犧牲路徑建立（不需安樂死單；backend 驗動物已犧牲）
  createForAnimal: (animalId: string, data: CreateByproductSampleRequest) =>
    api.post<ByproductSample>(`/animals/${animalId}/byproduct-samples`, data),
  update: (id: string, data: UpdateByproductSampleRequest) =>
    api.patch<ByproductSample>(`/byproduct-samples/${id}`, data),
  delete: (id: string) => api.delete(`/byproduct-samples/${id}`),
}
