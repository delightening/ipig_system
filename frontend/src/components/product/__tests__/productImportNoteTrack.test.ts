import i18n from '@/lib/i18n'

/**
 * 產品匯入說明「追蹤批號／追蹤效期」那一行。
 *
 * 表頭名稱是 CSV 契約，刻意固定中文（由呼叫端傳入）；但句子本身的其餘部分
 * 要跟著語系走——曾經把「是/否」當插值從元件寫死傳入，en 介面因此中英夾雜。
 * 後端 product_parser.rs 的 parse_bool 同時接受 yes 與 是，兩種寫法都是正確說明。
 */

const HEADERS = { trackBatch: '追蹤批號', trackExpiry: '追蹤效期' }
const KEY = 'erpMaster.import.product.noteTrack'

let previousLanguage: string

beforeAll(() => {
  previousLanguage = i18n.language
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

it('en：除了傳入的中文表頭，句子沒有中文，也沒有未填的插值', async () => {
  await i18n.changeLanguage('en')
  const text = i18n.t(KEY, HEADERS)
  const withoutHeaders = text.replace(HEADERS.trackBatch, '').replace(HEADERS.trackExpiry, '')
  expect(withoutHeaders).not.toMatch(/[㐀-鿿]/)
  expect(text).not.toContain('{{')
  expect(text).toContain('yes/no')
})

it('zh-TW：維持 true/false 或 是/否', async () => {
  await i18n.changeLanguage('zh-TW')
  expect(i18n.t(KEY, HEADERS)).toBe('追蹤批號、追蹤效期：true/false 或 是/否')
})
