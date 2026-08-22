import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useScrollChaining } from '@/hooks/useScrollChaining'

interface ScrollerMetrics {
  scrollHeight: number
  clientHeight: number
  scrollTop?: number
  overflowY?: string
  overscrollBehavior?: string
}

/** jsdom 不做排版：捲動相關度量全為 0、scrollTop setter 是 no-op，這裡手動補上 */
function makeScroller(metrics: ScrollerMetrics): HTMLDivElement {
  const el = document.createElement('div')
  el.style.overflowY = metrics.overflowY ?? 'auto'
  // jsdom 的 cssstyle 不認得 overscroll-behavior，寫進 el.style 讀不回來（真實瀏覽器無此限制）。
  // 改記在 data 屬性上，由下方 stub 的 getComputedStyle 代為回報。
  if (metrics.overscrollBehavior) el.dataset.testOverscroll = metrics.overscrollBehavior
  Object.defineProperty(el, 'scrollHeight', { configurable: true, value: metrics.scrollHeight })
  Object.defineProperty(el, 'clientHeight', { configurable: true, value: metrics.clientHeight })
  Object.defineProperty(el, 'scrollTop', {
    configurable: true,
    writable: true,
    value: metrics.scrollTop ?? 0,
  })
  return el
}

/** 頁面捲軸（MainLayout 的 <main>）→ 網格根節點 → widget 內容區 → 內容 */
function buildTree(innerMetrics: ScrollerMetrics, outerMetrics: ScrollerMetrics) {
  const outer = makeScroller(outerMetrics)
  const root = document.createElement('div')
  const inner = makeScroller(innerMetrics)
  const content = document.createElement('span')
  inner.appendChild(content)
  root.appendChild(inner)
  outer.appendChild(root)
  document.body.appendChild(outer)
  return { outer, root, inner, content }
}

/**
 * 三層巢狀：頁面 → 中間捲動容器 → 最內層捲動容器 → 內容。
 * 用來驗證接力途中遇到「已到邊界」的中間層時的行為。
 */
function buildNestedTree(
  innerMetrics: ScrollerMetrics,
  middleMetrics: ScrollerMetrics,
  outerMetrics: ScrollerMetrics,
) {
  const outer = makeScroller(outerMetrics)
  const root = document.createElement('div')
  const middle = makeScroller(middleMetrics)
  const inner = makeScroller(innerMetrics)
  const content = document.createElement('span')
  inner.appendChild(content)
  middle.appendChild(inner)
  root.appendChild(middle)
  outer.appendChild(root)
  document.body.appendChild(outer)
  return { outer, root, middle, inner, content }
}

function wheel(target: Element, init: WheelEventInit): WheelEvent {
  const event = new WheelEvent('wheel', { bubbles: true, cancelable: true, ...init })
  target.dispatchEvent(event)
  return event
}

/**
 * 派送觸控事件。jsdom 沒有 TouchEvent 建構子，改用一般 Event 掛上 `touches`
 * ——hook 只讀 `touches.length` 與 `touches[0].clientY`。
 */
function touch(target: Element, type: string, clientY?: number, touchCount = 1): Event {
  const event = new Event(type, { bubbles: true, cancelable: true })
  const touches = clientY === undefined ? [] : Array.from({ length: touchCount }, () => ({ clientY }))
  Object.defineProperty(event, 'touches', { value: touches })
  target.dispatchEvent(event)
  return event
}

/**
 * 本檔登記過、待 afterEach 收掉的 cleanup。
 *
 * ref callback 把 cleanup 交還給呼叫端（React 19 的 ref cleanup 契約，正式使用時由 React
 * 在節點抽換 / unmount 呼叫）。測試是手動叫 ref，React 不會代勞，沒收乾淨的話監聽會跨測試
 * 累加——尤其掛在 document.body 上時，`body.innerHTML = ''` 清不掉 body 自身的監聽。
 */
const pendingCleanups: Array<() => void> = []

