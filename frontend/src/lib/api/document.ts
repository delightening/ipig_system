import api from './client'

export interface PoReceiptItem {
  product_id: string
  product_sku: string
  product_name: string
  base_uom: string
  uom: string
  unit_price: number | null
  ordered_qty: number
  received_qty: number
  remaining_qty: number
}

export interface PoReceiptStatus {
  po_id: string
  po_no: string
  status: 'pending' | 'partial' | 'complete'
  items: PoReceiptItem[]
}

export const getPoReceiptStatus = async (poId: string): Promise<PoReceiptStatus> => {
  const response = await api.get<PoReceiptStatus>(`/documents/${poId}/receipt-status`)
  return response.data
}

export const adminApproveDocument = async (id: string) => {
  const response = await api.post(`/documents/${id}/admin-approve`)
  return response.data
}

export const adminRejectDocument = async (id: string, reason: string) => {
  const response = await api.post(`/documents/${id}/admin-reject`, { reason })
  return response.data
}

/** R84-5 發起沖銷（倉庫管理員 / 管理員）：建立一張待管理員核准的沖銷單，此階段不動庫存。 */
export const reverseDocument = async (id: string) => {
  const response = await api.post(`/documents/${id}/reverse`)
  return response.data
}

/** R84-5 管理員最終核准沖銷單：執行庫存與會計的反向鏡射（發起人不得自行核准）。 */
export const reverseApproveDocument = async (id: string) => {
  const response = await api.post(`/documents/${id}/reverse-approve`)
  return response.data
}

/**
 * 撤銷已核准、尚無有效下游單據的採購單（倉庫管理員 / 管理員，原因必填）。
 * PO 核准不寫庫存、不過帳，所以不需要二人核准的沖銷；已有有效入庫單者後端會拒絕並引導走沖銷。
 */
export const revokeDocument = async (id: string, reason: string) => {
  const response = await api.post(`/documents/${id}/revoke`, { reason })
  return response.data
}

export const createGrnFromPo = async (poId: string): Promise<{ id: string }> => {
  const response = await api.post(`/documents/${poId}/create-grn`)
  return response.data
}
