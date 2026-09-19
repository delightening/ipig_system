import i18n from '@/lib/i18n'

import { getQuickItems, getGloveSpecs } from '../constants'
import { formatPartnerType, formatSupplierCategory, formatCustomerCategory } from '../partners/constants'

/**
 * 使用者裁定 2026-09-19：
 * - 產品「快速選擇」按鈕顯示文字走雙語，但點按後寫入名稱／規格欄的值固定中文。
 * - 供應商／客戶匯出 CSV 的欄位值固定中文（format* 傳 'zh-TW'）。
 */
const HAN = /[㐀-鿿]/

describe('en 介面下的固定中文值', () => {
  beforeAll(async () => {
    await i18n.changeLanguage('en')
  })
  afterAll(async () => {
    await i18n.changeLanguage('zh-TW')
  })

  it('快速選擇品項：label 為英文、value 固定中文', () => {
    const items = getQuickItems(i18n.t.bind(i18n))
    expect(items.length).toBeGreaterThan(0)
    for (const item of items) {
      expect(HAN.test(item.label)).toBe(false)
      expect(item.value).toBeDefined()
      expect(HAN.test(item.value as string)).toBe(true)
    }
    expect(items.find((i) => i.id === 'glove')?.value).toBe('手套')
  })

  it('手套規格：primary/secondary 為英文、value 固定中文', () => {
    const specs = getGloveSpecs(i18n.t.bind(i18n))
    for (const spec of specs) {
      expect(HAN.test(spec.primary)).toBe(false)
      expect(HAN.test(spec.value?.primary ?? '')).toBe(true)
      expect(HAN.test(spec.value?.secondary ?? '')).toBe(true)
    }
    expect(specs.find((s) => s.id === 'l-powder-free')?.value).toEqual({ primary: 'L號', secondary: '無粉' })
  })

  it('夥伴匯出：format* 傳 zh-TW 時為中文、不傳時隨 UI 語系', () => {
    expect(formatPartnerType('supplier', 'zh-TW')).toBe('供應商')
    expect(HAN.test(formatPartnerType('supplier'))).toBe(false)
    expect(HAN.test(formatSupplierCategory('drug', 'zh-TW'))).toBe(true)
    expect(HAN.test(formatCustomerCategory('internal', 'zh-TW'))).toBe(true)
    expect(formatSupplierCategory(undefined, 'zh-TW')).toBe('')
  })
})
