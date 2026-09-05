import { TAIWAN_TIMEZONE } from '@/lib/utils'
import type { ProtocolConsumptionReport } from '@/types/report'

/**
 * 案件消耗報表的三種聚合軸。
 *
 * 後端回的是最細顆粒度（一列 = 一個案件 × 一個品項），三個分頁都由這裡轉出來，
 * 不各自打一次 API——同一份資料換三種看法，多打兩次只會讓三個分頁有機會不一致。
 *
 * 🔴 **案件彙總刻意沒有「數量合計」**。一個案件會用到手套（雙）、滴管（包）、
 * 紗布（片），把它們的 `qty_base` 加起來得到的數字沒有任何意義。金額可以加，
 * 數量不行。要看數量請展開到品項那一層，或用交叉表。
 */

/**
 * 後端的取用上限。後端實際取 `LIMIT 1001`，多的那一筆是截斷訊號：
 * 拿到超過這個數量就代表**確定還有更多**，等於這個數量則是**確定剛好取完**。
 * 用 `length >= LIMIT` 判斷會把後者誤報成前者。
 */
export const ROW_LIMIT = 1000

export interface LoadedRows {
  rows: ProtocolConsumptionReport[]
  truncated: boolean
}

/** 切掉截斷訊號用的那一筆，並回報是否真的被截斷。 */
export function splitTruncationSignal(raw: ProtocolConsumptionReport[]): LoadedRows {
  return raw.length > ROW_LIMIT
    ? { rows: raw.slice(0, ROW_LIMIT), truncated: true }
    : { rows: raw, truncated: false }
}

/**
 * 檔名用的日期戳（`YYYY-MM-DD`，台灣時間）。
 *
 * 不能用 `toISOString().split('T')[0]`——那是 UTC，台灣時間 00:00–07:59 之間
 * 匯出的檔名會標成前一天。`en-CA` 這個 locale 的短日期格式剛好就是 `YYYY-MM-DD`，
 * 是取得固定格式又能指定時區的標準做法（`formatDate` 走 `uiLocale()`，
 * 格式隨語系變動，不能拿來組檔名）。
 */
export function taipeiDateStamp(now: Date = new Date()): string {
  return now.toLocaleDateString('en-CA', { timeZone: TAIWAN_TIMEZONE })
}

/** `Decimal` 序列化成字串，空值當 0。 */
export function toNum(v: string | null | undefined): number {
  if (v === null || v === undefined || v === '') return 0
  const n = Number(v)
  return Number.isFinite(n) ? n : 0
}

/** 一個案件的彙總（跨品項）。注意沒有數量欄位，理由見檔頭。 */
export interface ProtocolTotal {
  protocol_id: string
  protocol_no: string
  iacuc_no: string | null
  protocol_title: string | null
  /** 這個案件動用過幾種品項 */
  product_count: number
  /** 貢獻的單據張數（跨品項去重） */
  doc_count: number
  total_cost: number
  first_trx_date: string
  last_trx_date: string
}

/** 一個品項的彙總（跨案件）。同一品項單位相同，數量可以加。 */
export interface ProductTotal {
  product_id: string
  product_sku: string
  product_name: string
  category_name: string | null
  base_uom: string
  /** 被幾個案件用過 */
  protocol_count: number
  qty_base: number
  total_cost: number
}

/** 交叉表：列＝案件、行＝品項。 */
export interface CrossTab {
  protocols: Array<Pick<ProtocolTotal, 'protocol_id' | 'protocol_no' | 'iacuc_no' | 'protocol_title'>>
  products: Array<Pick<ProductTotal, 'product_id' | 'product_sku' | 'product_name' | 'base_uom'>>
  /** 稀疏儲存：沒消耗過的組合不建 key，讀取端自行當 0。 */
  cells: Map<string, number>
}

/**
 * 交叉表的 cell key。
 *
 * 分隔字元是 NUL（\u0000），**寫成跳脫序列而不是字面字元**。UUID 不可能含 NUL，
 * 所以兩段 id 怎麼組合都不會撞 key；但字面的 NUL 會讓 git 把整個 .ts 判成二進位檔
 * （diff 渲染不出來、審查工具可能整份跳過），而且在編輯器裡完全看不見。
 * 這個檔案先前就是這樣，2026-09-05 修正。
 */
export function cellKey(protocolId: string, productId: string): string {
  return `${protocolId}\u0000${productId}`
}

/**
 * `doc_count` 是後端**依案件 × 品項**算出來的，跨品項相加會把同一張領用單
 * 重複計數（一張單常常同時領好幾種東西）。這裡取最大值當下界，
 * 明確標示為「至少幾張」而不是精確值——要精確得由後端另開一支查詢。
 */
function docCountLowerBound(rows: ProtocolConsumptionReport[]): number {
  return rows.reduce((max, r) => Math.max(max, r.doc_count), 0)
}

