import { useState, useCallback } from 'react'

export interface ListFiltersConfig<TFilters extends Record<string, string> = Record<string, string>> {
  initialFilters?: TFilters
  defaultPerPage?: number
}

/**
 * 列表頁篩選、分頁、排序狀態管理。
 * 適用於產品、倉庫、合作夥伴等 CRUD 列表頁。
 *
 * @param config 可選配置
 * @returns search、filters、page、perPage、sort、setSearch、setFilter、setPage 等
 */
export function useListFilters<TFilters extends Record<string, string> = Record<string, string>>(
  config: ListFiltersConfig<TFilters> = {}
) {
  const {
    initialFilters = {} as TFilters,
    defaultPerPage = 20,
  } = config

  const [search, setSearch] = useState('')
  const [filters, setFilters] = useState<TFilters>(initialFilters)
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(defaultPerPage)
  const [sortColumn, setSortColumn] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

  const setFilter = useCallback(<K extends keyof TFilters>(key: K, value: TFilters[K]) => {
    setFilters((p) => ({ ...p, [key]: value }))
    setPage(1)
  }, [])

  const resetFilters = useCallback(() => {
    setSearch('')
    setFilters(initialFilters)
    setPage(1)
    setSortColumn(null)
    setSortDirection('asc')
  }, [initialFilters])

  return {
    search,
    setSearch,
    filters,
    setFilter,
    setFilters,
    page,
    setPage,
    perPage,
    setPerPage,
    sortColumn,
    setSortColumn,
    sortDirection,
    setSortDirection,
    resetFilters,
  }
}
