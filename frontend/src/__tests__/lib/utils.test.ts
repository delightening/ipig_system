import { describe, it, expect } from 'vitest'
import {
  cn,
  formatDate,
  formatDateTime,
  formatNumber,
  formatCurrency,
  formatFileSize,
  formatQuantity,
  formatUnitPrice,
  formatUom,
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
  it('maps known codes to Chinese', () => {
    expect(formatUom('EA')).toBe('個')
    expect(formatUom('BT')).toBe('瓶')
    expect(formatUom('KG')).toBe('kg')
    expect(formatUom('ML')).toBe('mL')
  })

  it('returns code as-is for unknown codes', () => {
    expect(formatUom('UNKNOWN')).toBe('UNKNOWN')
  })
})

describe('UOM_MAP', () => {
  it('contains expected entries', () => {
    expect(Object.keys(UOM_MAP).length).toBeGreaterThan(10)
    expect(UOM_MAP['EA']).toBeDefined()
    expect(UOM_MAP['pcs']).toBeDefined()
  })
})
