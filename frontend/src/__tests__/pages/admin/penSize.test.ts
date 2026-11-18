import { describe, expect, it } from 'vitest'
import { exactAreaM2, formatArea, formatMeters, penAreaM2, validatePenSize } from '@/pages/admin/utils/penSize'

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

describe('formatArea', () => {
  it('最多四位小數、至少兩位，尾端多餘的 0 去掉', () => {
    expect(formatArea(3.24)).toBe('3.24')
    expect(formatArea(3)).toBe('3.00')
    expect(formatArea(3.2)).toBe('3.20')
    expect(formatArea('3.205')).toBe('3.205')
  })

  it('差不到 0.01 的短缺仍看得出來（後端面積已四捨五入到兩位）', () => {
    // 1.55 × 2.09 = 3.2395：四捨五入成 3.24 會和需求 3.24 看起來一樣
    const exact = penAreaM2('1.55', '2.09')
    expect(exact).toBe(3.2395)
    expect(formatArea(exact ?? 0)).toBe('3.2395')
    expect(formatArea(3.24 - (exact ?? 0))).toBe('0.0005')
  })
})

describe('exactAreaM2', () => {
  it('有長寬時用長寬算，不用後端已捨入的值', () => {
    expect(exactAreaM2('1.55', '2.09', 3.24)).toBe(3.2395)
  })

  it('長寬缺少或無效才退回後端的值', () => {
    expect(exactAreaM2(null, '2.09', 3.24)).toBe(3.24)
    expect(exactAreaM2('abc', '2.09', 3.24)).toBe(3.24)
  })
})

describe('formatMeters', () => {
  it('固定兩位小數，字串與數字都可', () => {
    expect(formatMeters(7.2)).toBe('7.20')
    expect(formatMeters('2.4')).toBe('2.40')
  })
})
