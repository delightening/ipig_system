// 欄位長寬（公尺）的前端驗證與換算。
// 後端才是最終裁決（只給一邊、非正數、超過 100 m、超過兩位小數皆回 400）；
// 這裡只是在送出前先提示，讓使用者少撞一次錯誤。

export const PEN_SIZE_MAX_M = 100

const DIMENSION_RE = /^\d+(\.\d{1,2})?$/

// 'bothRequired'：套用到整個區時兩格皆必填；'cannotClear'：已有尺寸的欄位不能清空（後端把空值視為「不改」）
export type PenSizeError = 'bothOrNone' | 'bothRequired' | 'invalid' | 'tooLarge' | 'cannotClear'

/** 解析單一邊長；格式不合回 null（不接受科學記號、負號、超過兩位小數） */
function parseDimension(value: string): number | null {
  const v = value.trim()
  if (!DIMENSION_RE.test(v)) return null
  return Number(v)
}

/** 長、寬都空 → 合法（表示不填尺寸）；只填一邊、非正數、超過上限 → 回錯誤代碼 */
export function validatePenSize(length: string, width: string): PenSizeError | null {
  const l = length.trim()
  const w = width.trim()
  if (!l && !w) return null
  if (!l || !w) return 'bothOrNone'
  const ln = parseDimension(l)
  const wn = parseDimension(w)
  if (ln === null || wn === null || ln <= 0 || wn <= 0) return 'invalid'
  if (ln > PEN_SIZE_MAX_M || wn > PEN_SIZE_MAX_M) return 'tooLarge'
  return null
}

/** 長 × 寬（m²）；先換成 0.01 m 的整數再相乘，避免浮點誤差（例：1.1 × 1.1）。任一邊無效回 null */
export function penAreaM2(length?: string | null, width?: string | null): number | null {
  if (length == null || width == null) return null
  const ln = parseDimension(length)
  const wn = parseDimension(width)
  if (ln === null || wn === null) return null
  return (Math.round(ln * 100) * Math.round(wn * 100)) / 10000
}

/** 顯示用：固定兩位小數（邊長、所需面積這類本來就只有兩位的值） */
export function formatMeters(value: number | string): string {
  return Number(value).toFixed(2)
}

/**
 * 顯示用：面積最多四位小數、至少兩位（尾端多餘的 0 去掉）。
 * 後端回傳的面積已四捨五入到兩位，但超標判定用的是未捨入的值；
 * 若也只顯示兩位，差不到 0.01 m² 的短缺會看起來「剛好夠」卻標超標。
 */
export function formatArea(value: number | string): string {
  const rounded = Math.round(Number(value) * 10000) / 10000
  return rounded.toFixed(4).replace(/0{1,2}$/, '')
}

/** 精確面積：優先由長、寬（兩位小數）整數運算；算不出來才退回後端已捨入的值 */
export function exactAreaM2(length: string | null | undefined, width: string | null | undefined, fallback: number): number {
  return penAreaM2(length, width) ?? fallback
}
