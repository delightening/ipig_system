import { describe, it, expect } from 'vitest'
import { LayoutItem } from 'react-grid-layout/legacy'
import {
  buildResponsiveLayouts,
  breakpointFromWidth,
  normalizeDashboardPref,
  GRID_BREAKPOINTS,
  GRID_COLS_BY_BREAKPOINT,
  BREAKPOINT_KEYS,
  DEFAULT_DASHBOARD_LAYOUT,
  WidgetLayoutItem,
} from '@/components/dashboard'

const base: LayoutItem[] = [
  { i: 'a', x: 0, y: 0, w: 4, h: 2 },
  { i: 'b', x: 4, y: 0, w: 4, h: 2 },
  { i: 'c', x: 8, y: 0, w: 4, h: 3 },
]

describe('buildResponsiveLayouts', () => {
  it('回傳全部 7 個斷點，且 lg 即 base 本身', () => {
    const layouts = buildResponsiveLayouts(base)
    expect(Object.keys(layouts).sort()).toEqual([...BREAKPOINT_KEYS].sort())
    expect(layouts.lg).toBe(base)
  })

  it('無 override 時，較寬斷點（xl/xxl）seed 自 lg 並保留座標（欄數較多不需夾）', () => {
    const layouts = buildResponsiveLayouts(base)
    // xl 14 欄、xxl 16 欄皆 ≥ base 寬度，座標應原樣保留
    expect(layouts.xl).toEqual(base)
    expect(layouts.xxl).toEqual(base)
  })

  it('有 override 的斷點採用存檔座標', () => {
    const layouts = buildResponsiveLayouts(base, {
      xl: { a: { x: 0, y: 0, w: 7, h: 2 }, b: { x: 7, y: 0, w: 7, h: 2 }, c: { x: 0, y: 2, w: 14, h: 3 } },
    })
    expect(layouts.xl.find((l) => l.i === 'c')).toMatchObject({ x: 0, y: 2, w: 14, h: 3 })
  })

  it('override 寬度超出欄數時夾到欄數內', () => {
    const layouts = buildResponsiveLayouts(base, {
      md: { a: { x: 0, y: 0, w: 99, h: 2 } },
    })
    const a = layouts.md.find((l) => l.i === 'a')
    expect(a?.w).toBe(GRID_COLS_BY_BREAKPOINT.md)
    expect((a?.x ?? 0) + (a?.w ?? 0)).toBeLessThanOrEqual(GRID_COLS_BY_BREAKPOINT.md)
  })

  it('seed 自「最近的已存斷點」：xxl 在只有 xl override 時 seed 自 xl 而非 lg', () => {
    const xlPos = { a: { x: 1, y: 1, w: 3, h: 2 }, b: { x: 4, y: 1, w: 3, h: 2 }, c: { x: 7, y: 1, w: 3, h: 3 } }
    const layouts = buildResponsiveLayouts(base, { xl: xlPos })
    // xxl 無 override，最近已存為 xl(1920) 而非 lg(1200)，故座標來自 xl
    expect(layouts.xxl.find((l) => l.i === 'a')).toMatchObject({ x: 1, y: 1 })
  })

  it('手機斷點（xs/xxs）單欄堆疊：每個 widget x=0、滿欄寬', () => {
    const layouts = buildResponsiveLayouts(base)
    for (const item of layouts.xs) {
      expect(item.x).toBe(0)
      expect(item.w).toBe(GRID_COLS_BY_BREAKPOINT.xs)
    }
  })

  it('override 未涵蓋的 widget 沿用 base 座標（新增 widget 不遺失）', () => {
    const layouts = buildResponsiveLayouts(base, {
      xl: { a: { x: 0, y: 0, w: 7, h: 2 } }, // 只指定 a
    })
    expect(layouts.xl.find((l) => l.i === 'b')).toMatchObject({ x: 4, y: 0, w: 4 })
  })
})

describe('breakpointFromWidth', () => {
  it('1920 螢幕最大化（容器約 1680）落在 xl，而非 react-grid-layout 預設寬度的 lg', () => {
    expect(breakpointFromWidth(1680)).toBe('xl')
  })

  it('各斷點代表寬度對應正確', () => {
    expect(breakpointFromWidth(2400)).toBe('xxl')
    expect(breakpointFromWidth(1300)).toBe('lg')
    expect(breakpointFromWidth(1000)).toBe('md')
    expect(breakpointFromWidth(800)).toBe('sm')
    expect(breakpointFromWidth(500)).toBe('xs')
    expect(breakpointFromWidth(320)).toBe('xxs')
  })

  it('寬度恰等於門檻時落在下一個較窄斷點（與 react-grid-layout 的嚴格大於同語意）', () => {
    // 兩者必須一致，否則拖曳座標會存到與實際渲染不同的斷點
    expect(breakpointFromWidth(GRID_BREAKPOINTS.lg)).toBe('md')
    expect(breakpointFromWidth(GRID_BREAKPOINTS.lg + 1)).toBe('lg')
    expect(breakpointFromWidth(GRID_BREAKPOINTS.xl)).toBe('lg')
    expect(breakpointFromWidth(GRID_BREAKPOINTS.xxl)).toBe('xl')
  })

  it('寬度 0（尚未排版）退回最窄斷點，不丟例外', () => {
    expect(breakpointFromWidth(0)).toBe('xxs')
  })
})

describe('normalizeDashboardPref', () => {
  it('舊格式（v1 純陣列）轉為 v2，byBreakpoint 為空', () => {
    const arr: WidgetLayoutItem[] = [{ i: 'a', x: 0, y: 0, w: 4, h: 2 }]
    expect(normalizeDashboardPref(arr)).toEqual({ v: 2, widgets: arr, byBreakpoint: {} })
  })

  it('v2 物件原樣保留 widgets 與 byBreakpoint', () => {
    const pref = { v: 2, widgets: [{ i: 'a', x: 0, y: 0, w: 4, h: 2 }], byBreakpoint: { xl: { a: { x: 1, y: 1, w: 3, h: 2 } } } }
    expect(normalizeDashboardPref(pref)).toEqual(pref)
  })

  it('v2 缺 byBreakpoint 時補空物件', () => {
    const pref = { v: 2, widgets: [{ i: 'a', x: 0, y: 0, w: 4, h: 2 }] }
    expect(normalizeDashboardPref(pref).byBreakpoint).toEqual({})
  })

  it('不認得的值（null / 非預期物件）退回預設佈局', () => {
    expect(normalizeDashboardPref(null).widgets).toBe(DEFAULT_DASHBOARD_LAYOUT)
    expect(normalizeDashboardPref({ foo: 1 }).widgets).toBe(DEFAULT_DASHBOARD_LAYOUT)
  })

  it('byBreakpoint 損壞（primitive / 陣列）時退回空物件', () => {
    const widgets = [{ i: 'a', x: 0, y: 0, w: 4, h: 2 }]
    expect(normalizeDashboardPref({ v: 2, widgets, byBreakpoint: 'oops' }).byBreakpoint).toEqual({})
    expect(normalizeDashboardPref({ v: 2, widgets, byBreakpoint: [1, 2] }).byBreakpoint).toEqual({})
    expect(normalizeDashboardPref({ v: 2, widgets, byBreakpoint: null }).byBreakpoint).toEqual({})
  })
})
