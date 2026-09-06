import { describe, it, expect } from 'vitest'

import type { ProtocolConsumptionReport } from '@/types/report'
import {
  ROW_LIMIT,
  aggregateByProduct,
  aggregateByProtocol,
  buildCrossTab,
  cellKey,
  crossTabCsv,
  exportFilename,
  neutralizeFormula,
  round4,
  splitTruncationSignal,
  taipeiDateStamp,
  toCsv,
  toNum,
} from '../protocolConsumptionAggregate'

const PA = '11111111-1111-1111-1111-111111111111'
const PB = '22222222-2222-2222-2222-222222222222'
const GLOVE = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
const DROPPER = 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'

function row(over: Partial<ProtocolConsumptionReport>): ProtocolConsumptionReport {
  return {
    protocol_id: PA,
    protocol_no: 'P-001',
    iacuc_no: 'IACUC-001',
    protocol_title: '案件一',
    product_id: GLOVE,
    product_sku: 'CON-GLV-001',
    product_name: '無菌手套 6號',
    category_name: '消耗品',
    base_uom: '雙',
    qty_base: '10',
    doc_count: 1,
    first_trx_date: '2026-03-01T00:00:00Z',
    last_trx_date: '2026-03-01T00:00:00Z',
    total_cost: '20',
    ...over,
  }
}

describe('toNum', () => {
  it('空值與非數字一律當 0，不讓 NaN 汙染合計', () => {
    expect(toNum(null)).toBe(0)
    expect(toNum(undefined)).toBe(0)
    expect(toNum('')).toBe(0)
    expect(toNum('abc')).toBe(0)
    expect(toNum('12.5')).toBe(12.5)
    // 負數是合法的：領了又沖銷過頭時後端會回負值
    expect(toNum('-3')).toBe(-3)
  })
})

describe('aggregateByProtocol', () => {
  const rows = [
    row({ product_id: GLOVE, qty_base: '10', total_cost: '20', doc_count: 2 }),
    row({
      product_id: DROPPER,
      product_sku: 'CON-OTH-022',
      product_name: '滴管',
      base_uom: '包',
      qty_base: '3',
      total_cost: '9',
      doc_count: 3,
      first_trx_date: '2026-02-01T00:00:00Z',
      last_trx_date: '2026-05-01T00:00:00Z',
    }),
    row({
      protocol_id: PB,
      protocol_no: 'P-002',
      iacuc_no: null,
      protocol_title: '案件二',
      qty_base: '4',
      total_cost: '8',
    }),
  ]

  it('依案件分組，品項數與金額分開算', () => {
    const result = aggregateByProtocol(rows)
    expect(result).toHaveLength(2)

    const a = result[0]
    expect(a.protocol_no).toBe('P-001')
    expect(a.product_count).toBe(2)
    expect(a.total_cost).toBe(29)
  })

  it('🔴 案件層沒有數量合計欄位——雙加包沒有意義', () => {
    const a = aggregateByProtocol(rows)[0]
    expect(a).not.toHaveProperty('qty_base')
  })

  it('日期取跨品項的最早與最晚', () => {
    const a = aggregateByProtocol(rows)[0]
    expect(a.first_trx_date).toBe('2026-02-01T00:00:00Z')
    expect(a.last_trx_date).toBe('2026-05-01T00:00:00Z')
  })

  it('doc_count 取最大值當下界，不跨品項相加（同一張單常同時領多種東西）', () => {
    const a = aggregateByProtocol(rows)[0]
    expect(a.doc_count).toBe(3)
    expect(a.doc_count).not.toBe(5)
  })

  it('依案件編號排序，與輸入順序無關', () => {
    const shuffled = [rows[2], rows[0], rows[1]]
    expect(aggregateByProtocol(shuffled).map(p => p.protocol_no)).toEqual(['P-001', 'P-002'])
  })

  it('iacuc_no 為 null（DRAFT 計畫）不會讓分組壞掉', () => {
    const b = aggregateByProtocol(rows).find(p => p.protocol_no === 'P-002')
    expect(b?.iacuc_no).toBeNull()
    expect(b?.product_count).toBe(1)
  })
})

