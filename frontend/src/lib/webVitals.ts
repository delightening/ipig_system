/**
 * Web Vitals 監控
 *
 * 收集 Core Web Vitals 指標（CLS、INP、LCP、FCP、TTFB），
 * 開發環境輸出至 console，正式環境以 sendBeacon 送至後端。
 */

import { onCLS, onFCP, onINP, onLCP, onTTFB } from 'web-vitals'
import type { Metric } from 'web-vitals'

function sendToAnalytics(metric: Metric): void {
  if (import.meta.env.DEV) {
    console.debug('[Web Vitals]', metric.name, metric.value, metric)
  } else {
    const body = JSON.stringify(metric)
    navigator.sendBeacon('/api/metrics/vitals', body)
  }
}

export function reportWebVitals(): void {
  onCLS(sendToAnalytics)
  onINP(sendToAnalytics)
  onLCP(sendToAnalytics)
  onFCP(sendToAnalytics)
  onTTFB(sendToAnalytics)
}
