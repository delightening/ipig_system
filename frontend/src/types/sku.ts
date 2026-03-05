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

export interface GenerateSkuResponse {
  sku: string
  category: SkuCategoryOption
  subcategory: SkuCategoryOption
  sequence: number
}