/** 把 hook 的 ref 掛到指定節點，並登記其 cleanup */
function attachTo(ref: ReturnType<typeof useScrollChaining>, node: HTMLElement) {
  let cleanup: (() => void) | undefined
  act(() => {
    cleanup = ref(node) ?? undefined
  })
  if (cleanup) pendingCleanups.push(cleanup)
  return () => cleanup?.()
}

/** 掛上 hook 的 ref 並回傳 DOM 樹 */
function setup(innerMetrics: ScrollerMetrics, outerMetrics: ScrollerMetrics) {
  const tree = buildTree(innerMetrics, outerMetrics)
  const { result } = renderHook(() => useScrollChaining())
  const cleanup = attachTo(result.current, tree.root)
  return { ...tree, ref: result.current, cleanup }
}

/** 巢狀三層版本的 setup */
function setupNested(
  innerMetrics: ScrollerMetrics,
  middleMetrics: ScrollerMetrics,
  outerMetrics: ScrollerMetrics,
) {
  const tree = buildNestedTree(innerMetrics, middleMetrics, outerMetrics)
  const { result } = renderHook(() => useScrollChaining())
  const cleanup = attachTo(result.current, tree.root)
  return { ...tree, ref: result.current, cleanup }
}

const PAGE: ScrollerMetrics = { scrollHeight: 2000, clientHeight: 500 }
/** 內層已捲到底（300 + 200 === 500） */
const INNER_AT_BOTTOM: ScrollerMetrics = { scrollHeight: 500, clientHeight: 200, scrollTop: 300 }
/** 內層還有空間可捲 */
const INNER_WITH_ROOM: ScrollerMetrics = { scrollHeight: 500, clientHeight: 200, scrollTop: 0 }

const realGetComputedStyle = window.getComputedStyle.bind(window)

/** 讓 data-test-overscroll 的值以 overscrollBehaviorY 回報，補上 jsdom 缺的屬性支援 */
function stubOverscrollSupport() {
  vi.stubGlobal('getComputedStyle', (el: Element, pseudo?: string | null) => {
    const style = realGetComputedStyle(el, pseudo)
    const override = el instanceof HTMLElement ? el.dataset.testOverscroll : undefined
    if (!override) return style
    return new Proxy(style, {
      get(target, prop, receiver) {
        if (prop === 'overscrollBehaviorY' || prop === 'overscrollBehavior') return override
        const value = Reflect.get(target, prop, receiver)
        return typeof value === 'function' ? value.bind(target) : value
      },
    })
  })
}

