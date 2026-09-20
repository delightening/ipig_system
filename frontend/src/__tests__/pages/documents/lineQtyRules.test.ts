import { describe, expect, it, vi } from 'vitest'

import { lineQtyError } from '@/pages/documents/lineQtyRules'
import type { DocType } from '@/lib/api'

// lineQtyError 的訊息走 i18n.t()：mock 成回傳 key（有內插參數時附上 JSON），
// 斷言比對 key 與行號參數，而不是任何一種語言的譯文。
vi.mock('@/lib/i18n', () => ({
  default: {
    t: (key: string, opts?: Record<string, unknown>) =>
      opts ? `${key}${JSON.stringify(opts)}` : key,
  },
}))

/** 「第 N 行：」前綴經 mock 後的樣子 */
const prefix = (lineNo: number) =>
  `erpDocs.documents.validation.linePrefix${JSON.stringify({ lineNo })}`

const MSG = {
  notNumber: 'erpDocs.documents.validation.qtyNotNumber',
  stocktakeNegative: 'erpDocs.documents.validation.stocktakeQtyNegative',
  adjustZero: 'erpDocs.documents.validation.adjustQtyZero',
  positive: 'validation.quantityPositive',
}

/** STK / ADJ 以外的單別，數量規則一律「必須大於 0」 */
const POSITIVE_ONLY: DocType[] = ['PO', 'GRN', 'PR', 'SO', 'TR']

describe('lineQtyError — STK 盤點', () => {
  it('0 合法——「系統有、現場沒有」＝全數短少，是盤點最該登記的結果，擋掉它等於無法登記', () => {
    expect(lineQtyError('STK', '0', 1)).toBeUndefined()
  })

  it('正數合法', () => {
    expect(lineQtyError('STK', '7', 1)).toBeUndefined()
  })

  it('小數合法——盤點可能是重量或體積', () => {
    expect(lineQtyError('STK', '0.5', 1)).toBeUndefined()
  })

  it('負數不合法——實盤數不可能是負的', () => {
    expect(lineQtyError('STK', '-1', 1)).toContain(MSG.stocktakeNegative)
  })

  it('負小數也不合法', () => {
    expect(lineQtyError('STK', '-0.5', 1)).toContain(MSG.stocktakeNegative)
  })
})

describe('lineQtyError — ADJ 調整', () => {
  it('正數合法（調增）', () => {
    expect(lineQtyError('ADJ', '3', 1)).toBeUndefined()
  })

  it('負數合法（調減）——payload 全程不轉正負號，擋掉負數等於調減開不出單', () => {
    expect(lineQtyError('ADJ', '-3', 1)).toBeUndefined()
  })

  it('0 不合法——沒有調整就不該開調整單', () => {
    expect(lineQtyError('ADJ', '0', 1)).toContain(MSG.adjustZero)
  })

  it('-0 視同 0 而擋下——JS 的 -0 === 0，不要讓它變成一張沒有效果的調整單', () => {
    expect(lineQtyError('ADJ', '-0', 1)).toContain(MSG.adjustZero)
  })
})

describe('lineQtyError — 其餘單別', () => {
  it.each(POSITIVE_ONLY)('%s 正數合法', (docType) => {
    expect(lineQtyError(docType, '1', 1)).toBeUndefined()
  })

  it.each(POSITIVE_ONLY)('%s 的 0 不合法', (docType) => {
    expect(lineQtyError(docType, '0', 1)).toContain(MSG.positive)
  })

  it.each(POSITIVE_ONLY)('%s 的負數不合法', (docType) => {
    expect(lineQtyError(docType, '-1', 1)).toContain(MSG.positive)
  })
})

describe('lineQtyError — 非數字', () => {
  // `<input type="number">` 在值不成數時回空字串，所以空字串是實際會發生的輸入，
  // 不是假想案例。三種單別都必須擋，否則 payload 會帶著 NaN 送到後端。
  it.each(['', '   ', 'abc', '-'] as const)(
    '所有單別都拒絕非數字輸入（%j）',
    (raw) => {
      for (const docType of ['STK', 'ADJ', ...POSITIVE_ONLY] as DocType[]) {
        expect(lineQtyError(docType, raw, 1)).toContain(MSG.notNumber)
      }
    },
  )

  it('undefined 也當非數字擋下', () => {
    expect(lineQtyError('STK', undefined, 1)).toContain(MSG.notNumber)
  })
})

describe('lineQtyError — 錯誤訊息帶行號', () => {
  it('行號原樣帶進訊息，使用者才知道是哪一行', () => {
    expect(lineQtyError('STK', '-1', 3)).toBe(`${prefix(3)}${MSG.stocktakeNegative}`)
    expect(lineQtyError('ADJ', '0', 12)).toBe(`${prefix(12)}${MSG.adjustZero}`)
    expect(lineQtyError('PO', '0', 5)).toBe(`${prefix(5)}${MSG.positive}`)
  })
})
