/**
 * Web Vitals 監控
 *
 * 收集 Core Web Vitals 指標（CLS、INP、LCP、FCP、TTFB）。
 * 開發環境輸出至 console；正式環境佇列後於頁面隱藏時「一次」以 sendBeacon 送至後端，
 * 避免每個指標各打一次 /api/metrics/vitals（效能 P2：每頁多呼叫 → 至多 1 次）。
 */

import { onCLS, onFCP, onINP, onLCP, onTTFB } from 'web-vitals'
import type { Metric } from 'web-vitals'

// 以 metric.name 為鍵去重：同一指標若多次回報，保留最後一筆，最終整批送出。
const pending = new Map<string, Metric>()

function enqueue(metric: Metric): void {
  if (import.meta.env.DEV) {
    console.debug('[Web Vitals]', metric.name, metric.value, metric)
    return
  }
  pending.set(metric.name, metric)
}

// flush 後清空 pending；空佇列直接 no-op。故同一次隱藏事件中 visibilitychange 與
// pagehide 皆觸發時不會重送；而使用者切回頁面（web-vitals v5 會在後續可見性週期
// 再回報 INP/CLS）產生的新指標仍能於下次隱藏時送出（不再被永久 latch 攔截）。
function flush(): void {
  if (pending.size === 0) return
  const body = JSON.stringify([...pending.values()])
  pending.clear()
  navigator.sendBeacon('/api/metrics/vitals', new Blob([body], { type: 'application/json' }))
}

export function reportWebVitals(): void {
  onCLS(enqueue)
  onINP(enqueue)
  onLCP(enqueue)
  onFCP(enqueue)
  onTTFB(enqueue)
  // 頁面隱藏/卸載時整批送出（sendBeacon 的設計用途）。visibilitychange→hidden 為主，
  // pagehide 補 Safari/iOS 不一定觸發 visibilitychange 的情況。
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') flush()
  })
  window.addEventListener('pagehide', flush)
}
