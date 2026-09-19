import i18n from '@/lib/i18n'

/**
 * Partner type / category 代碼 → i18n 鍵。
 * 值是翻譯鍵而非顯示文字（模組頂層不可存翻譯後字串）；顯示請走下方 format* 函式或 `t(key)`。
 */
export const partnerTypeKeys: Record<string, string> = {
  supplier: 'erpMaster.partners.type.supplier',
  customer: 'erpMaster.partners.type.customer',
}

export const supplierCategoryKeys: Record<string, string> = {
  drug: 'erpMaster.partners.supplierCategory.drug',
  consumable: 'erpMaster.partners.supplierCategory.consumable',
  feed: 'erpMaster.partners.supplierCategory.feed',
  equipment: 'erpMaster.partners.supplierCategory.equipment',
}

export const customerCategoryKeys: Record<string, string> = {
  internal: 'erpMaster.partners.customerCategory.internal',
  external: 'erpMaster.partners.customerCategory.external',
  research: 'erpMaster.partners.customerCategory.research',
  other: 'erpMaster.partners.customerCategory.other',
}

// 以下 format* 在呼叫當下才求值（非 React 情境，例如 CSV 匯出）
// `lng` 可選：內部匯出檔傳 'zh-TW' 固定中文（使用者裁定 2026-09-19）；不傳則隨 UI 語系
export const formatPartnerType = (type: string, lng?: string) =>
  partnerTypeKeys[type] ? i18n.t(partnerTypeKeys[type], { lng }) : type

export const formatSupplierCategory = (c?: string, lng?: string) =>
  c ? (supplierCategoryKeys[c] ? i18n.t(supplierCategoryKeys[c], { lng }) : c) : ''

export const formatCustomerCategory = (c?: string, lng?: string) =>
  c ? (customerCategoryKeys[c] ? i18n.t(customerCategoryKeys[c], { lng }) : c) : ''

export type SupplierCategory = 'drug' | 'consumable' | 'feed' | 'equipment'
export type CustomerCategory = 'internal' | 'external' | 'research' | 'other'

export interface PartnerFormData {
  partner_type: 'supplier' | 'customer'
  supplier_category: '' | SupplierCategory
  customer_category: '' | CustomerCategory
  code: string
  name: string
  tax_id: string
  phone: string
  phone_ext: string
  email: string
  address: string
}

export interface PartnerSubmissionData {
  partner_type: 'supplier' | 'customer'
  supplier_category: SupplierCategory | null
  customer_category: CustomerCategory | null
  code: string | null
  name: string
  tax_id: string | null
  phone: string | null
  phone_ext: string | null
  email: string | null
  address: string | null
}

export const EMPTY_FORM: PartnerFormData = {
  partner_type: 'supplier',
  supplier_category: '',
  customer_category: '',
  code: '',
  name: '',
  tax_id: '',
  phone: '',
  phone_ext: '',
  email: '',
  address: '',
}

const VALID_SUPPLIER_CATEGORIES = ['', 'drug', 'consumable', 'feed', 'equipment'] as const

export function isValidSupplierCategory(
  value: string,
): value is '' | SupplierCategory {
  return (VALID_SUPPLIER_CATEGORIES as readonly string[]).includes(value)
}
