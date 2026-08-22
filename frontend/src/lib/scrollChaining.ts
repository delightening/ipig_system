/**
 * 捲動接力（scroll chaining）的純 DOM 判定工具。
 *
 * 與 React 無關，供 `hooks/useScrollChaining` 使用；抽出來是為了讓 hook 只留事件處理邏輯。
 * 背景與整體設計見該 hook 的檔頭註解。
 */

/** deltaMode = DOM_DELTA_LINE 時一「行」估算的像素數（近似瀏覽器預設行高） */
const LINE_HEIGHT_PX = 16
/** scrollTop / scrollHeight 在頁面縮放與高 DPI 下可能是小數，邊界判定留 1px 容差 */
export const EDGE_TOLERANCE_PX = 1

/**
 * 把 wheel 的 deltaY 換算成像素（Firefox 會送 line / page 模式，不是像素）。
 *
 * `pageHeight` 要傳「實際接收這次捲動的容器」的 clientHeight：DOM_DELTA_PAGE 的一單位是
 * 一個 scrollport 的高度，而接力對象未必是視窗，用 window.innerHeight 換算會捲錯距離。
 */
export function wheelDeltaToPixels(event: WheelEvent, pageHeight: number): number {
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) return event.deltaY * LINE_HEIGHT_PX
  if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) return event.deltaY * pageHeight
  return event.deltaY
}

/** 此元素目前是否為垂直捲動容器（有溢出內容 + overflow-y 允許捲動） */
export function isVerticalScroller(el: HTMLElement): boolean {
  if (el.scrollHeight - el.clientHeight <= EDGE_TOLERANCE_PX) return false
  const { overflowY } = getComputedStyle(el)
  return overflowY === 'auto' || overflowY === 'scroll'
}

/**
 * 此元素是否明示「不要把捲動傳出去」。
 *
 * 尊重標準的 `overscroll-behavior`：瀏覽器原生就用它決定 chaining，我們接管時沿用同一份
 * 宣告，作者不需要為這套機制學一組新約定。優先讀 `overscroll-behavior-y`，讀不到再退回簡寫。
 */
export function blocksChaining(el: HTMLElement): boolean {
  const style = getComputedStyle(el)
  const value = style.overscrollBehaviorY || style.overscrollBehavior || 'auto'
  return value.split(' ').some((v) => v === 'contain' || v === 'none')
}

/** 指定方向上是否還捲得動（deltaY > 0 = 往下） */
export function canScrollFurther(el: HTMLElement, deltaY: number): boolean {
  return deltaY < 0
    ? el.scrollTop > EDGE_TOLERANCE_PX
    : el.scrollTop + el.clientHeight < el.scrollHeight - EDGE_TOLERANCE_PX
}

/** 此元素是否已抵達任一端（上緣或下緣） */
export function isAtAnyEdge(el: HTMLElement): boolean {
  return !canScrollFurther(el, 1) || !canScrollFurther(el, -1)
}

/** 從 start 往上找第一個垂直捲動容器，走到 stopAt（不含）為止 */
export function findScrollerWithin(start: Element | null, stopAt: Element): HTMLElement | null {
  for (let node = start; node && node !== stopAt; node = node.parentElement) {
    if (node instanceof HTMLElement && isVerticalScroller(node)) return node
  }
  return null
}

/**
 * 從 start 往上找第一個「在該方向還捲得動」的垂直捲動容器，一路找到文件根節點。
 *
 * 途中若遇到「已到邊界且以 `overscroll-behavior` 明示不外傳」的捲動容器，鏈接就此打住
 * （回 null）——原生語義是停在那個容器，不是跳過它繼續往外找。少了這道判斷，巢狀捲軸中
 * 標了 contain 的中間層會被靜默略過，捲動照樣洩漏到頁面。
 */
export function findScrollerWithRoom(start: Element | null, deltaY: number): HTMLElement | null {
  for (let node = start; node; node = node.parentElement) {
    if (!(node instanceof HTMLElement) || !isVerticalScroller(node)) continue
    if (canScrollFurther(node, deltaY)) return node
    if (blocksChaining(node)) return null
  }
  return null
}

/**
 * 把捲動量餵給該吃的容器：內層還捲得動就給內層，到邊界才接力給外層。
 *
 * @returns 是否真的有容器吃下這段捲動（false = 上下都到底，呼叫端應交回瀏覽器）
 */
export function applyScrollWithChaining(inner: HTMLElement, deltaY: number): boolean {
  const target = canScrollFurther(inner, deltaY)
    ? inner
    : blocksChaining(inner)
      ? null
      : findScrollerWithRoom(inner.parentElement, deltaY)
  if (!target) return false
  target.scrollTop += deltaY
  return true
}
