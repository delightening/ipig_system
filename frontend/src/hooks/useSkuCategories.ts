import { useMemo } from 'react'
import { useQuery, useQueries } from '@tanstack/react-query'
import api from '@/lib/api'
import type {
  SkuCategoryOption,
  SkuCategoriesResponse,
  SkuSubcategoriesResponse,
} from '@/types/sku'

/**
 * 品類／子類單一來源：從 API 讀取（業界主流：主資料集中於 DB）。
 * 用於新增產品、編輯產品、產品列表篩選、匯入產品等，與 GET /sku/categories 一致。
 */
export function useSkuCategories(options?: { enabled?: boolean }) {
  const enabled = options?.enabled ?? true

  const { data: categoriesData, isLoading: categoriesLoading } = useQuery({
    queryKey: ['sku-categories'],
    queryFn: async () => {
      const res = await api.get<SkuCategoriesResponse>('/sku/categories')
      return res.data
    },
    enabled,
  })

  const categories: SkuCategoryOption[] = categoriesData?.categories ?? []
  const categoryCodes = useMemo(
    () => categories.map((c) => c.code),
    [categories]
  )

  const subcategoryQueries = useQueries({
    queries: categoryCodes.map((code) => ({
      queryKey: ['sku-subcategories', code],
      queryFn: async () => {
        const res = await api.get<SkuSubcategoriesResponse>(
          `/sku/categories/${code}/subcategories`
        )
        return res.data
      },
      enabled: enabled && categoryCodes.length > 0,
    })),
  })

  const subcategoriesByCategory: Record<string, SkuCategoryOption[]> =
    useMemo(() => {
      const out: Record<string, SkuCategoryOption[]> = {}
      categoryCodes.forEach((code, i) => {
        const data = subcategoryQueries[i]?.data
        out[code] = data?.subcategories ?? []
      })
      return out
    }, [categoryCodes, subcategoryQueries])

  const isLoading = categoriesLoading || subcategoryQueries.some((q) => q.isLoading)

  return {
    categories,
    subcategoriesByCategory,
    isLoading,
  }
}
