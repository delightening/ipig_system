import { useCallback, useEffect, useRef, useState } from 'react'

interface UnsavedChangesGuard {
  isBlocked: boolean
  proceed: () => void
  reset: () => void
}

export function useUnsavedChangesGuard(isDirty: boolean): UnsavedChangesGuard {
  const [isBlocked, setIsBlocked] = useState(false)
  const pendingNavigationRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    if (!isDirty) return
    const handler = (e: BeforeUnloadEvent) => {
      e.preventDefault()
      e.returnValue = ''
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [isDirty])

  const proceed = useCallback(() => {
    setIsBlocked(false)
    pendingNavigationRef.current?.()
    pendingNavigationRef.current = null
  }, [])

  const reset = useCallback(() => {
    setIsBlocked(false)
    pendingNavigationRef.current = null
  }, [])

  return { isBlocked, proceed, reset }
}
