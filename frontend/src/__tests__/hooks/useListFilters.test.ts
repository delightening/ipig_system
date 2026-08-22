import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useListFilters } from '@/hooks/useListFilters'

describe('useListFilters', () => {
  it('initializes with defaults', () => {
    const { result } = renderHook(() => useListFilters())
    expect(result.current.search).toBe('')
    expect(result.current.page).toBe(1)
    expect(result.current.perPage).toBe(20)
    expect(result.current.sortColumn).toBeNull()
    expect(result.current.sortDirection).toBe('asc')
  })

  it('accepts custom defaults', () => {
    const { result } = renderHook(() =>
      useListFilters({
        initialFilters: { status: 'active' },
        defaultPerPage: 50,
      })
    )
    expect(result.current.filters.status).toBe('active')
    expect(result.current.perPage).toBe(50)
  })

  it('updates search', () => {
    const { result } = renderHook(() => useListFilters())
    act(() => { result.current.setSearch('keyword') })
    expect(result.current.search).toBe('keyword')
  })

  it('sets individual filter and resets page to 1', () => {
    const { result } = renderHook(() =>
      useListFilters({ initialFilters: { status: '' } })
    )
    act(() => { result.current.setPage(3) })
    expect(result.current.page).toBe(3)

    act(() => { result.current.setFilter('status', 'archived') })
    expect(result.current.filters.status).toBe('archived')
    expect(result.current.page).toBe(1)
  })

  it('resets all filters to initial state', () => {
    const { result } = renderHook(() =>
      useListFilters({ initialFilters: { status: 'default' } })
    )
    act(() => {
      result.current.setSearch('test')
      result.current.setFilter('status', 'changed')
      result.current.setPage(5)
      result.current.setSortColumn('name')
      result.current.setSortDirection('desc')
    })

    act(() => { result.current.resetFilters() })
    expect(result.current.search).toBe('')
    expect(result.current.filters.status).toBe('default')
    expect(result.current.page).toBe(1)
    expect(result.current.sortColumn).toBeNull()
    expect(result.current.sortDirection).toBe('asc')
  })

  it('updates perPage', () => {
    const { result } = renderHook(() => useListFilters())
    act(() => { result.current.setPerPage(100) })
    expect(result.current.perPage).toBe(100)
  })

  it('updates sort column and direction', () => {
    const { result } = renderHook(() => useListFilters())
    act(() => {
      result.current.setSortColumn('created_at')
      result.current.setSortDirection('desc')
    })
    expect(result.current.sortColumn).toBe('created_at')
    expect(result.current.sortDirection).toBe('desc')
  })
})
