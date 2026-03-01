import { useState, useMemo, useCallback } from 'react'

export interface CategoryOption {
  code: string
  name: string
  subcategories: { code: string; name: string }[]
}

export interface ProductListFilters {
  search: string
  categoryFilter: string
  subcategoryFilter: string
  statusFilter: string
  trackBatchFilter: string
  trackExpiryFilter: string
}

const defaultFilters: ProductListFilters = {
  search: '',
  categoryFilter: 'all',
  subcategoryFilter: 'all',
  statusFilter: 'all',
  trackBatchFilter: 'all',
  trackExpiryFilter: 'all',
}

export function useProductListState(categories: CategoryOption[]) {
  const [filters, setFilters] = useState<ProductListFilters>(defaultFilters)
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(20)
  const [sortBy, setSortBy] = useState<string>('')
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc')

  const subcategories = useMemo(() => {
    const category = categories.find((c) => c.code === filters.categoryFilter)
    return category?.subcategories || []
  }, [categories, filters.categoryFilter])

  const handleCategoryChange = useCallback((value: string) => {
    setFilters((prev) => ({
      ...prev,
      categoryFilter: value,
      subcategoryFilter: 'all',
    }))
    setPage(1)
  }, [])

  const setFilter = useCallback(<K extends keyof ProductListFilters>(
    key: K,
    value: ProductListFilters[K]
  ) => {
    setFilters((prev) => ({ ...prev, [key]: value }))
    setPage(1)
  }, [])

  const handleSort = useCallback((field: string) => {
    setSortBy((prev) => {
      if (prev === field) {
        setSortOrder((o) => (o === 'asc' ? 'desc' : 'asc'))
        return prev
      }
      setSortOrder('asc')
      return field
    })
    setPage(1)
  }, [])

  const queryParams = useMemo(() => {
    const params = new URLSearchParams()
    if (filters.search) params.append('keyword', filters.search)
    if (filters.categoryFilter && filters.categoryFilter !== 'all')
      params.append('category_code', filters.categoryFilter)
    if (filters.subcategoryFilter && filters.subcategoryFilter !== 'all')
      params.append('subcategory_code', filters.subcategoryFilter)
    if (filters.statusFilter && filters.statusFilter !== 'all')
      params.append('status', filters.statusFilter)
    if (filters.trackBatchFilter && filters.trackBatchFilter !== 'all')
      params.append('track_batch', filters.trackBatchFilter)
    if (filters.trackExpiryFilter && filters.trackExpiryFilter !== 'all')
      params.append('track_expiry', filters.trackExpiryFilter)
    params.append('page', page.toString())
    params.append('per_page', perPage.toString())
    if (sortBy) {
      params.append('sort_by', sortBy)
      params.append('sort_order', sortOrder)
    }
    return params.toString()
  }, [filters, page, perPage, sortBy, sortOrder])

  const resetFilters = useCallback(() => {
    setFilters(defaultFilters)
    setPage(1)
    setSortBy('')
    setSortOrder('asc')
  }, [])

  const activeFilterCount = [
    filters.categoryFilter !== 'all' ? filters.categoryFilter : '',
    filters.subcategoryFilter !== 'all' ? filters.subcategoryFilter : '',
    filters.statusFilter !== 'all' ? filters.statusFilter : '',
    filters.trackBatchFilter !== 'all' ? filters.trackBatchFilter : '',
    filters.trackExpiryFilter !== 'all' ? filters.trackExpiryFilter : '',
  ].filter(Boolean).length

  return {
    filters,
    setFilter,
    handleCategoryChange,
    page,
    setPage,
    perPage,
    setPerPage,
    sortBy,
    sortOrder,
    handleSort,
    subcategories,
    queryParams,
    activeFilterCount,
    resetFilters,
  }
}
