import { describe, it, expect } from 'vitest'
import { stackByPriority } from '@/components/dashboard'

interface Item {
  i: string
  h: number
  minW?: number
}

const priority = ['a', 'b', 'c']

describe('stackByPriority', () => {
  it('依 priority 順序排列，未列入者排最後', () => {
    const items: Item[] = [
      { i: 'z', h: 2 },
      { i: 'b', h: 2 },
      { i: 'a', h: 2 },
    ]
    const out = stackByPriority(items, 2, priority)
    expect(out.map((o) => o.i)).toEqual(['a', 'b', 'z'])
  })

  it('全部 x=0、滿欄寬，且不重疊（y 依高度嚴格遞增）', () => {
    const items: Item[] = [
      { i: 'a', h: 2 },
      { i: 'b', h: 3 },
      { i: 'c', h: 4 },
    ]
    const out = stackByPriority(items, 4, priority)
    expect(out.every((o) => o.x === 0)).toBe(true)
    expect(out.every((o) => o.w === 4)).toBe(true)
    // y 應為前面所有 widget 高度的累加
    expect(out.map((o) => o.y)).toEqual([0, 2, 5])
    // 任兩個垂直不重疊
    for (let k = 0; k < out.length - 1; k++) {
      expect(out[k].y + out[k].h).toBeLessThanOrEqual(out[k + 1].y)
    }
  })

  it('minW 夾到欄數內，避免窄螢幕被放大溢出', () => {
    const out = stackByPriority([{ i: 'a', h: 2, minW: 3 }], 2, priority)
    expect(out[0].minW).toBe(2)
    expect(out[0].w).toBe(2)
  })

  it('保留原本高度與其他欄位', () => {
    const out = stackByPriority([{ i: 'a', h: 6, minW: 2 }], 2, priority)
    expect(out[0].h).toBe(6)
  })

  it('不修改傳入的原陣列', () => {
    const items: Item[] = [{ i: 'b', h: 2 }, { i: 'a', h: 2 }]
    stackByPriority(items, 2, priority)
    expect(items.map((o) => o.i)).toEqual(['b', 'a'])
  })
})
