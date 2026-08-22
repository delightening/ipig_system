/** SKU 品類／子類選項（與後端 GET /sku/categories 一致，單一來源） */
export interface SkuCategoryOption {
  code: string
  name: string
}

export interface SkuCategoriesResponse {
  categories: SkuCategoryOption[]
}

export interface SkuSubcategoriesResponse {
  category: SkuCategoryOption
  subcategories: SkuCategoryOption[]
}

/** 編輯用：子類（含 sort_order, is_active） */
export interface SubcategoryForEdit {
  id: number
  code: string
  name: string
  sort_order: number
  is_active: boolean
}

/** 編輯用：品類（含子類） */
export interface CategoryForEdit {
  code: string
  name: string
  sort_order: number
  is_active: boolean
  subcategories: SubcategoryForEdit[]
}

/** 編輯分類用：完整品類樹 */
export interface CategoriesTreeResponse {
  categories: CategoryForEdit[]
}

/** 更新品類請求 */
export interface UpdateSkuCategoryRequest {
  name?: string
  sort_order?: number
  is_active?: boolean
}

/** 更新子類請求 */
export interface UpdateSkuSubcategoryRequest {
  name?: string
  sort_order?: number
  is_active?: boolean
}

/** 新增子類請求 */
export interface CreateSkuSubcategoryRequest {
  code: string
  name: string
  sort_order?: number
  is_active?: boolean
}

export interface GenerateSkuResponse {
  sku: string
  category: SkuCategoryOption
  subcategory: SkuCategoryOption
  sequence: number
}
