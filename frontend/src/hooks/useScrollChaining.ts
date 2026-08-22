/**
 * 捲動接力（scroll chaining）Hook
 *
 * 解決什麼：頁面內的卡片 / 面板常是 `overflow-auto` 的內層捲動容器，捲到底之後頁面不會
 * 跟著繼續捲，使用者必須停手或把游標移出該容器。儀表板 widget 幾乎鋪滿視窗，最早就是在
 * 那裡被發現（#1084），實測平板上也會卡。
 *
 * 兩種輸入的成因與對策不同：
 *
 * - **滑鼠滾輪**：Chrome 的 latching / Firefox 的 scroll transaction——一次連續手勢一旦鎖定
 *   內層容器，到底後剩餘捲動量不會傳給外層。對策是在頁面捲動容器掛 non-passive `wheel`
 *   監聽（委派），內層到邊界時 `preventDefault` 並手動把剩餘量交給外層。未到邊界完全不插手。
 *
 * - **觸控**：無法中途接管——原生捲動一旦因第一個未被攔截的 `touchmove` 啟動，之後再呼叫
 *   `preventDefault()` 會被忽略。因此改在 `touchstart` 就判定：**只有「起手時內層已在邊界」
 *   的手勢才建立接管**，再於第一個 `touchmove` 拿到真實方向後複判，內層在該方向還捲得動就
 *   立刻交回原生。其餘手勢（絕大多數）完全走原生，保留 compositor thread 捲動與原生慣性——
 *   這也是為什麼不無條件掛 non-passive `touchmove`：那會讓整個 app 失去行動裝置的捲動優化。
 *
 * 掛一處即涵蓋其下所有內層捲動容器，不必逐一改每個元件。不想被接力的容器（下拉選單、
 * 對話框內容區等）設 CSS `overscroll-behavior: contain`（或 `none`）即可——本機制尊重這個
 * 標準屬性，語義與瀏覽器原生的 chaining 控制一致。Radix 的對話框 / 選單 portal 到 `body`、
 * 側邊欄是頁面容器的 sibling，本來就在委派範圍外。
 *
 * 事件接線與手勢狀態機在 `lib/scrollChainingController`，純 DOM 判定在 `lib/scrollChaining`；
 * 本檔只負責把它接上 React 的生命週期。
 */
import { useCallback, type RefCallback } from 'react'
import { attachScrollChaining } from '@/lib/scrollChainingController'

/**
 * 讓內層捲動容器捲到邊界後，把剩餘捲動量接力給外層容器。
 *
 * 掛在頁面捲動容器上即可，其下所有內層捲動容器都適用（事件委派）。
 *
 * 回傳的 ref callback 直接把 cleanup 交還給 React（React 19 起支援）：節點抽換與 unmount
 * 時由 React 負責呼叫，不需要自行以 `useRef` + `useEffect` 記錄與收尾。
 *
 * @returns 掛到容器根節點上的 ref callback
 */
export function useScrollChaining(): RefCallback<HTMLElement> {
  return useCallback((node: HTMLElement | null) => {
    if (!node) return
    return attachScrollChaining(node)
  }, [])
}
