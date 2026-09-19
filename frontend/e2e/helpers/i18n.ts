/**
 * E2E 用的語言包比對工具
 *
 * 畫面文字走 react-i18next，瀏覽器語系決定顯示 zh-TW 或 en（CI 渲染 en）。
 * 測試寫死任一語言的字串，換語系就會找不到元素——登入鈕從寫死的「登入」改成
 * t('auth.login.submit') 之後，英文介面顯示「Log In」，auth.setup 因此整批失敗。
 *
 * txt('鍵') 直接讀 zh-TW 與 en 語言包，產生兩種語言都認得的 RegExp；
 * 翻譯改了測試自動跟上，不必同步改測試裡的字串。
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const LOCALES_DIR = path.resolve(__dirname, '../../src/locales')

type Messages = { [key: string]: string | Messages }

const LOCALES: Messages[] = ['zh-TW', 'en'].map(
    (lng) => JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, `${lng}.json`), 'utf8')) as Messages,
)

function lookup(messages: Messages, key: string): string {
    const value = key.split('.').reduce<string | Messages | undefined>(
        (node, part) => (node && typeof node === 'object' ? node[part] : undefined),
        messages,
    )
    if (typeof value !== 'string') {
        // 鍵打錯要立刻失敗，不要讓測試靜默地比對一個永遠找不到的字串
        throw new Error(`[e2e i18n] 語言包沒有這個鍵：${key}`)
    }
    return value
}

const escapeRegExp = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

/** {{var}} 插值位置改成任意字元，其餘照字面比對 */
const toPattern = (text: string) =>
    text
        .split(/\{\{\s*\w+\s*\}\}/)
        .map(escapeRegExp)
        .join('.*?')

/**
 * 回傳比對 zh-TW 或 en 翻譯的 RegExp。
 * @param exact 預設 true：整段文字相等（前後空白除外）；false：包含即可
 */
export function txt(key: string, { exact = true }: { exact?: boolean } = {}): RegExp {
    const alternatives = [...new Set(LOCALES.map((m) => toPattern(lookup(m, key))))]
    const body = `(?:${alternatives.join('|')})`
    return new RegExp(exact ? `^\\s*${body}\\s*$` : body)
}
