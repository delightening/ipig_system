/**
 * R97-3：批號的 FEFO 排序（先到期先出）。
 *
 * 這支比較器決定倉管在下拉選單裡「先看到哪個批號」，等同系統對「該領哪一批」
 * 的建議。兩個容易出錯、且錯了不會有人發現的地方：
 *
 * 1. 用「有沒有效期字串」而非「效期能不能判讀」判定有效性——讀不出來的值
 *    （如 2026-02-31）會被排到最前面，變成建議優先領用一個效期不明的批號。
 * 2. 效期相同時沒有 tie-breaker——comparator 回 0，順序取決於 API 回傳順序，
 *    同樣的資料每次可能排出不同結果。
 *
 * 兩者皆為 CodeRabbit 於 PR #143 指出。
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import { compareByFefo, expiryStatus, type SortableBatch } from '../expiry'

/** 依真實流程組出一個待排序批號：效期字串經 `expiryStatus` 判讀。 */
function batch(name: string, expiry: string): SortableBatch {
  return { batch: name, expiry, ...expiryStatus(expiry) }
}

function sortedNames(items: SortableBatch[]): string[] {
  return [...items].sort(compareByFefo).map((b) => b.batch)
}

describe('compareByFefo', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-08-17T09:30:00'))
  })
  afterEach(() => vi.useRealTimers())

  it('先到期的排前面', () => {
    const items = [batch('B', '2026-12-01'), batch('A', '2026-09-01')]
    expect(sortedNames(items)).toEqual(['A', 'B'])
  })

  it('沒有效期的排在有效期之後', () => {
    const items = [batch('NOEXP', ''), batch('HASEXP', '2027-01-01')]
    expect(sortedNames(items)).toEqual(['HASEXP', 'NOEXP'])
  })

  it('讀不出來的效期不得排在可判讀者之前', () => {
    // 2026-02-31 有字串但不是真實日期；若用「字串非空」判定，
    // 它會被當成 2026 年初而排到最前面，等於建議先領一個效期不明的批號。
    const items = [batch('BAD', '2026-02-31'), batch('GOOD', '2029-01-01')]
    expect(sortedNames(items)).toEqual(['GOOD', 'BAD'])
  })

  it('效期相同時以批號決勝，順序穩定', () => {
    const items = [batch('B2', '2026-10-01'), batch('B1', '2026-10-01')]
    expect(sortedNames(items)).toEqual(['B1', 'B2'])
    // 輸入順序顛倒也要得到同樣結果——否則畫面會隨 API 回傳順序跳動。
    expect(sortedNames([...items].reverse())).toEqual(['B1', 'B2'])
  })

  it('兩者皆不可判讀時以批號決勝', () => {
    const items = [batch('Z', ''), batch('A', '2026-02-31')]
    expect(sortedNames(items)).toEqual(['A', 'Z'])
  })

  it('綜合排序：可判讀者依日期在前，不可判讀者依批號在後', () => {
    const items = [
      batch('none', ''),
      batch('late', '2027-06-01'),
      batch('malformed', '2026-04-31'),
      batch('early', '2026-09-01'),
    ]
    expect(sortedNames(items)).toEqual(['early', 'late', 'malformed', 'none'])
  })

  it('比較器自身對稱：a 對 b 與 b 對 a 正負相反', () => {
    // 不對稱的 comparator 在不同 JS 引擎的 sort 實作下會排出不同結果。
    const a = batch('A', '2026-09-01')
    const b = batch('B', '2027-01-01')
    expect(Math.sign(compareByFefo(a, b))).toBe(-Math.sign(compareByFefo(b, a)))

    const bad = batch('X', '2026-02-31')
    expect(Math.sign(compareByFefo(a, bad))).toBe(-Math.sign(compareByFefo(bad, a)))
  })
})
