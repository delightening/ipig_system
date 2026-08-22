import { describe, it, expect } from 'vitest'
import { packLayoutCompact, WidgetLayoutItem } from '@/components/dashboard'

// 任兩個 widget 是否在網格上重疊
function overlaps(a: WidgetLayoutItem, b: WidgetLayoutItem): boolean {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

function makeItem(i: string, w: number, h: number): WidgetLayoutItem {
  // 起始座標刻意打亂，確認 pack 會重新指派而非沿用
  return { i, x: 99, y: 99, w, h, visible: true }
}

describe('packLayoutCompact', () => {
  it('保留 widget 數量、順序與寬高、額外欄位', () => {
    const items: WidgetLayoutItem[] = [
      { i: 'a', x: 0, y: 0, w: 4, h: 2, minW: 2, options: { days: 7 } },
      { i: 'b', x: 0, y: 0, w: 3, h: 2 },
    ]
    const packed = packLayoutCompact(items, 12)
    expect(packed.map((p) => p.i)).toEqual(['a', 'b'])
    expect(packed[0].h).toBe(2)
    expect(packed[0].minW).toBe(2)
    expect(packed[0].options).toEqual({ days: 7 })
  })

  it('產生的佈局任兩個 widget 都不重疊', () => {
    const items = [
      makeItem('a', 12, 2),
      makeItem('b', 4, 3),
      makeItem('c', 3, 3),
      makeItem('d', 5, 4),
      makeItem('e', 2, 6),
      makeItem('f', 9, 5),
    ]
    const packed = packLayoutCompact(items, 12)
    for (let i = 0; i < packed.length; i++) {
      for (let j = i + 1; j < packed.length; j++) {
        expect(overlaps(packed[i], packed[j])).toBe(false)
      }
    }
  })

  it('寬度夾到欄數內、且不超出右邊界', () => {
    const packed = packLayoutCompact([makeItem('wide', 12, 2)], 6)
    expect(packed[0].w).toBe(6)
    expect(packed[0].x + packed[0].w).toBeLessThanOrEqual(6)
  })

  it('垂直壓實：第一個 widget 必定貼齊頂端 (y=0)', () => {
    const packed = packLayoutCompact([makeItem('a', 4, 2), makeItem('b', 4, 2)], 12)
    expect(packed[0].y).toBe(0)
    expect(packed[0].x).toBe(0)
  })

  it('同寬度且未滿欄時於同一列並排（壓實不留垂直空隙）', () => {
    // 兩個 4 寬 widget 在 12 欄應排在同一列 (y=0)，x=0 與 x=4
    const packed = packLayoutCompact([makeItem('a', 4, 3), makeItem('b', 4, 3)], 12)
    expect(packed[0]).toMatchObject({ x: 0, y: 0 })
    expect(packed[1]).toMatchObject({ x: 4, y: 0 })
  })

  it('適應較小的子集：少量 widget 仍從左上角緊湊排列', () => {
    const packed = packLayoutCompact([makeItem('only', 3, 2)], 12)
    expect(packed[0]).toMatchObject({ x: 0, y: 0, w: 3, h: 2 })
  })
})
