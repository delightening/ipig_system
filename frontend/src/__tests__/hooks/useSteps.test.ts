import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useSteps } from '@/hooks/useSteps'

describe('useSteps', () => {
  it('starts at step 0 by default', () => {
    const { result } = renderHook(() => useSteps(5))
    expect(result.current.step).toBe(0)
    expect(result.current.isFirst).toBe(true)
    expect(result.current.isLast).toBe(false)
  })

  it('accepts initial step', () => {
    const { result } = renderHook(() => useSteps(5, 2))
    expect(result.current.step).toBe(2)
    expect(result.current.isFirst).toBe(false)
    expect(result.current.isLast).toBe(false)
  })

  it('advances with next()', () => {
    const { result } = renderHook(() => useSteps(3))
    act(() => { result.current.next() })
    expect(result.current.step).toBe(1)

    act(() => { result.current.next() })
    expect(result.current.step).toBe(2)
    expect(result.current.isLast).toBe(true)
  })

  it('does not exceed max step', () => {
    const { result } = renderHook(() => useSteps(2))
    act(() => { result.current.next() })
    act(() => { result.current.next() })
    act(() => { result.current.next() })
    expect(result.current.step).toBe(1)
  })

  it('goes back with prev()', () => {
    const { result } = renderHook(() => useSteps(5, 3))
    act(() => { result.current.prev() })
    expect(result.current.step).toBe(2)
  })

  it('does not go below 0', () => {
    const { result } = renderHook(() => useSteps(5))
    act(() => { result.current.prev() })
    expect(result.current.step).toBe(0)
    expect(result.current.isFirst).toBe(true)
  })

  it('navigates with goTo()', () => {
    const { result } = renderHook(() => useSteps(5))
    act(() => { result.current.goTo(3) })
    expect(result.current.step).toBe(3)
  })

  it('clamps goTo within bounds', () => {
    const { result } = renderHook(() => useSteps(5))
    act(() => { result.current.goTo(10) })
    expect(result.current.step).toBe(4)

    act(() => { result.current.goTo(-1) })
    expect(result.current.step).toBe(0)
  })
})
