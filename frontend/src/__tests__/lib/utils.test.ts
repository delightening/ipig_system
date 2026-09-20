import { describe, it, expect, afterEach } from 'vitest'
import i18n from '@/lib/i18n'
import {
  cn,
  formatDate,
  formatDateTime,
  formatTime,
  formatTimeShort,
  formatNumber,
  formatCurrency,
  formatFileSize,
  formatQuantity,
  formatUnitPrice,
  formatUom,
  INVALID_DATE_FALLBACK,
  UOM_MAP,
} from '@/lib/utils'

describe('cn (classname merge)', () => {
  it('merges class names', () => {
    expect(cn('foo', 'bar')).toBe('foo bar')
  })

  it('handles conditional classes', () => {
    const cond = false
    expect(cn('base', cond && 'hidden', 'visible')).toBe('base visible')
  })

  it('resolves Tailwind conflicts', () => {
    // twMerge should resolve p-4 vs p-2 => p-2 wins
    expect(cn('p-4', 'p-2')).toBe('p-2')
  })

  it('handles empty input', () => {
    expect(cn()).toBe('')
  })
})

describe('formatDate', () => {
  it('formats a date string to zh-TW locale', () => {
    const result = formatDate('2024-03-15')
    expect(result).toMatch(/2024/)
    expect(result).toMatch(/03/)
    expect(result).toMatch(/15/)
  })

  it('formats a Date object', () => {
    const result = formatDate(new Date(2024, 0, 1)) // Jan 1, 2024
    expect(result).toMatch(/2024/)
    expect(result).toMatch(/01/)
  })
})

describe('formatDateTime', () => {
  it('includes date and time components', () => {
    const result = formatDateTime('2024-03-15T14:30:00')
    expect(result).toMatch(/2024/)
    expect(result).toMatch(/03/)
    expect(result).toMatch(/15/)
    // zh-TW locale may use 12h format (下午02:30) or 24h format (14:30)
    expect(result).toMatch(/30/)
  })
})

/**
 * R90-1：壞資料不得把字面上的 `Invalid Date` 顯示給使用者。
 *
 * 這幾個 case 在修好之前全部會失敗——`new Date('壞資料')` 不 throw，
 * `toLocaleDateString()` 也不 throw，兩者串起來就是把 `Invalid Date` 印在畫面上。
 * 所以這裡確實測得到本次改動，不是恆真的裝飾。
 */
describe('date formatters reject invalid input (R90-1)', () => {
  const invalidInputs: Array<[string, string | Date | null | undefined]> = [
    ['unparsable string', 'not-a-date'],
    ['empty string', ''],
    ['null', null],
    ['undefined', undefined],
    ['Invalid Date object', new Date('nope')],
  ]

  for (const [label, input] of invalidInputs) {
    it(`formatDate returns the fallback for ${label}`, () => {
      expect(formatDate(input)).toBe(INVALID_DATE_FALLBACK)
    })

    it(`formatDateTime returns the fallback for ${label}`, () => {
      expect(formatDateTime(input)).toBe(INVALID_DATE_FALLBACK)
    })

    it(`formatTime returns the fallback for ${label}`, () => {
      expect(formatTime(input)).toBe(INVALID_DATE_FALLBACK)
    })

    it(`formatTimeShort returns the fallback for ${label}`, () => {
      expect(formatTimeShort(input)).toBe(INVALID_DATE_FALLBACK)
    })
  }

  it('never leaks the literal string "Invalid Date"', () => {
    for (const [, input] of invalidInputs) {
      expect(formatDate(input)).not.toMatch(/Invalid Date/)
      expect(formatDateTime(input)).not.toMatch(/Invalid Date/)
      expect(formatTime(input)).not.toMatch(/Invalid Date/)
      expect(formatTimeShort(input)).not.toMatch(/Invalid Date/)
    }
  })

  it('honours a caller-supplied fallback', () => {
    expect(formatDate('not-a-date', undefined, '未設定')).toBe('未設定')
    expect(formatDateTime('not-a-date', '未設定')).toBe('未設定')
    expect(formatTimeShort('not-a-date', '未設定')).toBe('未設定')
  })

  it('still formats valid input after the guard', () => {
    expect(formatDate('2024-03-15')).toMatch(/2024/)
    expect(formatDateTime('2024-03-15T14:30:00')).toMatch(/2024/)
    expect(formatTime('2024-03-15T14:30:00')).toMatch(/\d{2}:\d{2}/)
    expect(formatTimeShort('2024-03-15T14:30:00')).toMatch(/\d{2}:\d{2}/)
  })
})

