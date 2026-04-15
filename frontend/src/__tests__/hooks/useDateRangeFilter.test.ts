import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'

describe('useDateRangeFilter', () => {
  it('defaults to empty strings', () => {
    const { result } = renderHook(() => useDateRangeFilter())
    expect(result.current.from).toBe('')
    expect(result.current.to).toBe('')
  })

  it('accepts initial values', () => {
    const { result } = renderHook(() =>
      useDateRangeFilter({ initialFrom: '2024-01-01', initialTo: '2024-12-31' })
    )
    expect(result.current.from).toBe('2024-01-01')
    expect(result.current.to).toBe('2024-12-31')
  })

  it('accepts lazy initializers', () => {
    const { result } = renderHook(() =>
      useDateRangeFilter({
        initialFrom: () => '2024-06-01',
        initialTo: () => '2024-06-30',
      })
    )
    expect(result.current.from).toBe('2024-06-01')
    expect(result.current.to).toBe('2024-06-30')
  })

  it('sets from and to independently', () => {
    const { result } = renderHook(() => useDateRangeFilter())
    act(() => { result.current.setFrom('2024-03-01') })
    expect(result.current.from).toBe('2024-03-01')
    expect(result.current.to).toBe('')

    act(() => { result.current.setTo('2024-03-31') })
    expect(result.current.to).toBe('2024-03-31')
  })

  it('sets range atomically', () => {
    const { result } = renderHook(() => useDateRangeFilter())
    act(() => { result.current.setRange('2024-01-01', '2024-01-31') })
    expect(result.current.from).toBe('2024-01-01')
    expect(result.current.to).toBe('2024-01-31')
  })

  it('resets to initial values', () => {
    const { result } = renderHook(() =>
      useDateRangeFilter({ initialFrom: '2024-01-01', initialTo: '2024-12-31' })
    )
    act(() => { result.current.setRange('2025-01-01', '2025-12-31') })
    act(() => { result.current.reset() })
    expect(result.current.from).toBe('2024-01-01')
    expect(result.current.to).toBe('2024-12-31')
  })

  it('resets with lazy initializers', () => {
    let callCount = 0
    const lazyFrom = () => { callCount++; return '2024-01-01' }

    const { result } = renderHook(() =>
      useDateRangeFilter({ initialFrom: lazyFrom, initialTo: '2024-12-31' })
    )

    act(() => { result.current.setFrom('2025-06-01') })
    act(() => { result.current.reset() })
    expect(result.current.from).toBe('2024-01-01')
  })
})
