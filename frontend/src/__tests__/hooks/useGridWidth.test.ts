import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useGridWidth } from '@/pages/dashboard/hooks/useGridWidth'

const observers: MockResizeObserver[] = []

class MockResizeObserver {
  observe = vi.fn()
  unobserve = vi.fn()
  disconnect = vi.fn()
  constructor(public callback: ResizeObserverCallback) {
    observers.push(this)
  }
}

/** 建立帶指定 offsetWidth 的節點（jsdom 不做排版，offsetWidth 恆為 0） */
function makeNode(width: number): HTMLDivElement {
  const node = document.createElement('div')
  Object.defineProperty(node, 'offsetWidth', { configurable: true, value: width })
  return node
}

/** 模擬容器尺寸變動：更新節點的 offsetWidth（hook 的量測來源）後觸發 observer 回呼 */
function emitResize(observer: MockResizeObserver, node: HTMLDivElement, width: number) {
  Object.defineProperty(node, 'offsetWidth', { configurable: true, value: width })
  observer.callback(
    [{ target: node, contentRect: { width } } as unknown as ResizeObserverEntry],
    observer as unknown as ResizeObserver,
  )
}

describe('useGridWidth', () => {
  beforeEach(() => {
    observers.length = 0
    vi.stubGlobal('ResizeObserver', MockResizeObserver)
    // rAF 同步執行，讓 resize 回呼在 act 內即完成
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
      cb(0)
      return 1
    })
    vi.stubGlobal('cancelAnimationFrame', vi.fn())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('尚未掛上節點時 width 為 null（呼叫端此時只渲染量尺 div）', () => {
    const { result } = renderHook(() => useGridWidth())
    expect(result.current.width).toBeNull()
  })

  it('ref callback 掛上節點時立即量到真實寬度（非 WidthProvider 的 1280 預設值）', () => {
    const { result } = renderHook(() => useGridWidth())
    act(() => result.current.containerRef(makeNode(1680)))
    expect(result.current.width).toBe(1680)
  })

  it('量到 0（容器尚未排版 / 被隱藏）時維持未量測，等 observer 送出真實寬度', () => {
    const { result } = renderHook(() => useGridWidth())
    const node = makeNode(0)
    act(() => result.current.containerRef(node))
    expect(result.current.width).toBeNull()

    act(() => emitResize(observers[0], node, 1400))
    expect(result.current.width).toBe(1400)
  })

  it('容器尺寸變動時更新寬度', () => {
    const { result } = renderHook(() => useGridWidth())
    const node = makeNode(1680)
    act(() => result.current.containerRef(node))
    act(() => emitResize(observers[0], node, 900))
    expect(result.current.width).toBe(900)
  })

  it('resize 也以 border-box（offsetWidth）為準，不吃 content-box 的 contentRect', () => {
    const { result } = renderHook(() => useGridWidth())
    const node = makeNode(1680)
    act(() => result.current.containerRef(node))
    // 模擬容器帶 padding：contentRect（content-box）比 offsetWidth（border-box）小
    Object.defineProperty(node, 'offsetWidth', { configurable: true, value: 1680 })
    act(() =>
      observers[0].callback(
        [{ target: node, contentRect: { width: 1600 } } as unknown as ResizeObserverEntry],
        observers[0] as unknown as ResizeObserver,
      ),
    )
    expect(result.current.width).toBe(1680)
  })

  it('節點抽換（量尺 div → 網格根節點）時改觀察新節點，舊 observer 中止', () => {
    const { result } = renderHook(() => useGridWidth())
    act(() => result.current.containerRef(makeNode(1680)))
    act(() => result.current.containerRef(null))
    expect(observers[0].disconnect).toHaveBeenCalled()

    act(() => result.current.containerRef(makeNode(1400)))
    expect(observers).toHaveLength(2)
    expect(result.current.width).toBe(1400)
  })

  it('unmount 時中止 observer，不留下監聽', () => {
    const { result, unmount } = renderHook(() => useGridWidth())
    act(() => result.current.containerRef(makeNode(1680)))
    unmount()
    expect(observers[0].disconnect).toHaveBeenCalled()
  })
})
