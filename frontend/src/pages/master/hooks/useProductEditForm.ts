import { useState, useEffect, useMemo, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, { Product } from '@/lib/api'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { toast } from '@/components/ui/use-toast'
import { UOM_MAP } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'

export interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  pack_unit?: string
  pack_qty?: number
  default_expiry_days?: number
  safety_stock_uom?: string
  reorder_point_uom?: string
  barcode?: string
  storage_condition?: string
  license_no?: string
  remark?: string
  tags?: string[]
}

/** Reverse-lookup: display name or code -> unit code */
function unitToCode(value: string | undefined): string {
  if (!value?.trim()) return ''
  if (UOM_MAP[value]) return value
  const entry = Object.entries(UOM_MAP).find(([, name]) => name === value)
  return entry ? entry[0] : value
}

export interface ProductEditFormState {
  name: string
  spec: string
  categoryCode: string
  subcategoryCode: string
  packagingLayers: 2 | 3
  outerUnitCode: string
  outerQty: number
  innerUnitCode: string
  innerQty: number
  baseUnitCode: string
  baseQty: number
  trackBatch: boolean
  trackExpiry: boolean
  defaultExpiryDays: number | ''
  safetyStock: number | ''
  safetyStockUom: string
  reorderPoint: number | ''
  reorderPointUom: string
  barcode: string
  storageCondition: string
  licenseNo: string
  remark: string
  tagsInput: string
}

export type ProductEditFormReturn = ReturnType<typeof useProductEditForm>

export function useProductEditForm(id: string | undefined) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const {
    categories: skuCategories,
    subcategoriesByCategory,
    isLoading: skuCategoriesLoading,
  } = useSkuCategories()

  const displayCategories = useMemo(
    () => skuCategories.filter((c) => c.code !== 'GEN').slice(0, 5),
    [skuCategories],
  )

  const hasSubcategories = useCallback(
    (catCode: string) => {
      const subList = subcategoriesByCategory[catCode] ?? []
      if (subList.length === 0) return false
      if (subList.length === 1 && subList[0].code === catCode) return false
      return true
    },
    [subcategoriesByCategory],
  )

  const getSubcategories = useCallback(
    (catCode: string) => {
      const subList = subcategoriesByCategory[catCode] ?? []
      if (subList.length === 1 && subList[0].code === catCode) return []
      return subList
    },
    [subcategoriesByCategory],
  )

  // Form state
  const [form, setForm] = useState<ProductEditFormState>({
    name: '',
    spec: '',
    categoryCode: '',
    subcategoryCode: '',
    packagingLayers: 2,
    outerUnitCode: '',
    outerQty: 1,
    innerUnitCode: '',
    innerQty: 1,
    baseUnitCode: '',
    baseQty: 1,
    trackBatch: false,
    trackExpiry: false,
    defaultExpiryDays: '',
    safetyStock: '',
    safetyStockUom: '',
    reorderPoint: '',
    reorderPointUom: '',
    barcode: '',
    storageCondition: '',
    licenseNo: '',
    remark: '',
    tagsInput: '',
  })

  const updateField = useCallback(
    <K extends keyof ProductEditFormState>(key: K, value: ProductEditFormState[K]) => {
      setForm((prev) => ({ ...prev, [key]: value }))
    },
    [],
  )

  // Product query
  const {
    data: product,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['product', id],
    queryFn: async () => {
      const response = await api.get<ExtendedProduct>(`/products/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  // Populate form when product loads
  useEffect(() => {
    if (!product) return

    const catCode = product.category_code ?? ''
    const subCode = product.subcategory_code ?? ''
    const resolvedCat = catCode === 'LAB' ? 'CON' : catCode
    const resolvedSub = catCode === 'LAB' ? (subCode || 'LAB') : subCode

    const pu = unitToCode(product.pack_unit || product.base_uom)
    const bu = unitToCode(product.base_uom)
    const pq = product.pack_qty ?? 1

    setForm({
      name: product.name,
      spec: product.spec ?? '',
      categoryCode: resolvedCat,
      subcategoryCode: resolvedSub,
      packagingLayers: 2,
      outerUnitCode: pu,
      outerQty: 1,
      innerUnitCode: bu,
      innerQty: pq,
      baseUnitCode: bu,
      baseQty: 1,
      trackBatch: product.track_batch,
      trackExpiry: product.track_expiry,
      defaultExpiryDays: product.default_expiry_days ?? '',
      safetyStock: product.safety_stock != null ? Number(product.safety_stock) : '',
      safetyStockUom: product.safety_stock_uom ?? product.base_uom ?? '',
      reorderPoint: product.reorder_point != null ? Number(product.reorder_point) : '',
      reorderPointUom: product.reorder_point_uom ?? product.base_uom ?? '',
      barcode: product.barcode ?? '',
      storageCondition: product.storage_condition ?? '',
      licenseNo: product.license_no ?? '',
      remark: product.remark ?? '',
      tagsInput: product.tags?.join(', ') ?? '',
    })
  }, [product])

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async () => {
      const subCode = hasSubcategories(form.categoryCode)
        ? form.subcategoryCode
        : form.categoryCode
      const computedPackUnit =
        form.packagingLayers === 2
          ? form.outerUnitCode || form.innerUnitCode
          : form.innerUnitCode
      const computedPackQty =
        form.packagingLayers === 2
          ? form.outerUnitCode
            ? form.innerQty
            : 1
          : form.innerQty * form.baseQty

      return api.put(`/products/${id}`, {
        name: form.name.trim() || undefined,
        spec: form.spec.trim() || undefined,
        category_code: form.categoryCode || undefined,
        subcategory_code: subCode || undefined,
        pack_unit: computedPackUnit || undefined,
        pack_qty: computedPackQty || undefined,
        track_batch: form.trackBatch,
        track_expiry: form.trackExpiry,
        default_expiry_days:
          form.defaultExpiryDays === '' ? undefined : Number(form.defaultExpiryDays),
        safety_stock: form.safetyStock === '' ? undefined : Number(form.safetyStock),
        safety_stock_uom: form.safetyStockUom || undefined,
        reorder_point: form.reorderPoint === '' ? undefined : Number(form.reorderPoint),
        reorder_point_uom: form.reorderPointUom || undefined,
        barcode: form.barcode.trim() || undefined,
        storage_condition: form.storageCondition || undefined,
        license_no: form.licenseNo.trim() || undefined,
        remark: form.remark.trim() || undefined,
        tags: form.tagsInput.trim()
          ? form.tagsInput
              .split(/,\s*/)
              .map((t) => t.trim())
              .filter(Boolean)
          : undefined,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['product', id] })
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '產品已更新', description: '變更已儲存' })
      navigate(`/products/${id}`)
    },
    onError: (err: unknown) => {
      toast({
        title: '更新失敗',
        description: getApiErrorMessage(err, '儲存時發生錯誤'),
        variant: 'destructive',
      })
    },
  })

  const subcategories = getSubcategories(form.categoryCode)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!form.name.trim()) {
      toast({ title: '請輸入產品名稱', variant: 'destructive' })
      return
    }
    updateMutation.mutate()
  }

  return {
    form,
    updateField,
    product,
    isLoading,
    error,
    displayCategories,
    subcategories,
    skuCategoriesLoading,
    hasSubcategories,
    updateMutation,
    handleSubmit,
    navigate,
  }
}
