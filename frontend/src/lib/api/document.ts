import api from './client'

export interface PoReceiptItem {
  product_id: string
  product_name: string
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