describe('aggregateByProduct', () => {
  const rows = [
    row({ protocol_id: PA, qty_base: '10', total_cost: '20' }),
    row({ protocol_id: PB, protocol_no: 'P-002', qty_base: '4', total_cost: '8' }),
    row({
      protocol_id: PA,
      product_id: DROPPER,
      product_sku: 'CON-OTH-022',
      product_name: '滴管',
      base_uom: '包',
      qty_base: '3',
      total_cost: '9',
    }),
  ]

  it('同一品項跨案件可以加總數量（單位相同）', () => {
    const glove = aggregateByProduct(rows).find(p => p.product_sku === 'CON-GLV-001')
    expect(glove?.qty_base).toBe(14)
    expect(glove?.protocol_count).toBe(2)
    expect(glove?.base_uom).toBe('雙')
  })

  it('protocol_count 去重，不是列數', () => {
    const dup = [...rows, row({ protocol_id: PA, qty_base: '1', total_cost: '2' })]
    const glove = aggregateByProduct(dup).find(p => p.product_sku === 'CON-GLV-001')
    expect(glove?.protocol_count).toBe(2)
    expect(glove?.qty_base).toBe(15)
  })

  it('依 SKU 排序', () => {
    expect(aggregateByProduct(rows).map(p => p.product_sku)).toEqual([
      'CON-GLV-001',
      'CON-OTH-022',
    ])
  })
})

describe('buildCrossTab', () => {
  const rows = [
    row({ protocol_id: PA, product_id: GLOVE, qty_base: '10' }),
    row({
      protocol_id: PB,
      protocol_no: 'P-002',
      product_id: DROPPER,
      product_sku: 'CON-OTH-022',
      product_name: '滴管',
      base_uom: '包',
      qty_base: '3',
    }),
  ]

  it('列為案件、行為品項，稀疏格不建 key', () => {
    const tab = buildCrossTab(rows)
    expect(tab.protocols.map(p => p.protocol_no)).toEqual(['P-001', 'P-002'])
    expect(tab.products.map(p => p.product_sku)).toEqual(['CON-GLV-001', 'CON-OTH-022'])

    expect(tab.cells.get(cellKey(PA, GLOVE))).toBe(10)
    expect(tab.cells.get(cellKey(PB, DROPPER))).toBe(3)
    // 案件一沒用過滴管 → 不建 key，讀取端當 0
    expect(tab.cells.has(cellKey(PA, DROPPER))).toBe(false)
  })

  it('同一格出現多列時相加，不是後者覆蓋前者', () => {
    const tab = buildCrossTab([
      row({ protocol_id: PA, product_id: GLOVE, qty_base: '10' }),
      row({ protocol_id: PA, product_id: GLOVE, qty_base: '5' }),
    ])
    expect(tab.cells.get(cellKey(PA, GLOVE))).toBe(15)
  })

  it('空輸入回空表而不是爆掉', () => {
    const tab = buildCrossTab([])
    expect(tab.protocols).toEqual([])
    expect(tab.products).toEqual([])
    expect(tab.cells.size).toBe(0)
  })
})

describe('splitTruncationSignal', () => {
  const many = (n: number) =>
    Array.from({ length: n }, (_, i) => row({ product_id: `p${i}`, product_sku: `SKU-${i}` }))

  it('剛好 ROW_LIMIT 筆 = 沒被截斷（這正是 length >= limit 會誤判的那一格）', () => {
    const r = splitTruncationSignal(many(ROW_LIMIT))
    expect(r.truncated).toBe(false)
    expect(r.rows).toHaveLength(ROW_LIMIT)
  })

  it('多一筆 = 確定被截斷，且那筆訊號列要被切掉不進畫面', () => {
    const r = splitTruncationSignal(many(ROW_LIMIT + 1))
    expect(r.truncated).toBe(true)
    expect(r.rows).toHaveLength(ROW_LIMIT)
  })

  it('少於上限一律不截斷', () => {
    expect(splitTruncationSignal(many(3)).truncated).toBe(false)
    expect(splitTruncationSignal([]).truncated).toBe(false)
  })
})

