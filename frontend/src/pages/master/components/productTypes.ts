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

/** 產品狀態選項（labelKey 為 i18n 鍵，渲染時才 t()） */
export const STATUS_OPTIONS = [
  { value: 'all', labelKey: 'common.allStatus' },
  { value: 'active', labelKey: 'erpMaster.common.active' },
  { value: 'inactive', labelKey: 'erpMaster.common.inactive' },
  { value: 'discontinued', labelKey: 'erpMaster.products.status.discontinued' },
] as const

/** 布林篩選選項（labelKey 為 i18n 鍵，渲染時才 t()） */
export const BOOLEAN_OPTIONS = [
  { value: 'all', labelKey: 'erpMaster.common.all' },
  { value: 'true', labelKey: 'common.yes' },
  { value: 'false', labelKey: 'common.no' },
] as const

export type { CategoryOption }
