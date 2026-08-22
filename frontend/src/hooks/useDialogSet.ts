import { useState, useCallback } from 'react'

/**
 * 集中管理多個 Dialog 的開關狀態。
 * 適用於 CRUD 頁面有多個 Dialog（create、edit、import、delete）的場景。
 *
 * @param keys Dialog 識別鍵陣列，例如 ['create', 'edit', 'import']
 * @returns isOpen、open、close、setOpen、openDialogs
 *
 * @example
 * const dialogs = useDialogSet(['create', 'edit', 'import'])
 * <Dialog open={dialogs.isOpen('create')} onOpenChange={dialogs.setOpen('create')}>
 */
export function useDialogSet<K extends string>(keys: readonly K[]) {
  const [openDialogs, setOpenDialogs] = useState<Record<K, boolean>>(
    () => Object.fromEntries(keys.map((k) => [k, false])) as Record<K, boolean>
  )

  const open = useCallback((key: K) => {
    setOpenDialogs((p) => ({ ...p, [key]: true }))
  }, [])

  const close = useCallback((key: K) => {
    setOpenDialogs((p) => ({ ...p, [key]: false }))
  }, [])

  const setOpen = useCallback((key: K) => {
    return (value: boolean) => {
      setOpenDialogs((p) => ({ ...p, [key]: value }))
    }
  }, [])

  const isOpen = useCallback(
    (key: K): boolean => !!openDialogs[key],
    [openDialogs]
  )

  return {
    isOpen,
    open,
    close,
    setOpen,
    openDialogs,
  }
}
