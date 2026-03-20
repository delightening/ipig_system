import { useState, useMemo } from 'react'
import { useMutation, useQuery, useQueries, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type {
  SkuCategoryOption,
  SkuCategoriesResponse,
  SkuSubcategoriesResponse,
  GenerateSkuResponse,
} from '@/types/sku'

import type {
  ProductImportResult,
  ProductImportCheckResult,
  ProductImportPreviewRow,
  ProductImportPreviewResult,
} from './importTypes'

export function useProductImport(open: boolean) {
  const queryClient = useQueryClient()
  const [file, setFile] = useState<File | null>(null)
  const [result, setResult] = useState<ProductImportResult | null>(null)
  const [checkResult, setCheckResult] = useState<ProductImportCheckResult | null>(null)
  const [userAcceptedNoSku, setUserAcceptedNoSku] = useState(false)
  const [previewRows, setPreviewRows] = useState<ProductImportPreviewRow[] | null>(null)
  const [skuOverrides, setSkuOverrides] = useState<Record<number, string>>({})
  const [rowCategoryCode, setRowCategoryCode] = useState<Record<number, string>>({})
  const [rowSubcategoryCode, setRowSubcategoryCode] = useState<Record<number, string>>({})
  const [categorySubcategoryOverrides, setCategorySubcategoryOverrides] = useState<
    Record<number, { category_code: string; subcategory_code: string }>
  >({})

  // SKU categories query
  const { data: skuCategoriesData } = useQuery({
    queryKey: ['sku-categories'],
    queryFn: async () => {
      const res = await api.get<SkuCategoriesResponse>('/sku/categories')
      return res.data
    },
    enabled: open && !!previewRows?.length,
  })
  const skuCategories = skuCategoriesData?.categories ?? []

  const selectedCategoryCodes = useMemo(
    () => [...new Set(Object.values(rowCategoryCode).filter(Boolean))],
    [rowCategoryCode]
  )

  const subcategoryQueries = useQueries({
    queries: selectedCategoryCodes.map((code) => ({
      queryKey: ['sku-subcategories', code],
      queryFn: async () => {
        const res = await api.get<SkuSubcategoriesResponse>(`/sku/categories/${code}/subcategories`)
        return res.data
      },
      enabled: open && !!code,
    })),
  })

  const subcategoriesByCategory: Record<string, SkuCategoryOption[]> = useMemo(() => {
    const out: Record<string, SkuCategoryOption[]> = {}
    selectedCategoryCodes.forEach((code, i) => {
      out[code] = subcategoryQueries[i]?.data?.subcategories ?? []
    })
    return out
  }, [selectedCategoryCodes, subcategoryQueries])

  // Mutations
  const generateSkuMutation = useMutation({
    mutationFn: async ({ category, subcategory }: { category: string; subcategory: string }) => {
      const res = await api.post<GenerateSkuResponse>('/sku/generate', { category, subcategory })
      return res.data
    },
    onError: (error: unknown) => {
      toast({ title: '產生 SKU 失敗', description: getApiErrorMessage(error, '請稍後再試'), variant: 'destructive' })
    },
  })

  const importMutation = useMutation({
    mutationFn: async ({ f, skipDuplicates, regenerateSkuForDuplicates }: {
      f: File; skipDuplicates: boolean; regenerateSkuForDuplicates: boolean
    }) => {
      const formData = new FormData()
      formData.append('file', f)
      formData.append('skip_duplicates', String(skipDuplicates))
      formData.append('regenerate_sku_for_duplicates', String(regenerateSkuForDuplicates))
      const res = await api.post<ProductImportResult>('/products/import', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      return res.data
    },
    onSuccess: (data) => {
      setResult(data)
      setCheckResult(null)
      setPreviewRows(null)
      setSkuOverrides({})
      queryClient.invalidateQueries({ queryKey: ['products'] })
      if (data.error_count === 0) {
        toast({ title: '匯入成功', description: `成功匯入 ${data.success_count} 筆產品` })
      } else {
        toast({
          title: '匯入完成（部分失敗）',
          description: `成功: ${data.success_count} 筆，失敗: ${data.error_count} 筆`,
          variant: 'destructive',
        })
      }
    },
    onError: (error: unknown) => {
      toast({ title: '匯入失敗', description: getApiErrorMessage(error, '發生未知錯誤'), variant: 'destructive' })
    },
  })

  const doImport = (f: File, skipDuplicates: boolean, regenerateSkuForDuplicates = false) => {
    importMutation.mutate({ f, skipDuplicates, regenerateSkuForDuplicates })
  }

  const checkMutation = useMutation({
    mutationFn: async (f: File) => {
      const formData = new FormData()
      formData.append('file', f)
      const res = await api.post<ProductImportCheckResult>('/products/import/check', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      return res.data
    },
    onSuccess: (data, f) => {
      setCheckResult(data)
      if (data.duplicate_count === 0 && data.has_sku_column && f) {
        doImport(f, false)
      }
    },
    onError: (error: unknown) => {
      toast({ title: '預檢失敗', description: getApiErrorMessage(error, '無法檢查重複'), variant: 'destructive' })
    },
  })

  const previewMutation = useMutation({
    mutationFn: async (f: File) => {
      const formData = new FormData()
      formData.append('file', f)
      const res = await api.post<ProductImportPreviewResult>('/products/import/preview', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      return res.data
    },
    onSuccess: (data) => {
      setPreviewRows(data.rows)
      setSkuOverrides({})
      const initialCat: Record<number, string> = {}
      const initialSub: Record<number, string> = {}
      const initialOverrides: Record<number, { category_code: string; subcategory_code: string }> = {}
      data.rows.forEach((r) => {
        if (r.category_code?.trim()) {
          const cat = r.category_code.trim()
          const sub = r.subcategory_code?.trim()
          initialCat[r.row] = cat
          if (sub) initialSub[r.row] = sub
          initialOverrides[r.row] = { category_code: cat, subcategory_code: sub || 'OTH' }
        }
      })
      setRowCategoryCode(initialCat)
      setRowSubcategoryCode(initialSub)
      setCategorySubcategoryOverrides(initialOverrides)
    },
    onError: (error: unknown) => {
      toast({ title: '預覽失敗', description: getApiErrorMessage(error, '無法解析檔案'), variant: 'destructive' })
    },
  })

  // Handlers
  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (selectedFiles && selectedFiles.length > 0) {
      setFile(selectedFiles[0])
      setCheckResult(null)
      setResult(null)
      setUserAcceptedNoSku(false)
    }
  }

  const handleImport = () => {
    if (!file) {
      toast({ title: '錯誤', description: '請先選擇檔案', variant: 'destructive' })
      return
    }
    setCheckResult(null)
    checkMutation.mutate(file)
  }

  const resetPreviewState = () => {
    setPreviewRows(null)
    setSkuOverrides({})
    setRowCategoryCode({})
    setRowSubcategoryCode({})
    setCategorySubcategoryOverrides({})
  }

  const handleClose = (onOpenChange: (open: boolean) => void) => {
    setFile(null)
    setResult(null)
    setCheckResult(null)
    setUserAcceptedNoSku(false)
    resetPreviewState()
    onOpenChange(false)
  }

  const buildCsvWithSku = (): File => {
    if (!previewRows?.length) throw new Error('無預覽資料')
    const escape = (v: string) => {
      const s = String(v ?? '').trim()
      if (s.includes(',') || s.includes('"') || s.includes('\n') || s.includes('\r')) {
        return '"' + s.replace(/"/g, '""') + '"'
      }
      return s
    }
    const header = 'SKU編碼,名稱,規格,品類代碼,子類代碼,單位,追蹤批號,追蹤效期,安全庫存,備註'
    const lines = previewRows.map((r) => {
      const sku = skuOverrides[r.row]?.trim() ?? ''
      const catOverride = categorySubcategoryOverrides[r.row]
      const cc = catOverride?.category_code ?? r.category_code ?? ''
      const sc = catOverride?.subcategory_code ?? r.subcategory_code ?? ''
      return [
        escape(sku), escape(r.name), escape(r.spec ?? ''),
        escape(cc), escape(sc), escape(r.base_uom),
        r.track_batch ? 'true' : 'false', r.track_expiry ? 'true' : 'false',
        escape(r.safety_stock != null ? String(r.safety_stock) : ''), escape(r.remark ?? ''),
      ].join(',')
    })
    const csv = '\uFEFF' + header + '\n' + lines.join('\n')
    return new File([new Blob([csv], { type: 'text/csv;charset=utf-8' })], 'product_import_with_sku.csv')
  }

  const handleConfirmImportWithSku = () => {
    try {
      const f = buildCsvWithSku()
      doImport(f, false, false)
    } catch (e) {
      toast({
        title: '無法產生匯入檔',
        description: e instanceof Error ? e.message : '請稍後再試',
        variant: 'destructive',
      })
    }
  }

  const handleDownloadTemplate = async () => {
    try {
      const response = await api.get('/products/import/template', { responseType: 'blob' })
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      const contentDisposition = response.headers['content-disposition']
      let filename = 'product_import_template.xlsx'
      if (contentDisposition) {
        const filenameMatch = contentDisposition.match(/filename="(.+)"/)
        if (filenameMatch) filename = filenameMatch[1]
      }
      link.setAttribute('download', filename)
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
      toast({ title: '下載成功', description: '範本檔案已開始下載' })
    } catch (error: unknown) {
      toast({
        title: '下載失敗',
        description: getApiErrorMessage(error, '無法下載範本檔案'),
        variant: 'destructive',
      })
    }
  }

  const showNoSkuPrompt =
    !!checkResult && !checkResult.has_sku_column && !result && !previewRows &&
    (checkResult.duplicate_count === 0 || !userAcceptedNoSku)

  const showDuplicateWarning =
    !!checkResult && checkResult.duplicate_count > 0 &&
    (checkResult.has_sku_column || userAcceptedNoSku) && !result && !previewRows

  return {
    file, result, checkResult, previewRows, skuOverrides, setSkuOverrides,
    rowCategoryCode, setRowCategoryCode, rowSubcategoryCode, setRowSubcategoryCode,
    skuCategories, subcategoriesByCategory,
    setCategorySubcategoryOverrides, setUserAcceptedNoSku,
    generateSkuMutation, checkMutation, previewMutation, importMutation,
    showNoSkuPrompt, showDuplicateWarning,
    handleFileInputChange, handleImport, handleClose, handleConfirmImportWithSku,
    handleDownloadTemplate, resetPreviewState, doImport,
  }
}
