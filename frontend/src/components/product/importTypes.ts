import type { SkuCategoryOption } from '@/types/sku'

export interface ProductImportErrorDetail {
  row: number
  sku?: string
  error: string
}

export interface ProductImportResult {
  success_count: number
  error_count: number
  errors?: ProductImportErrorDetail[]
}

export interface ProductImportDuplicateItem {
  row: number
  name: string
  spec?: string
  existing_sku: string
  existing_id: string
}

export interface ProductImportCheckResult {
  total_rows: number
  duplicate_count: number
  duplicates: ProductImportDuplicateItem[]
  has_sku_column: boolean
}

export interface ProductImportPreviewRow {
  row: number
  name: string
  spec?: string
  category_code?: string
  subcategory_code?: string
  base_uom: string
  track_batch: boolean
  track_expiry: boolean
  safety_stock?: number
  remark?: string
}

export interface ProductImportPreviewResult {
  rows: ProductImportPreviewRow[]
  has_sku_column: boolean
}

export interface SkuPreviewTableProps {
  previewRows: ProductImportPreviewRow[]
  skuOverrides: Record<number, string>
  setSkuOverrides: React.Dispatch<React.SetStateAction<Record<number, string>>>
  rowCategoryCode: Record<number, string>
  setRowCategoryCode: React.Dispatch<React.SetStateAction<Record<number, string>>>
  rowSubcategoryCode: Record<number, string>
  setRowSubcategoryCode: React.Dispatch<React.SetStateAction<Record<number, string>>>
  skuCategories: SkuCategoryOption[]
  subcategoriesByCategory: Record<string, SkuCategoryOption[]>
  generateSkuIsPending: boolean
  importIsPending: boolean
  onGenerateSku: (
    row: number,
    category: string,
    subcategory: string,
    onSuccess: (sku: string) => void
  ) => void
  onConfirmImport: () => void
  onBack: () => void
  setCategorySubcategoryOverrides: React.Dispatch<
    React.SetStateAction<Record<number, { category_code: string; subcategory_code: string }>>
  >
}

export interface DuplicateWarningProps {
  checkResult: ProductImportCheckResult
  importIsPending: boolean
  onSkipDuplicates: () => void
  onImportWithNewSku: () => void
  onImportAnyway: () => void
}

export interface ImportResultSummaryProps {
  result: ProductImportResult
}

export interface NoSkuColumnPromptProps {
  previewMutationIsPending: boolean
  importIsPending: boolean
  hasDuplicates: boolean
  onSetSkuManually: () => void
  onAutoGenerateSku: () => void
  onDownloadTemplate: () => void
}
