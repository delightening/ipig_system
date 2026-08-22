/**
 * R97-3（純顯示階段）：批號效期狀態判斷。
 *
 * 這支函式只驅動視覺提示、不阻擋任何操作，但它的邊界很容易寫錯——
 * 尤其「今天到期」不該算成已過期（差一天就是把還能用的藥標成過期），
 * 以及時區造成的 off-by-one。以下逐項鎖住。
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import { expiryStatus, isExpiringSoon, EXPIRING_SOON_DAYS } from '../expiry'

/** 以本地時間建構一個固定的「現在」，避免測試隨執行時區飄移。 */
function freezeToday(localDate: string) {
  vi.useFakeTimers()
  vi.setSystemTime(new Date(`${localDate}T09:30:00`))
}

describe('expiryStatus', () => {
  beforeEach(() => freezeToday('2026-08-16'))
  afterEach(() => vi.useRealTimers())

  it('沒有效期時不標任何狀態', () => {
    expect(expiryStatus('')).toEqual({ expired: false, daysLeft: null })
  })

  it('無法解析的日期視同沒有效期，不誤標為過期', () => {
    expect(expiryStatus('not-a-date')).toEqual({ expired: false, daysLeft: null })
  })

  // ── 日曆上不存在的日期 ────────────────────────────────────────
  //
  // JS 會把「月份合法但日期超界」的輸入靜默捲到下個月，而不是回 Invalid Date：
  //   new Date('2026-02-31T00:00:00') → Mar 03 2026
  //   new Date('2026-04-31T00:00:00') → May 01 2026
  // 只用 Number.isNaN 判斷有效性會漏掉這類輸入，使用者會看到一個
  // 「看起來正常」但實際錯誤的天數——對藥品效期而言是最糟的失敗方式
  //（CodeRabbit 於 PR #143 指出，已實測確認捲動行為）。

  it('2 月 31 日不得被捲成 3 月 3 日', () => {
    expect(expiryStatus('2026-02-31')).toEqual({ expired: false, daysLeft: null })
  })

  it('小月的 31 日同樣視為無效', () => {
    expect(expiryStatus('2026-04-31')).toEqual({ expired: false, daysLeft: null })
  })

  it('月份超界視為無效', () => {
    expect(expiryStatus('2026-13-01')).toEqual({ expired: false, daysLeft: null })
  })

  it('閏年 2/29 有效、平年 2/29 無效', () => {
    // 2028 是閏年、2026 不是——證明比對的是真實日曆，不只是格式。
    expect(expiryStatus('2028-02-29').daysLeft).not.toBeNull()
    expect(expiryStatus('2026-02-29')).toEqual({ expired: false, daysLeft: null })
  })

  it('非 YYYY-MM-DD 的格式一律視為無效', () => {
    for (const bad of ['2026-8-16', '26-08-16', '2026/08/16', '2026-08-16T00:00:00']) {
      expect(expiryStatus(bad)).toEqual({ expired: false, daysLeft: null })
    }
  })

  it('今天到期不算過期（daysLeft = 0）', () => {
    // 差一天的判斷若寫錯，會把今天還能用的藥標成過期。
    expect(expiryStatus('2026-08-16')).toEqual({ expired: false, daysLeft: 0 })
  })

  it('明天到期不算過期', () => {
    expect(expiryStatus('2026-08-17')).toEqual({ expired: false, daysLeft: 1 })
  })

  it('昨天到期算過期，daysLeft 為 -1', () => {
    expect(expiryStatus('2026-08-15')).toEqual({ expired: true, daysLeft: -1 })
  })

  it('過期天數與實際天數一致（prod 實例：Amoxicillin 2026-04-23）', () => {
    const { expired, daysLeft } = expiryStatus('2026-04-23')
    expect(expired).toBe(true)
    expect(daysLeft).toBe(-115)
  })

  it('剛好落在「即將到期」門檻上仍算即將到期', () => {
    const status = expiryStatus('2026-09-15')
    expect(status.expired).toBe(false)
    expect(status.daysLeft).toBe(EXPIRING_SOON_DAYS)
    expect(isExpiringSoon(status)).toBe(true)
  })

  it('超過門檻一天就不算即將到期', () => {
    const status = expiryStatus('2026-09-16')
    expect(status.daysLeft).toBeGreaterThan(EXPIRING_SOON_DAYS)
    expect(isExpiringSoon(status)).toBe(false)
  })

  it('已過期者不重複標成「即將到期」', () => {
    // expired 與 expiring-soon 互斥，否則同一批號會同時出現紅字與黃字語意。
    expect(isExpiringSoon(expiryStatus('2026-04-23'))).toBe(false)
  })

  it('無效期者不算即將到期', () => {
    expect(isExpiringSoon(expiryStatus(''))).toBe(false)
  })

  it('當天稍晚執行不會讓「今天到期」翻成過期', () => {
    // 以 23:59 重跑一次：若拿 now 直接相減而未歸零時分，會算出負數而誤判。
    vi.setSystemTime(new Date('2026-08-16T23:59:00'))
    expect(expiryStatus('2026-08-16')).toEqual({ expired: false, daysLeft: 0 })
  })
})