describe('formatNumber', () => {
  it('formats with default 2 decimal places', () => {
    const result = formatNumber(1234.5)
    expect(result).toContain('1,234.50')
  })

  it('formats string input', () => {
    const result = formatNumber('99.1', 1)
    expect(result).toContain('99.1')
  })

  it('formats with custom decimal places', () => {
    const result = formatNumber(42, 0)
    expect(result).toBe('42')
  })
})

describe('formatCurrency', () => {
  it('formats as TWD currency', () => {
    const result = formatCurrency(1500)
    expect(result).toContain('1,500')
  })

  it('handles string input', () => {
    const result = formatCurrency('2500')
    expect(result).toContain('2,500')
  })
})

describe('formatFileSize', () => {
  it('returns 0 Bytes for 0', () => {
    expect(formatFileSize(0)).toBe('0 Bytes')
  })

  it('formats bytes', () => {
    expect(formatFileSize(500)).toBe('500 Bytes')
  })

  it('formats KB', () => {
    expect(formatFileSize(1024)).toBe('1 KB')
  })

  it('formats MB', () => {
    expect(formatFileSize(1048576)).toBe('1 MB')
  })

  it('formats GB', () => {
    expect(formatFileSize(1073741824)).toBe('1 GB')
  })

  it('rounds KB to integer (no decimals)', () => {
    expect(formatFileSize(1536)).toBe('2 KB')
    expect(formatFileSize(1280)).toBe('1 KB')
  })

  it('switches to MB when KB >= 1000', () => {
    expect(formatFileSize(1024 * 999)).toBe('999 KB')
    expect(formatFileSize(1024 * 1000)).toBe('1 MB')
    expect(formatFileSize(1024 * 1500)).toBe('1.5 MB')
  })
})

describe('formatQuantity', () => {
  it('rounds to integer', () => {
    expect(formatQuantity(3.7)).toBe('4')
  })

  it('handles string input', () => {
    expect(formatQuantity('10.2')).toBe('10')
  })

  it('returns empty for NaN', () => {
    expect(formatQuantity('abc')).toBe('')
  })

  it('handles zero', () => {
    expect(formatQuantity(0)).toBe('0')
  })
})

describe('formatUnitPrice', () => {
  it('returns integer for whole numbers', () => {
    expect(formatUnitPrice(100)).toBe('100')
  })

  it('returns 2 decimals for fractional', () => {
    expect(formatUnitPrice(99.5)).toBe('99.50')
  })

  it('handles string input', () => {
    expect(formatUnitPrice('42.123')).toBe('42.12')
  })

  it('returns empty for NaN', () => {
    expect(formatUnitPrice('abc')).toBe('')
  })
})

describe('formatUom', () => {
  // formatUom 走 i18n（`uom.<code>`），期望值依語系而定；測完把語系還原，避免影響同檔其他測試
  const originalLanguage = i18n.language
  afterEach(async () => {
    await i18n.changeLanguage(originalLanguage)
  })

  it('maps known codes to Chinese under zh-TW', async () => {
    await i18n.changeLanguage('zh-TW')
    expect(formatUom('EA')).toBe('個')
    expect(formatUom('BT')).toBe('瓶')
    expect(formatUom('KG')).toBe('kg')
    expect(formatUom('ML')).toBe('mL')
  })

  it('maps known codes to English under en', async () => {
    await i18n.changeLanguage('en')
    expect(formatUom('EA')).toBe('pcs')
    expect(formatUom('BT')).toBe('bottle')
    expect(formatUom('KG')).toBe('kg')
    expect(formatUom('ML')).toBe('mL')
  })

  it('returns code as-is for unknown codes in every language', async () => {
    for (const lng of ['zh-TW', 'en']) {
      await i18n.changeLanguage(lng)
      expect(formatUom('UNKNOWN')).toBe('UNKNOWN')
    }
  })

  it('returns empty input as-is', () => {
    expect(formatUom('')).toBe('')
  })
})

describe('UOM_MAP', () => {
  it('contains expected entries', () => {
    expect(Object.keys(UOM_MAP).length).toBeGreaterThan(10)
    expect(UOM_MAP['EA']).toBeDefined()
    expect(UOM_MAP['pcs']).toBeDefined()
  })

  // UOM_MAP 是「固定 zh-TW 名稱」，供舊資料（曾把中文單位名直接存進 base_uom）反查代碼；
  // 它必須和 locales 的 uom.<code>（zh-TW）保持一致，否則反查與顯示會各說各話。
  it('stays in sync with the zh-TW uom.<code> labels', () => {
    for (const [code, name] of Object.entries(UOM_MAP)) {
      expect(i18n.getResource('zh-TW', 'translation', `uom.${code}`)).toBe(name)
    }
  })

  it('has an English uom.<code> label for every code', () => {
    for (const code of Object.keys(UOM_MAP)) {
      expect(i18n.getResource('en', 'translation', `uom.${code}`)).toEqual(expect.any(String))
    }
  })
})
