/**
 * R97-3（純顯示階段）：批號效期狀態判斷。
 *
 * ⚠️ 這裡**只做視覺提示，不阻擋任何操作**。是否禁止領用過期批號，
 * 待實體盤點釐清「真過期」與「效期建檔錯誤」的比例後再定
 * （2026-08-16 實測：281 筆有效期在庫中 101 筆已過期，其中含 1991 年
 * 這種明顯打錯的日期，一刀切硬擋會誤傷仍可用的耗材）。
 *
 * 獨立成檔而非放在 `BatchNumberSelect.tsx`：從元件檔匯出非元件會觸發
 * `react-refresh/only-export-components` 警告，且這組判斷未來會被
 * 出庫核准等其他路徑共用。
 */

/** 幾天內到期算「即將到期」，給人來得及先領舊的。 */
export const EXPIRING_SOON_DAYS = 30

export interface ExpiryStatus {
  /** 效期已過（不含當天）。 */
  expired: boolean
  /** 距到期日的天數；今天到期為 0、已過期為負數；無或無法解析效期為 null。 */
  daysLeft: number | null
}

/** 無法判讀的效期一律回這個，呼叫端以 `daysLeft === null` 辨識。 */
const UNKNOWN: ExpiryStatus = { expired: false, daysLeft: null }

/** 僅接受 `YYYY-MM-DD`（DB 的 DATE 欄位序列化格式）。 */
const ISO_DATE = /^(\d{4})-(\d{2})-(\d{2})$/

/**
 * 以**本地日期零時**為基準比較，避免時區與執行時刻造成 off-by-one
 * ——直接拿 `Date.now()` 相減，會讓「今天到期」在下午變成負數而誤判為過期。
 *
 * ⚠️ **不能只用 `Number.isNaN(date.getTime())` 判斷有效性**
 * （CodeRabbit 於 PR #143 指出，已實測確認）：JS 對「月份合法但日期超界」的
 * 輸入會**靜默捲到下個月**，而不是回 Invalid Date——
 *
 * ```
 * new Date('2026-02-31T00:00:00') → Mar 03 2026
 * new Date('2026-04-31T00:00:00') → May 01 2026
 * new Date('2026-13-01T00:00:00') → Invalid Date   ← 只有月份超界會被抓到
 * ```
 *
 * 對一個判斷藥品效期的函式而言，把日期悄悄挪走是最糟的失敗方式：
 * 使用者會看到一個「看起來正常」但實際錯誤的天數。故改為建構後**回讀比對
 * 年/月/日**，對不上就當作無法判讀，而不是猜一個日期。
 *
 * 現況資料來自 PostgreSQL `DATE` 欄位、存不進 2/31；但本函式已標明供
 * 其他路徑共用，不倚賴呼叫端保證輸入乾淨。
 */
export function expiryStatus(expiry: string): ExpiryStatus {
  if (!expiry) return UNKNOWN
  const m = ISO_DATE.exec(expiry)
  if (!m) return UNKNOWN

  const [, yy, mm, dd] = m
  const year = Number(yy)
  const month = Number(mm)
  const day = Number(dd)

  // 直接以數值建構本地零時，省去字串解析的實作差異。
  const due = new Date(year, month - 1, day)
  // 回讀比對：2026-02-31 會捲成 3/3，三個欄位就對不上。
  if (
    due.getFullYear() !== year ||
    due.getMonth() !== month - 1 ||
    due.getDate() !== day
  ) {
    return UNKNOWN
  }

  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const daysLeft = Math.round((due.getTime() - today.getTime()) / 86_400_000)
  return { expired: daysLeft < 0, daysLeft }
}

/** 是否落在「即將到期」區間（尚未過期、但剩餘天數已達門檻）。 */
export function isExpiringSoon(status: ExpiryStatus): boolean {
  return !status.expired && status.daysLeft !== null && status.daysLeft <= EXPIRING_SOON_DAYS
}

/** 可排序的批號：批號字串 + 效期字串 + 判讀結果。 */
export interface SortableBatch extends ExpiryStatus {
  batch: string
  expiry: string
}

/**
 * FEFO 比較器：先到期先出。
 *
 * 三條規則，順序不可調換：
 *
 * 1. **能判讀效期的排在前面** —— 判定用 `daysLeft !== null`（能不能判讀），
 *    而不是 `expiry` 是否非空（有沒有字串）。`2026-02-31` 這種讀不出來的值
 *    有字串卻無意義，若當成有效日期就會被排到最前面，等於建議倉管優先領用
 *    一個效期不明的批號（CodeRabbit 於 PR #143 指出：顯示端已改用 `daysLeft`
 *    判斷，排序端當時漏改——同一個判準要一起換，否則畫面與排序各說各話）。
 * 2. 兩者皆可判讀 → 比效期日期。
 * 3. **日期相同、或兩者皆不可判讀 → 比批號** —— 少了這個 tie-breaker，
 *    comparator 會回 0，最終順序取決於 API 回傳順序，同樣的資料可能排出
 *    不同結果，畫面會無故跳動。
 */
export function compareByFefo(a: SortableBatch, b: SortableBatch): number {
  const aKnown = a.daysLeft !== null
  const bKnown = b.daysLeft !== null
  if (aKnown && bKnown) {
    const byExpiry = a.expiry.localeCompare(b.expiry)
    return byExpiry !== 0 ? byExpiry : a.batch.localeCompare(b.batch)
  }
  if (aKnown) return -1
  if (bKnown) return 1
  return a.batch.localeCompare(b.batch)
}
