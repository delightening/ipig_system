/** Partner type / category display name maps */

export const partnerTypeNames: Record<string, string> = {
  supplier: '供應商',
  customer: '客戶',
}

export const supplierCategoryNames: Record<string, string> = {
  drug: '藥物',
  consumable: '耗材',
  feed: '飼料',
  equipment: '儀器',
}

export const customerCategoryNames: Record<string, string> = {
  internal: '內部單位',
  external: '外部客戶',
  research: '研究計畫',
  other: '其他',
}

export const formatPartnerType = (t: string) =>
  partnerTypeNames[t] ?? t

export const formatSupplierCategory = (c?: string) =>
  c ? (supplierCategoryNames[c] ?? c) : ''

export const formatCustomerCategory = (c?: string) =>
  c ? (customerCategoryNames[c] ?? c) : ''

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
