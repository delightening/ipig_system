import { useState, useCallback } from 'react'

/**
 * 表格勾選／多選項目狀態管理。
 * 適用於 ProductsPage 批次操作、TreatmentDrugOptionsPage 匯入選取等。
 *
 * @param initialIds 初始選取 ID 陣列，預設空
 * @returns selectedIds、toggle、selectAll、clear、has、size、setSelectedIds
 */
export function useSelection<T = string>(initialIds: T[] = []) {
  const [selectedIds, setSelectedIds] = useState<Set<T>>(
    () => new Set(initialIds)
  )

  const toggle = useCallback((id: T) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const selectAll = useCallback((ids: T[]) => {
    setSelectedIds((prev) => {
      const allSelected = ids.every((id) => prev.has(id))
      if (allSelected) return new Set()
      return new Set(ids)
    })
  }, [])

  const clear = useCallback(() => {
    setSelectedIds(new Set())
  }, [])

  const has = useCallback(
    (id: T) => selectedIds.has(id),
    [selectedIds]
  )

  const size = selectedIds.size

  return {
    selectedIds,
    setSelectedIds,
    toggle,
    selectAll,
    clear,
    has,
    size,
  }
}