describe('taipeiDateStamp', () => {
  it('回 YYYY-MM-DD', () => {
    expect(taipeiDateStamp(new Date('2026-09-05T12:00:00Z'))).toMatch(/^\d{4}-\d{2}-\d{2}$/)
  })

  it('🔴 台灣時間凌晨用的是當天，不是 UTC 的前一天', () => {
    // 2026-09-05 00:30 (UTC+8) === 2026-09-04 16:30 UTC
    const earlyMorningTaipei = new Date('2026-09-04T16:30:00Z')
    expect(taipeiDateStamp(earlyMorningTaipei)).toBe('2026-09-05')
    // 對照：toISOString 會給出前一天，那正是本函式要避開的
    expect(earlyMorningTaipei.toISOString().split('T')[0]).toBe('2026-09-04')
  })

  it('台灣時間深夜仍是當天', () => {
    // 2026-09-05 23:30 (UTC+8) === 2026-09-05 15:30 UTC
    expect(taipeiDateStamp(new Date('2026-09-05T15:30:00Z'))).toBe('2026-09-05')
  })
})

describe('neutralizeFormula（CSV 公式注入）', () => {
  it('🔴 公式前綴開頭的字串前面加單引號', () => {
    for (const evil of ['=1+1', '+1', '-1+1', '@SUM(A1)']) {
      expect(neutralizeFormula(evil)).toBe(`'${evil}`)
    }
  })

  it('🔴 前置空白不能當免死金牌——Excel 會忽略它再解讀公式', () => {
    expect(neutralizeFormula('   =1+1')).toBe("'   =1+1")
    expect(neutralizeFormula(' @SUM(A1)')).toBe("' @SUM(A1)")
  })

  it('🔴 tab / CR / LF 開頭也要擋', () => {
    expect(neutralizeFormula('\t=1+1')).toBe("'\t=1+1")
    expect(neutralizeFormula('\rfoo')).toBe("'\rfoo")
    // LF 與 CR 是同一件事（換列注入），集合裡少一個等於沒擋
    expect(neutralizeFormula('\nfoo')).toBe("'\nfoo")
  })

  it('🔴 前置空白要取最寬定義，不只半形空格與 NBSP', () => {
    // 這些在「只剝 [ \u00a0]」的舊寫法下全部漏網：LF 不在觸發集合、也不被剝除
    expect(neutralizeFormula('\n=1+1')).toBe("'\n=1+1")
    expect(neutralizeFormula('\r\n=1+1')).toBe("'\r\n=1+1")
    expect(neutralizeFormula('\n\t  @SUM(A1)')).toBe("'\n\t  @SUM(A1)")
    expect(neutralizeFormula('\u00a0=1+1')).toBe("'\u00a0=1+1")
    // 全形空格：JS 的 \s 涵蓋它，舊的 [ \u00a0] 不涵蓋
    expect(neutralizeFormula('\u3000=1+1')).toBe("'\u3000=1+1")
  })

  it('🔴 含 LF 的惡意品名經 toCsv 後被中和（RFC 4180 引號擋不住公式）', () => {
    const csv = toCsv(['品名'], [['\n=1+1']])
    expect(csv).toBe('"品名"\r\n"' + "'" + '\n=1+1"')
  })

  it('一般文字原樣通過，不會被加引號', () => {
    for (const ok of ['無菌手套 6號', 'CON-GLV-001', '"太平洋" 10號導尿管', '2026-09-05', '']) {
      expect(neutralizeFormula(ok)).toBe(ok)
    }
  })

  it('經 toCsv 之後，惡意品名被中和且仍正確跳脫', () => {
    const csv = toCsv(['品名'], [['=HYPERLINK("http://evil","click")']])
    // 表頭與資料列是兩筆 record，以 CRLF 分隔
    expect(csv).toBe('"品名"\r\n"\'=HYPERLINK(""http://evil"",""click"")"')
  })

  it('🔴 數值欄位維持數值，負數不會被當公式加引號', () => {
    // -5 是 number 不是 string；加了引號下游就沒辦法加總了
    expect(toCsv(['金額'], [[-5]])).toBe('"金額"\r\n"-5"')
  })
})

