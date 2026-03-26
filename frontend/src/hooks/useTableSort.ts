/**
 * 通用表格排序 hook
 * 提供前端排序功能，適用於已載入的當頁資料
 */
import { useState, useMemo, useCallback } from 'react'

export type SortDirection = 'asc' | 'desc'

export interface SortState<K extends string = string> {
  column: K | null
  direction: SortDirection
}

/** 取得巢狀物件屬性值 (支援 "a.b.c" 路徑) */
function getNestedValue(obj: unknown, path: string): unknown {
  return path.split('.').reduce((acc: unknown, key) => {
    if (acc && typeof acc === 'object') return (acc as Record<string, unknown>)[key]
    return undefined
  }, obj)
}

function compareValues(a: unknown, b: unknown, direction: SortDirection): number {
  // null / undefined 排最後
  if (a == null && b == null) return 0
  if (a == null) return 1
  if (b == null) return -1

  const multiplier = direction === 'asc' ? 1 : -1

  // 數字
  if (typeof a === 'number' && typeof b === 'number') return (a - b) * multiplier

  // 日期字串 (YYYY-MM-DD 或 ISO)
  const aStr = String(a)
  const bStr = String(b)
  if (/^\d{4}-\d{2}-\d{2}/.test(aStr) && /^\d{4}-\d{2}-\d{2}/.test(bStr)) {
    return aStr.localeCompare(bStr) * multiplier
  }

  // boolean
  if (typeof a === 'boolean' && typeof b === 'boolean') {
    return ((a === b) ? 0 : a ? -1 : 1) * multiplier
  }

  // 字串 (locale 排序)
  return aStr.localeCompare(bStr, 'zh-Hant') * multiplier
}

export function useTableSort<T, K extends string = string>(
  data: T[] | undefined,
  defaultSort?: SortState<K>,
) {
  const [sort, setSort] = useState<SortState<K>>(
    defaultSort ?? { column: null, direction: 'asc' },
  )

  const toggleSort = useCallback((column: K) => {
    setSort((prev) => {
      if (prev.column === column) {
        return prev.direction === 'asc'
          ? { column, direction: 'desc' }
          : { column: null, direction: 'asc' }  // 第三次點擊取消排序
      }
      return { column, direction: 'asc' }
    })
  }, [])

  const sortedData = useMemo(() => {
    if (!data) return undefined
    if (!sort.column) return data

    const col = sort.column as string
    return [...data].sort((a, b) => {
      const aVal = getNestedValue(a, col)
      const bVal = getNestedValue(b, col)
      return compareValues(aVal, bVal, sort.direction)
    })
  }, [data, sort.column, sort.direction])

  return { sortedData, sort, toggleSort } as const
}
