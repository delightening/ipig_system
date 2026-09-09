import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'

import zhTW from '@/locales/zh-TW.json'
import en from '@/locales/en.json'

/** 每個測試自行決定「現在是不是訪客」 */
let guest = false
vi.mock('@/stores/auth', () => ({
  useAuthIsGuest: () => guest,
}))

/**
 * i18n：t() 回 key 本身，斷言比對 key 不比對文案，文案改寫不會弄紅這支測試。
 * ⚠️ 必須一併導出 initReactI18next——mock 掉整個模組後，若相依鏈拉進 `lib/i18n`
 * （它會 `.use(initReactI18next)`），少了這個 plugin 形狀 i18next 會在 import 階段就炸。
 */
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

import { GuestDateNotice } from '@/components/ui/guest-date-notice'

const KEY = 'guest.dateFilterInactive'

describe('GuestDateNotice', () => {
  beforeEach(() => {
    guest = false
  })

  it('訪客身分顯示提示', () => {
    guest = true
    render(<GuestDateNotice />)
    expect(screen.getByText(KEY)).toBeInTheDocument()
  })

  // 這是本元件唯一的職責。退化成「一律顯示」的話，正式環境的使用者
  // 會被告知他們的報表是示範資料——那比不顯示提示嚴重得多。
  it('🔴 非訪客時什麼都不渲染', () => {
    const { container } = render(<GuestDateNotice />)
    expect(screen.queryByText(KEY)).not.toBeInTheDocument()
    expect(container).toBeEmptyDOMElement()
  })

  /**
   * icon 純裝飾，不該被螢幕閱讀器唸出來。
   *
   * ⚠️ 這條**不是元件程式碼的 mutation 標的**：實測把元件裡顯式的
   * `aria-hidden="true"` 拿掉，這條仍然綠——`lucide-react` 的 icon 預設
   * 就會輸出 `aria-hidden="true"`。所以它驗的是「最終 DOM 具備這個契約」，
   * 來源是 lucide 或我們都算數。
   *
   * 元件裡仍保留顯式標註：不把 a11y 押在第三方的預設行為上，
   * lucide 改版時這條測試就是那個變動的偵測器。
   */
  it('裝飾用 icon 對輔助技術隱藏', () => {
    guest = true
    const { container } = render(<GuestDateNotice />)
    const svg = container.querySelector('svg')
    expect(svg).not.toBeNull()
    expect(svg?.getAttribute('aria-hidden')).toBe('true')
  })
})

describe('guest.dateFilterInactive 的翻譯', () => {
  // 元件用 t(KEY)。key 打錯或漏了某個語系，畫面上只會出現一串 key——
  // 那種缺陷在上面的 UI 測試裡看不出來（t 被 mock 成回傳 key），只能在這裡擋。
  it('zh-TW 與 en 都有這個 key，且不是空字串', () => {
    const zhVal = (zhTW as { guest: Record<string, unknown> }).guest?.dateFilterInactive
    const enVal = (en as { guest: Record<string, unknown> }).guest?.dateFilterInactive
    expect(typeof zhVal).toBe('string')
    expect(typeof enVal).toBe('string')
    expect((zhVal as string).trim().length).toBeGreaterThan(0)
    expect((enVal as string).trim().length).toBeGreaterThan(0)
  })

  it('兩個語系的 guest 區塊 key 集合一致', () => {
    const zhKeys = Object.keys((zhTW as { guest: object }).guest).sort()
    const enKeys = Object.keys((en as { guest: object }).guest).sort()
    expect(zhKeys).toEqual(enKeys)
  })
})
