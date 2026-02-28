import { useEffect } from 'react'
import { useBlocker } from 'react-router-dom'

export function useUnsavedChangesGuard(isDirty: boolean) {
  const blocker = useBlocker(isDirty)

  useEffect(() => {
    if (!isDirty) return
    const handler = (e: BeforeUnloadEvent) => {
      e.preventDefault()
      e.returnValue = ''
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [isDirty])

  return blocker
}