describe('round4（浮點累加雜訊）', () => {
  it('消除 IEEE-754 的尾數', () => {
    expect(round4(0.1 + 0.2)).toBe(0.3)
    // 字面寫 1234.5678000000001 會被 no-loss-of-precision 擋，改成算出雜訊
    expect(round4(1234.5678 + 1e-12)).toBe(1234.5678)
  })

  it('保留 4 位小數，不會誤砍有效位數', () => {
    expect(round4(12.3456)).toBe(12.3456)
    expect(round4(-7.0001)).toBe(-7.0001)
  })

  it('aggregateByProduct 的加總不會漏出浮點尾巴', () => {
    const [p] = aggregateByProduct([
      row({ qty_base: '0.1', total_cost: '0.1' }),
      row({ protocol_id: PB, protocol_no: 'P-002', qty_base: '0.2', total_cost: '0.2' }),
    ])
    expect(p.qty_base).toBe(0.3)
    expect(p.total_cost).toBe(0.3)
  })

  it('交叉表的格值同樣收斂', () => {
    const tab = buildCrossTab([
      row({ product_id: GLOVE, qty_base: '0.1' }),
      row({ product_id: GLOVE, qty_base: '0.2' }),
    ])
    expect(tab.cells.get(cellKey(PA, GLOVE))).toBe(0.3)
  })
})

describe('exportFilename', () => {
  it('未截斷時不帶 _partial', () => {
    expect(exportFilename('cross', '2026-09-05', false)).toBe(
      'protocol_consumption_cross_2026-09-05.csv'
    )
  })

  it('🔴 截斷時檔名帶 _partial——警示框不會跟著 CSV 走，檔名會', () => {
    expect(exportFilename('cross', '2026-09-05', true)).toBe(
      'protocol_consumption_cross_partial_2026-09-05.csv'
    )
  })

  it('三個分頁各有自己的 kind', () => {
    const names = (['by-protocol', 'by-product', 'cross'] as const).map(k =>
      exportFilename(k, '2026-09-05', false)
    )
    expect(new Set(names).size).toBe(3)
    expect(names.every(n => n.endsWith('.csv'))).toBe(true)
  })
})

describe('crossTabCsv', () => {
  const tab = buildCrossTab([
    row({ protocol_id: PA, protocol_no: 'P-001', product_id: GLOVE, qty_base: '10' }),
    row({
      protocol_id: PB,
      protocol_no: 'P-002',
      product_id: DROPPER,
      product_sku: 'CON-OTH-022',
      product_name: '滴管',
      base_uom: '包',
      qty_base: '3',
    }),
  ])
  const uom = (u: string) => u

  it('🔴 沒有紀錄的格輸出空字串，不是 0', () => {
    const lines = crossTabCsv(tab, uom).split('\r\n')
    // 表頭 + 兩列案件
    expect(lines).toHaveLength(3)
    // P-001 用過手套(10)、沒用過滴管 → 第三欄必須是空字串
    expect(lines[1]).toBe('"P-001","10",""')
    // P-002 反過來
    expect(lines[2]).toBe('"P-002","","3"')
  })

  it('表頭把單位標在品名後面', () => {
    const header = crossTabCsv(tab, uom).split('\r\n')[0]
    expect(header).toBe('"計畫編號","無菌手套 6號(雙)","滴管(包)"')
  })

  it('uomLabel 由呼叫端決定，模組本身不依賴 UI 工具', () => {
    const header = crossTabCsv(tab, u => `<${u}>`).split('\r\n')[0]
    expect(header).toContain('<雙>')
  })
})

describe('toCsv', () => {
  it('品名含雙引號時按 RFC 4180 §2.7 escape，欄位不會錯位', () => {
    // 實查有品項叫 `"太平洋" 10號導尿管`
    const csv = toCsv(['品名', '數量'], [['"太平洋" 10號導尿管', 5]])
    expect(csv).toBe('"品名","數量"\r\n"""太平洋"" 10號導尿管","5"')
    // 逗號數量必須是每列各 1 個分隔逗號，escape 壞掉會多出來
    expect(csv.split('\r\n')[1].split('","')).toHaveLength(2)
  })

  it('record 之間是 CRLF 而非 LF（RFC 4180 §2.1）', () => {
    const csv = toCsv(['a'], [['1'], ['2']])
    expect(csv).toBe('"a"\r\n"1"\r\n"2"')
    // 不得出現落單的 LF——那代表某處用了 \n
    expect(csv.replace(/\r\n/g, '')).not.toContain('\n')
  })

  it('含逗號的內容被引號包住不會被當成分隔', () => {
    const csv = toCsv(['a', 'b'], [['x,y', 'z']])
    expect(csv.split('\r\n')[1]).toBe('"x,y","z"')
  })
})
