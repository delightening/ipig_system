import { describe, expect, it } from 'vitest'
import { formatMeters, penAreaM2, validatePenSize } from '@/pages/admin/utils/penSize'

describe('validatePenSize', () => {
  it('兩格皆空是合法的（表示不填尺寸）', () => {
    expect(validatePenSize('', '')).toBeNull()
    expect(validatePenSize('  ', ' ')).toBeNull()
  })

  it('只填一邊要擋', () => {
    expect(validatePenSize('2.4', '')).toBe('bothOrNone')
    expect(validatePenSize('', '3')).toBe('bothOrNone')
  })

  it('非正數、格式不合、超過兩位小數都算無效', () => {
    expect(validatePenSize('0', '3')).toBe('invalid')
    expect(validatePenSize('-1', '3')).toBe('invalid')
    expect(validatePenSize('abc', '3')).toBe('invalid')
    expect(validatePenSize('1e2', '3')).toBe('invalid')
    expect(validatePenSize('2.401', '3')).toBe('invalid')
    expect(validatePenSize('2,4', '3')).toBe('invalid')
  })

  it('超過 100 公尺要擋，剛好 100 可以', () => {
    expect(validatePenSize('100.01', '3')).toBe('tooLarge')
    expect(validatePenSize('100', '3')).toBeNull()
  })

  it('一到兩位小數都接受', () => {
    expect(validatePenSize('2.4', '3.00')).toBeNull()
    expect(validatePenSize('0.01', '0.01')).toBeNull()
  })
})

describe('penAreaM2', () => {
  it('長 × 寬', () => {
    expect(penAreaM2('2.40', '3.00')).toBe(7.2)
  })

  it('用整數換算，不受浮點誤差影響', () => {
    // 1.1 * 1.1 在浮點數是 1.2100000000000002
    expect(penAreaM2('1.1', '1.1')).toBe(1.21)
  })

  it('缺一邊或格式不合回 null', () => {
    expect(penAreaM2(null, '3.00')).toBeNull()
    expect(penAreaM2('2.40', undefined)).toBeNull()
    expect(penAreaM2('abc', '3.00')).toBeNull()
  })
})

describe('formatMeters', () => {
  it('固定兩位小數，字串與數字都可', () => {
    expect(formatMeters(7.2)).toBe('7.20')
    expect(formatMeters('2.4')).toBe('2.40')
  })
})
