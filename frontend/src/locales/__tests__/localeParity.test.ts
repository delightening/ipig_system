import zhTW from '@/locales/zh-TW.json'
import en from '@/locales/en.json'

/**
 * 語言包鍵集合必須 zh-TW / en 完全一致。
 *
 * `fallbackLng: 'zh-TW'`：en 缺鍵時 UI 不會報錯，而是**靜默顯示中文**，看起來像「該翻的沒翻」。
 * 這支測試把「缺鍵」變成紅燈，而不是等使用者切到 en 才看到殘留中文。
 */

type Tree = { [key: string]: string | Tree }

function flatten(tree: Tree, prefix = ''): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [key, value] of Object.entries(tree)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (typeof value === 'string') out[path] = value
    else Object.assign(out, flatten(value, path))
  }
  return out
}

const zh = flatten(zhTW as Tree)
const enFlat = flatten(en as Tree)

// 語言自稱（endonym）本來就該固定寫該語言的文字，不算 en 殘留中文。
const EN_MAY_CONTAIN_CJK = new Set(['language.zhTW'])
const HAN = /[㐀-鿿]/

describe('locale parity（zh-TW ⇄ en）', () => {
  it('en 不缺任何 zh-TW 有的鍵', () => {
    const missing = Object.keys(zh).filter((k) => !(k in enFlat))
    expect(missing).toEqual([])
  })

  it('zh-TW 不缺任何 en 有的鍵', () => {
    const missing = Object.keys(enFlat).filter((k) => !(k in zh))
    expect(missing).toEqual([])
  })

  it('en 的值不含中文（語言自稱除外）', () => {
    const leaked = Object.entries(enFlat)
      .filter(([k, v]) => HAN.test(v) && !EN_MAY_CONTAIN_CJK.has(k))
      .map(([k]) => k)
    expect(leaked).toEqual([])
  })

  it('兩邊的內插變數（{{name}}）一致', () => {
    // 比的是「用到哪些變數」，不是出現次數（中文句可能重複用同一個 {{noun}}，英文句只用一次）
    const vars = (s: string) => [...new Set([...s.matchAll(/\{\{\s*([\w.]+)\s*\}\}/g)].map((m) => m[1]))].sort()
    const mismatched = Object.keys(zh)
      .filter((k) => k in enFlat)
      .filter((k) => JSON.stringify(vars(zh[k])) !== JSON.stringify(vars(enFlat[k])))
    expect(mismatched).toEqual([])
  })
})
