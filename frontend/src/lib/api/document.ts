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
