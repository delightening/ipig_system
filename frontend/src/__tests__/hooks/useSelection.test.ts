import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useSelection } from '@/hooks/useSelection'

describe('useSelection', () => {
  it('starts empty by default', () => {
    const { result } = renderHook(() => useSelection())
    expect(result.current.size).toBe(0)
  })

  it('accepts initial ids', () => {
    const { result } = renderHook(() => useSelection(['a', 'b']))
    expect(result.current.size).toBe(2)
    expect(result.current.has('a')).toBe(true)
    expect(result.current.has('b')).toBe(true)
  })

  it('toggles selection on', () => {
    const { result } = renderHook(() => useSelection<string>())
    act(() => { result.current.toggle('x') })
    expect(result.current.has('x')).toBe(true)
    expect(result.current.size).toBe(1)
  })

  it('toggles selection off', () => {
    const { result } = renderHook(() => useSelection(['x']))
    act(() => { result.current.toggle('x') })
    expect(result.current.has('x')).toBe(false)
    expect(result.current.size).toBe(0)
  })

  it('selectAll selects all items', () => {
    const { result } = renderHook(() => useSelection<string>())
    act(() => { result.current.selectAll(['a', 'b', 'c']) })
    expect(result.current.size).toBe(3)
    expect(result.current.has('a')).toBe(true)
    expect(result.current.has('b')).toBe(true)
    expect(result.current.has('c')).toBe(true)
  })

  it('selectAll deselects all when already fully selected', () => {
    const { result } = renderHook(() => useSelection(['a', 'b']))
    act(() => { result.current.selectAll(['a', 'b']) })
    expect(result.current.size).toBe(0)
  })

  it('clears all selections', () => {
    const { result } = renderHook(() => useSelection(['a', 'b', 'c']))
    act(() => { result.current.clear() })
    expect(result.current.size).toBe(0)
  })

  it('works with numeric ids', () => {
    const { result } = renderHook(() => useSelection<number>())
    act(() => { result.current.toggle(1) })
    act(() => { result.current.toggle(2) })
    expect(result.current.has(1)).toBe(true)
    expect(result.current.has(2)).toBe(true)
    expect(result.current.size).toBe(2)
  })
})