describe('useScrollChaining', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    stubOverscrollSupport()
  })

  afterEach(() => {
    // 監聽掛在測試自建的節點上，不收會跨測試累加（見 pendingCleanups 註解）
    while (pendingCleanups.length) pendingCleanups.pop()?.()
    vi.unstubAllGlobals()
    document.body.innerHTML = ''
  })

  it('內層捲到底後繼續往下滾：接力給外層捲動容器', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(true)
    expect(outer.scrollTop).toBe(120)
  })

  it('內層捲到頂後繼續往上滾：反向也接力', () => {
    const { content, outer } = setup(
      { scrollHeight: 500, clientHeight: 200, scrollTop: 0 },
      { ...PAGE, scrollTop: 400 },
    )
    const event = wheel(content, { deltaY: -120 })
    expect(event.defaultPrevented).toBe(true)
    expect(outer.scrollTop).toBe(280)
  })

  it('內層還捲得動時完全不插手，交回瀏覽器原生行為', () => {
    const { content, outer } = setup(INNER_WITH_ROOM, PAGE)
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('游標下沒有內層捲動容器時不插手（瀏覽器本來就會捲外層）', () => {
    const { root, outer } = setup(INNER_AT_BOTTOM, PAGE)
    const bare = document.createElement('div')
    root.appendChild(bare)
    const event = wheel(bare, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('外層也已到底時不攔截，避免吃掉可能的更外層捲動', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, {
      scrollHeight: 2000,
      clientHeight: 500,
      scrollTop: 1500,
    })
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(1500)
  })

  it('overflow-y 為 hidden 的祖先不算捲動容器', () => {
    const { content, outer } = setup(
      { scrollHeight: 500, clientHeight: 200, scrollTop: 300, overflowY: 'hidden' },
      PAGE,
    )
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('內層設 overscroll-behavior: contain 時不接力（下拉選單 / 對話框內容區）', () => {
    const { content, outer } = setup(
      { ...INNER_AT_BOTTOM, overscrollBehavior: 'contain' },
      PAGE,
    )
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('內層設 overscroll-behavior: none 時同樣不接力', () => {
    const { content, outer } = setup({ ...INNER_AT_BOTTOM, overscrollBehavior: 'none' }, PAGE)
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('中間層已到邊界且設 contain 時，鏈接就此打住，不跳過它洩漏到頁面', () => {
    // 原生語義是停在標了 contain 的容器；曾經因為「已到邊界」就被略過而繼續往外找
    const { content, middle, outer } = setupNested(
      INNER_AT_BOTTOM,
      { scrollHeight: 800, clientHeight: 300, scrollTop: 500, overscrollBehavior: 'contain' },
      PAGE,
    )

    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(false)
    expect(middle.scrollTop).toBe(500)
    expect(outer.scrollTop).toBe(0)
  })

  it('中間層已到邊界但未設 contain 時，照常往外接力', () => {
    const { content, middle, outer } = setupNested(
      INNER_AT_BOTTOM,
      { scrollHeight: 800, clientHeight: 300, scrollTop: 500 },
      PAGE,
    )

    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(true)
    expect(middle.scrollTop).toBe(500)
    expect(outer.scrollTop).toBe(120)
  })

  it('中間層還捲得動時優先餵中間層，不直接跳到頁面', () => {
    const { content, middle, outer } = setupNested(
      INNER_AT_BOTTOM,
      { scrollHeight: 800, clientHeight: 300, scrollTop: 0 },
      PAGE,
    )

    wheel(content, { deltaY: 120 })
    expect(middle.scrollTop).toBe(120)
    expect(outer.scrollTop).toBe(0)
  })

  it('內層明示 overscroll-behavior: auto 時照常接力', () => {
    const { content, outer } = setup({ ...INNER_AT_BOTTOM, overscrollBehavior: 'auto' }, PAGE)
    const event = wheel(content, { deltaY: 120 })
    expect(event.defaultPrevented).toBe(true)
    expect(outer.scrollTop).toBe(120)
  })

  it('ctrl + 滾輪（瀏覽器縮放）不接力', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
    const event = wheel(content, { deltaY: 120, ctrlKey: true })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('橫向為主的手勢不接力', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
    const event = wheel(content, { deltaX: 200, deltaY: 20 })
    expect(event.defaultPrevented).toBe(false)
    expect(outer.scrollTop).toBe(0)
  })

  it('deltaMode 為 line（Firefox）時換算成像素再接力', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
    wheel(content, { deltaY: 3, deltaMode: WheelEvent.DOM_DELTA_LINE })
    expect(outer.scrollTop).toBe(48)
  })

  it('deltaMode 為 page 時以接力對象的 scrollport 高度換算，不是視窗高', () => {
    const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
    // 外層 clientHeight（500）刻意與 jsdom 的 window.innerHeight（768）不同：
    // 用視窗高換算會捲成 768，一頁必須是「接收捲動的容器」自己的高度。
    expect(outer.clientHeight).not.toBe(window.innerHeight)
    wheel(content, { deltaY: 1, deltaMode: WheelEvent.DOM_DELTA_PAGE })
    expect(outer.scrollTop).toBe(500)
  })

  describe('觸控', () => {
    it('起手時內層已在邊界：接管手勢並把捲動量接力給外層', () => {
      const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
      touch(content, 'touchstart', 300)
      const move = touch(content, 'touchmove', 200) // 手指上滑 100px = 內容往下 100px
      expect(move.defaultPrevented).toBe(true)
      expect(outer.scrollTop).toBe(100)
    })

    it('停在頂端（scrollTop 0）往下捲：方向複判後交回原生，不攔也不代管', () => {
      // 幾乎每個捲動容器初始都在頂端邊界，這是最常見的路徑——必須走原生才有慣性可言
      const { content, outer, inner } = setup(INNER_WITH_ROOM, PAGE)
      touch(content, 'touchstart', 300)
      const move = touch(content, 'touchmove', 200) // 內容往下，內層還有空間
      expect(move.defaultPrevented).toBe(false)
      expect(inner.scrollTop).toBe(0) // 沒被我們動過，等原生處理
      expect(outer.scrollTop).toBe(0)
    })

    it('在底部但第一個 move 是往回捲：同樣交回原生', () => {
      const { content, outer, inner } = setup(INNER_AT_BOTTOM, PAGE)
      touch(content, 'touchstart', 200)
      const move = touch(content, 'touchmove', 250) // 內容往上，內層還有空間
      expect(move.defaultPrevented).toBe(false)
      expect(inner.scrollTop).toBe(300)
      expect(outer.scrollTop).toBe(0)
    })

    it('已接管後方向反轉：不交還原生（已攔過就啟動不了），改由我們餵回內層', () => {
      const { content, outer, inner } = setup(INNER_AT_BOTTOM, PAGE)
      touch(content, 'touchstart', 300)
      touch(content, 'touchmove', 200) // 往下 100 → 接管，接力給外層
      expect(outer.scrollTop).toBe(100)

      const back = touch(content, 'touchmove', 250) // 反轉往上 50
      expect(back.defaultPrevented).toBe(true)
      // 手指底下的內層往上還有空間 → 優先餵內層，外層維持不動
      expect(inner.scrollTop).toBe(250)
      expect(outer.scrollTop).toBe(100)
    })

    it('內層設 overscroll-behavior: contain 時不接管觸控手勢', () => {
      const { content, outer } = setup({ ...INNER_AT_BOTTOM, overscrollBehavior: 'contain' }, PAGE)
      touch(content, 'touchstart', 300)
      const move = touch(content, 'touchmove', 200)
      expect(move.defaultPrevented).toBe(false)
      expect(outer.scrollTop).toBe(0)
    })

    it('多指手勢（縮放）不接管', () => {
      const { content, outer } = setup(INNER_AT_BOTTOM, PAGE)
      touch(content, 'touchstart', 300, 2)
      const move = touch(content, 'touchmove', 200, 2)
      expect(move.defaultPrevented).toBe(false)
      expect(outer.scrollTop).toBe(0)
    })

    it('touchend 後解除接管，下一個手勢重新判定', () => {
      const { content, outer, inner } = setup(INNER_AT_BOTTOM, PAGE)
      touch(content, 'touchstart', 300)
      touch(content, 'touchmove', 200)
      touch(content, 'touchend')
      expect(outer.scrollTop).toBe(100)

      // 讓內層離開底部：下一個手勢往下捲時內層還有空間，應交回原生
      inner.scrollTop = 100
      touch(content, 'touchstart', 300)
      const move = touch(content, 'touchmove', 200)
      expect(move.defaultPrevented).toBe(false)
      expect(outer.scrollTop).toBe(100) // 停在上一段接力的位置，未再增加
    })
  })

  it('執行 cleanup 後不留下監聽（React 於節點抽換 / unmount 時呼叫）', () => {
    const { content, outer, cleanup } = setup(INNER_AT_BOTTOM, PAGE)
    act(() => cleanup())
    wheel(content, { deltaY: 120 })
    expect(outer.scrollTop).toBe(0)
  })

  it('cleanup 後重新掛上同一節點可再次運作', () => {
    const { root, content, outer, ref, cleanup } = setup(INNER_AT_BOTTOM, PAGE)
    act(() => cleanup())
    wheel(content, { deltaY: 120 })
    expect(outer.scrollTop).toBe(0)

    act(() => {
      ref(root)
    })
    wheel(content, { deltaY: 120 })
    expect(outer.scrollTop).toBe(120)
  })

  it('ref callback 掛上節點時回傳 cleanup、收到 null 時不回傳', () => {
    const { result } = renderHook(() => useScrollChaining())
    const node = document.createElement('div')
    expect(typeof result.current(node)).toBe('function')
    expect(result.current(null)).toBeUndefined()
  })
})
