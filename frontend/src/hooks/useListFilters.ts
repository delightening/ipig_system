import { useState, useCallback, useMemo, useRef } from 'react'

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

  // gemini-fix-2：用 useRef 凍結 initialFilters 第一次值，避免呼叫端傳物件字面量
  // （如 `useListFilters({ initialFilters: {} })`）每 render 產生新 reference 導致
  // resetFilters / 整個 return 物件不穩定，使 useMemo 失效。
  const initialFiltersRef = useRef(initialFilters)

  const [search, setSearch] = useState('')
  const [filters, setFilters] = useState<TFilters>(initialFiltersRef.current)
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
    setFilters(initialFiltersRef.current)
    setPage(1)
    setSortColumn(null)
    setSortDirection('asc')
  }, [])

  // R34-17: useMemo 包穩定 reference，避免呼叫端把整個回傳物件放進 useEffect/useCallback deps
  // 觸發 infinite loop（CLAUDE.md「禁止把 hook return obj 放 deps」反例）。
  return useMemo(
    () => ({
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
    }),
    [search, filters, page, perPage, sortColumn, sortDirection, setFilter, resetFilters],
  )
}
