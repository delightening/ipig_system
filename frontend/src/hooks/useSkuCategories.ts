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

  const categories = useMemo<SkuCategoryOption[]>(
    () => categoriesData?.categories ?? [],
    [categoriesData]
  )
  const categoryCodes = useMemo(
    () => categories.map((c) => c.code),
    [categories]
  )

  // useQueries 回傳的 results 陣列每次 render 都是新 reference；若直接放進
  // useMemo deps 會讓 subcategoriesByCategory 每 render 變新物件，連鎖使下游
  // useCallback/useEffect 失效並觸發無限 re-render（React #185）。改用 combine：
  // TanStack 對其回傳值套用 structural sharing，內容不變時 reference 穩定。
  const { subcategoriesByCategory, subcategoriesLoading } = useQueries({
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
    combine: (results) => {
      const out: Record<string, SkuCategoryOption[]> = {}
      categoryCodes.forEach((code, i) => {
        out[code] = results[i]?.data?.subcategories ?? []
      })
      return {
        subcategoriesByCategory: out,
        subcategoriesLoading: results.some((q) => q.isLoading),
      }
    },
  })

  const isLoading = categoriesLoading || subcategoriesLoading

  return {
    categories,
    subcategoriesByCategory,
    isLoading,
  }
}
