/**
 * 儀表板網格容器寬度量測 Hook
 *
 * 為何不用 react-grid-layout 官方的 WidthProvider：
 * 它的寬度 state 固定以 1280px 起跑，掛載後才由 ResizeObserver（+ requestAnimationFrame）
 * 修正成真實寬度。在容器寬度 > 1280 的螢幕（1920 最大化 → 約 1680）第一幀會先以 lg（12 欄）
 * 畫一次，下一幀才跳到 xl（14 欄）→ 使用者看到「widget 先排錯位置再瞬間重排」。
 * `measureBeforeMount` 也救不了：它只是延後到 mount 後才渲染，屆時寬度仍是 1280。
 *
 * 這裡改由呼叫端自己拿寬度：containerRef 是 ref callback，React 在 commit 階段（瀏覽器 paint 前）
 * 就會呼叫它，於其中同步量到真實寬度並 setState，React 會在同一幀 paint 前重新渲染，
 * 因此「第一次畫出來的網格」就已經是正確斷點。
 *
 * width === null 代表尚未量測，呼叫端此時只渲染一個掛著 containerRef 的空 div 當量尺。
 *
 * 為何也不用 v2 主入口的 `useContainerWidth`（Reuse ladder：框架原生優先，此處刻意不採）：
 * 該 hook 在 `useEffect`（passive effect，paint 之後）才呼叫 measureWidth，第一幀一律用
 * `initialWidth`（預設同樣是 1280）。搭配 `measureBeforeMount` 雖能避免「畫錯位置」，但代價是
 * 多一個「只有空白量尺、還沒有網格」的已繪製影格（骨架 → 空白 → widget）。它也沒有寬度 0
 * （容器被隱藏）的保護，會直接以 0 寬度渲染。本 hook 改在 commit 階段量測，零多餘影格。
 */
import { useCallback, useEffect, useRef, useState } from 'react'

/** useGridWidth 的回傳值 */
export interface GridWidth {
  /** 容器寬度 px；null = 尚未量測 */
  width: number | null
  /** 掛到量測目標元素上的 ref callback（量尺 div 與網格根節點共用同一個） */
  containerRef: (node: HTMLDivElement | null) => void
}

/**
 * 量測容器寬度供 react-grid-layout 直接使用。
 *
 * @returns `width`（null = 尚未量測）與要掛到容器上的 `containerRef`
 */
export function useGridWidth(): GridWidth {
  const [width, setWidth] = useState<number | null>(null)
  const observerRef = useRef<ResizeObserver | null>(null)
  const rafRef = useRef<number | null>(null)

  const cleanup = useCallback(() => {
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current)
      rafRef.current = null
    }
    observerRef.current?.disconnect()
    observerRef.current = null
  }, [])

  const containerRef = useCallback(
    (node: HTMLDivElement | null) => {
      cleanup()
      if (!node) return
      // paint 前的同步量測：決定首幀斷點。
      // 量到 0 代表容器還沒排版（被隱藏 / 尚未進入版面），維持未量測狀態等 observer 補送，
      // 不要以 0 寬度畫出擠成一團的版面。
      if (node.offsetWidth > 0) setWidth(node.offsetWidth)
      // 後續變動（視窗縮放、側邊欄開合）走 ResizeObserver；rAF 節流避免 observer loop 警告。
      // 量測基準一律用 offsetWidth（border-box），與上面的首次量測同一來源——
      // entry.contentRect 是 content-box，容器日後若加上 padding / border 會在掛載後
      // 立刻從 border-box 跳成 content-box，造成一次無謂的寬度跳動。
      const observer = new ResizeObserver((entries) => {
        const entry = entries[0]
        if (!entry) return
        const nextWidth =
          entry.target instanceof HTMLElement ? entry.target.offsetWidth : entry.contentRect.width
        if (nextWidth <= 0) return
        if (rafRef.current !== null) cancelAnimationFrame(rafRef.current)
        rafRef.current = requestAnimationFrame(() => {
          rafRef.current = null
          setWidth(nextWidth)
        })
      })
      observer.observe(node)
      observerRef.current = observer
    },
    [cleanup],
  )

  useEffect(() => cleanup, [cleanup])

  return { width, containerRef }
}
