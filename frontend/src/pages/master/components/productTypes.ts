import type { Product } from '@/lib/api'
import type { useProductListState } from '../hooks/useProductListState'
import type { CategoryOption } from '../hooks/useProductListState'

/** 擴展產品型別，包含品類與狀態資訊 */
export interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  status?: 'active' | 'inactive' | 'discontinued'
  storage_condition?: string
}

/** useProductListState 回傳型別 */
export type ProductListState = ReturnType<typeof useProductListState>

/** 狀態操作型別 */
export type StatusAction = 'activate' | 'deactivate' | 'discontinue'

/** 產品狀態選項 */
export const STATUS_OPTIONS = [
  { value: 'all', label: '全部狀態' },
  { value: 'active', label: '啟用' },
  { value: 'inactive', label: '停用' },
  { value: 'discontinued', label: '停產' },
] as const

/** 布林篩選選項 */
export const BOOLEAN_OPTIONS = [
  { value: 'all', label: '全部' },
  { value: 'true', label: '是' },
  { value: 'false', label: '否' },
] as const

export type { CategoryOption }
