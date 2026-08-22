import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useTabState } from '@/hooks/useTabState'

describe('useTabState', () => {
  it('initializes with given tab', () => {
    const { result } = renderHook(() => useTabState('overview'))
    expect(result.current.activeTab).toBe('overview')
  })

  it('switches tab', () => {
    const { result } = renderHook(() => useTabState('tab1'))
    act(() => { result.current.setActiveTab('tab2') })
    expect(result.current.activeTab).toBe('tab2')
  })

  it('accepts string values for Radix UI compatibility', () => {
    const { result } = renderHook(() => useTabState<'a' | 'b'>('a'))
    act(() => { result.current.setActiveTab('b') })
    expect(result.current.activeTab).toBe('b')
  })
})