export function aggregateByProtocol(rows: ProtocolConsumptionReport[]): ProtocolTotal[] {
  const grouped = new Map<string, ProtocolConsumptionReport[]>()
  for (const row of rows) {
    const bucket = grouped.get(row.protocol_id)
    if (bucket) bucket.push(row)
    else grouped.set(row.protocol_id, [row])
  }

  return [...grouped.values()]
    .map(group => {
      const head = group[0]
      return {
        protocol_id: head.protocol_id,
        protocol_no: head.protocol_no,
        iacuc_no: head.iacuc_no,
        protocol_title: head.protocol_title,
        product_count: group.length,
        doc_count: docCountLowerBound(group),
        total_cost: group.reduce((sum, r) => sum + toNum(r.total_cost), 0),
        first_trx_date: group.reduce(
          (min, r) => (r.first_trx_date < min ? r.first_trx_date : min),
          head.first_trx_date
        ),
        last_trx_date: group.reduce(
          (max, r) => (r.last_trx_date > max ? r.last_trx_date : max),
          head.last_trx_date
        ),
      }
    })
    .sort((a, b) => a.protocol_no.localeCompare(b.protocol_no))
}

export function aggregateByProduct(rows: ProtocolConsumptionReport[]): ProductTotal[] {
  const grouped = new Map<string, ProtocolConsumptionReport[]>()
  for (const row of rows) {
    const bucket = grouped.get(row.product_id)
    if (bucket) bucket.push(row)
    else grouped.set(row.product_id, [row])
  }

  return [...grouped.values()]
    .map(group => {
      const head = group[0]
      return {
        product_id: head.product_id,
        product_sku: head.product_sku,
        product_name: head.product_name,
        category_name: head.category_name,
        base_uom: head.base_uom,
        protocol_count: new Set(group.map(r => r.protocol_id)).size,
        qty_base: group.reduce((sum, r) => sum + toNum(r.qty_base), 0),
        total_cost: group.reduce((sum, r) => sum + toNum(r.total_cost), 0),
      }
    })
    .sort((a, b) => a.product_sku.localeCompare(b.product_sku))
}

export function buildCrossTab(rows: ProtocolConsumptionReport[]): CrossTab {
  const cells = new Map<string, number>()
  for (const row of rows) {
    const key = cellKey(row.protocol_id, row.product_id)
    cells.set(key, (cells.get(key) ?? 0) + toNum(row.qty_base))
  }

  return {
    protocols: aggregateByProtocol(rows).map(p => ({
      protocol_id: p.protocol_id,
      protocol_no: p.protocol_no,
      iacuc_no: p.iacuc_no,
      protocol_title: p.protocol_title,
    })),
    products: aggregateByProduct(rows).map(p => ({
      product_id: p.product_id,
      product_sku: p.product_sku,
      product_name: p.product_name,
      base_uom: p.base_uom,
    })),
    cells,
  }
}

/**
 * 交叉表的 CSV 內容（表頭 + 資料列）。
 *
 * 🔴 **沒有消耗紀錄的格輸出空字串，不是 0。** 畫面上那格顯示「—」，與「領用後
 * 整筆沖銷」（淨額 0，根本不會進報表）是不同的兩件事；匯出成 `0` 等於把
 * 「沒有這筆紀錄」講成「量測到的結果是零」，下游拿去算平均或加總都會被汙染。
 *
 * `uomLabel` 由呼叫端傳入，因為單位的顯示轉換屬於 UI 層（`formatUom`），
 * 這個模組刻意保持不依賴 UI 工具。
 */
export function crossTabCsv(tab: CrossTab, uomLabel: (uom: string) => string): string {
  return toCsv(
    ['計畫編號', ...tab.products.map(p => `${p.product_name}(${uomLabel(p.base_uom)})`)],
    tab.protocols.map(pr => [
      pr.protocol_no,
      ...tab.products.map(pd => {
        const qty = tab.cells.get(cellKey(pr.protocol_id, pd.product_id))
        return qty === undefined ? '' : qty
      }),
    ])
  )
}

/**
 * 匯出檔名。
 *
 * 🔴 資料被截斷時檔名要帶 `_partial`。畫面上有黃色警示框說明殘缺，但**警示框不會
 * 跟著 CSV 走**——檔案一旦寄給稽核或存檔，「這只是前 1000 組」這件事就無聲消失了。
 * 檔名是唯一會跟著檔案一起移動的載體。
 *
 * 不在 CSV 內容裡加警告列：那會破壞欄位對齊，下游程式解析時反而更糟。
 */
export function exportFilename(kind: string, stamp: string, truncated: boolean): string {
  return `protocol_consumption_${kind}${truncated ? '_partial' : ''}_${stamp}.csv`
}

/** RFC 4180 §2.1：record 之間以 CRLF 分隔，不是 LF。 */
const CRLF = '\r\n'

/**
 * 組 CSV 內容（不含 BOM 與下載動作，那是頁面的事）。
 *
 * 兩處都照 RFC 4180：
 *
 * - **每格用雙引號包住，內含的雙引號 escape 成兩個**（§2.7）。品名裡出現 `"`
 *   是真的會發生的（實查有品項叫 `"太平洋" 10號導尿管`），不 escape 的話
 *   那一列的欄位會整個錯位。
 * - **record 之間用 CRLF**（§2.1）。用 `\n` 的話註解宣稱的 RFC 4180 就是假的，
 *   而且部分試算表與匯入工具只認 CRLF。
 */
export function toCsv(headers: string[], rows: Array<Array<string | number>>): string {
  const quote = (cell: string | number) => `"${String(cell).replace(/"/g, '""')}"`
  return [headers, ...rows].map(row => row.map(quote).join(',')).join(CRLF)
}
