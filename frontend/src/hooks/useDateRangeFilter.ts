import { useState, useCallback } from 'react'

export interface DateRangeFilterConfig {
  /** 起始日期初始值（YYYY-MM-DD），可傳函數做 lazy 初始化 */
  initialFrom?: string | (() => string)
  /** 結束日期初始值（YYYY-MM-DD），可傳函數做 lazy 初始化 */
  initialTo?: string | (() => string)
}

/**
 * 日期區間篩選狀態管理。
 * 適用於 HrLeavePage、AdminAuditPage、AuditLogsPage、AccountingReportPage 等有日期範圍篩選的頁面。
 *
 * @param config 可選初始值
 * @returns from、to、setFrom、setTo、setRange、reset
 */
export function useDateRangeFilter(config: DateRangeFilterConfig = {}) {
  const { initialFrom = '', initialTo = '' } = config

  const [from, setFrom] = useState(() =>
    typeof initialFrom === 'function' ? initialFrom() : initialFrom
  )
  const [to, setTo] = useState(() =>
    typeof initialTo === 'function' ? initialTo() : initialTo
  )

  const setRange = useCallback((f: string, t: string) => {
    setFrom(f)
    setTo(t)
  }, [])

  const reset = useCallback(() => {
    const fromVal = typeof initialFrom === 'function' ? initialFrom() : initialFrom
    const toVal = typeof initialTo === 'function' ? initialTo() : initialTo
    setFrom(fromVal)
    setTo(toVal)
  }, [initialFrom, initialTo])

  return {
    from,
    to,
    setFrom,
    setTo,
    setRange,
    reset,
  }
}
