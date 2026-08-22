import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useTableSort } from '@/hooks/useTableSort'

interface Row {
  name: string
  qty: number
  date: string | null
  nested?: { label: string }
}

const rows: Row[] = [
  { name: '乙', qty: 2, date: '2026-03-01', nested: { label: 'b' } },
  { name: '甲', qty: 10, date: null, nested: { label: 'a' } },
  { name: '丙', qty: 1, date: '2026-01-15', nested: { label: 'c' } },
]

describe('useTableSort', () => {
  it('預設不排序，維持原始順序', () => {
    const { result } = renderHook(() => useTableSort(rows))
    expect(result.current.sort.column).toBeNull()
    expect(result.current.sortedData?.map((r) => r.name)).toEqual(['乙', '甲', '丙'])
  })

  it('點擊循環 asc → desc → 取消排序', () => {
    const { result } = renderHook(() => useTableSort(rows))

    act(() => { result.current.toggleSort('qty') })
    expect(result.current.sort).toEqual({ column: 'qty', direction: 'asc' })
    expect(result.current.sortedData?.map((r) => r.qty)).toEqual([1, 2, 10])

    act(() => { result.current.toggleSort('qty') })
    expect(result.current.sort).toEqual({ column: 'qty', direction: 'desc' })
    expect(result.current.sortedData?.map((r) => r.qty)).toEqual([10, 2, 1])

    // 第三次點擊取消排序 → 回到原始順序
    act(() => { result.current.toggleSort('qty') })
    expect(result.current.sort.column).toBeNull()
    expect(result.current.sortedData?.map((r) => r.name)).toEqual(['乙', '甲', '丙'])
  })

  it('切換到其他欄位時重設為 asc', () => {
    const { result } = renderHook(() => useTableSort(rows))

    act(() => { result.current.toggleSort('qty') })
    act(() => { result.current.toggleSort('qty') })
    expect(result.current.sort.direction).toBe('desc')

    act(() => { result.current.toggleSort('name') })
    expect(result.current.sort).toEqual({ column: 'name', direction: 'asc' })
  })

  it('數字以數值大小排序（非字典序）', () => {
    const { result } = renderHook(() => useTableSort(rows))
    act(() => { result.current.toggleSort('qty') })
    // 字典序會是 1,10,2；數值序才是 1,2,10
    expect(result.current.sortedData?.map((r) => r.qty)).toEqual([1, 2, 10])
  })

  it('日期字串依時間先後排序，null 一律排最後', () => {
    const { result } = renderHook(() => useTableSort(rows))

    act(() => { result.current.toggleSort('date') })
    expect(result.current.sortedData?.map((r) => r.date)).toEqual(['2026-01-15', '2026-03-01', null])

    // 反向時 null 仍在最後（不因方向翻到最前）
    act(() => { result.current.toggleSort('date') })
    expect(result.current.sortedData?.map((r) => r.date)).toEqual(['2026-03-01', '2026-01-15', null])
  })

  it('支援巢狀路徑 a.b', () => {
    const { result } = renderHook(() => useTableSort(rows))
    act(() => { result.current.toggleSort('nested.label') })
    expect(result.current.sortedData?.map((r) => r.nested?.label)).toEqual(['a', 'b', 'c'])
  })

  it('不改動傳入的原始陣列', () => {
    const original = [...rows]
    const { result } = renderHook(() => useTableSort(rows))
    act(() => { result.current.toggleSort('qty') })
    expect(rows).toEqual(original)
  })

  it('data 為 undefined 時回傳 undefined', () => {
    const { result } = renderHook(() => useTableSort<Row>(undefined))
    expect(result.current.sortedData).toBeUndefined()
    act(() => { result.current.toggleSort('qty') })
    expect(result.current.sortedData).toBeUndefined()
  })

  it('可指定預設排序', () => {
    const { result } = renderHook(() =>
      useTableSort(rows, { column: 'qty', direction: 'desc' })
    )
    expect(result.current.sortedData?.map((r) => r.qty)).toEqual([10, 2, 1])
  })
})
