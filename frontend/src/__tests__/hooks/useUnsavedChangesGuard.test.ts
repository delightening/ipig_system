import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useUnsavedChangesGuard } from '@/hooks/useUnsavedChangesGuard'

describe('useUnsavedChangesGuard', () => {
  let addListenerSpy: ReturnType<typeof vi.spyOn>
  let removeListenerSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    addListenerSpy = vi.spyOn(window, 'addEventListener')
    removeListenerSpy = vi.spyOn(window, 'removeEventListener')
  })

  it('starts with isBlocked false', () => {
    const { result } = renderHook(() => useUnsavedChangesGuard(false))
    expect(result.current.isBlocked).toBe(false)
  })

  it('adds beforeunload listener when dirty', () => {
    renderHook(() => useUnsavedChangesGuard(true))
    expect(addListenerSpy).toHaveBeenCalledWith('beforeunload', expect.any(Function))
  })

  it('does not add beforeunload listener when clean', () => {
    addListenerSpy.mockClear()
    renderHook(() => useUnsavedChangesGuard(false))
    const beforeunloadCalls = addListenerSpy.mock.calls.filter(
      (call: [string, EventListener]) => call[0] === 'beforeunload'
    )
    expect(beforeunloadCalls.length).toBe(0)
  })

  it('removes listener on cleanup', () => {
    const { unmount } = renderHook(() => useUnsavedChangesGuard(true))
    unmount()
    expect(removeListenerSpy).toHaveBeenCalledWith('beforeunload', expect.any(Function))
  })

  it('provides proceed and reset functions', () => {
    const { result } = renderHook(() => useUnsavedChangesGuard(false))
    expect(typeof result.current.proceed).toBe('function')
    expect(typeof result.current.reset).toBe('function')
  })
})
