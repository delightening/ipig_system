import i18n from '@/lib/i18n'

import { navItemsConfig } from '../sidebarNavConfig'

/**
 * 選單 title 是 i18n 鍵（`nav.<title>`），不是顯示文字。
 * 過去有 ~45 個項目寫死中文且 translate:false，切到 en 選單仍是中文；
 * 這支測試釘住「每個選單項目兩種語言都有翻譯、且 en 不含中文」。
 */

interface TitledNode {
  title: string
  children?: TitledNode[]
}

function collectTitles(nodes: TitledNode[]): string[] {
  return nodes.flatMap((n) => [n.title, ...(n.children ? collectTitles(n.children) : [])])
}

const titles = collectTitles(navItemsConfig)
const HAN = /[㐀-鿿]/

describe('sidebarNavConfig 的 i18n', () => {
  it('title 皆為語言無關的鍵（不含中文）', () => {
    expect(titles.filter((t) => HAN.test(t))).toEqual([])
  })

  it('title 全域唯一（SortableNavItem 以 title 當 React key）', () => {
    const dup = titles.filter((t, i) => titles.indexOf(t) !== i)
    expect(dup).toEqual([])
  })

  it.each(['zh-TW', 'en'])('每個選單項目在 %s 都有 nav.* 翻譯', (lng) => {
    const missing = titles.filter((t) => !i18n.exists(`nav.${t}`, { lng }))
    expect(missing).toEqual([])
  })

  it('en 的選單文字不含中文', () => {
    const t = i18n.getFixedT('en')
    const leaked = titles.filter((title) => HAN.test(t(`nav.${title}`)))
    expect(leaked).toEqual([])
  })
})
