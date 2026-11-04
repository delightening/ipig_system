import { TAIWAN_TIMEZONE } from '@/lib/utils'

/** GET /admin/internal/version 的回應（對應後端 handlers/version.rs 的 `VersionResponse`）。 */
export interface SystemVersionInfo {
  git_sha: string
  build_time: string
  migration_max_version: number
  rust_version: string
}

/** 畫面上顯示的 git SHA 長度（與部署驗收用的 `git rev-parse --short=12 HEAD` 一致）。 */
export const SHORT_SHA_LENGTH = 12

/** 後端在 build 時沒收到值的占位字串（backend/build.rs 的 fallback）。 */
const UNKNOWN = 'unknown'

function isUnknown(value: string): boolean {
  return value === '' || value.toLowerCase() === UNKNOWN
}

/** 短 SHA（前 12 碼）；後端回 `unknown` 或空值時回 null，由呼叫端顯示「未知」。 */
export function shortSha(raw: string | null | undefined): string | null {
  const value = (raw ?? '').trim()
  return isUnknown(value) ? null : value.slice(0, SHORT_SHA_LENGTH)
}

/**
 * 建置時間轉成台灣時間（GMT+8）的 `年月日 時:分`，排列順序依 `locale`。
 * 後端回 `unknown`、空值或無法解析的字串時回 null，由呼叫端顯示「未知」。
 *
 * 用 `hourCycle: 'h23'`：en 系語言預設的 12 小時制會帶 AM/PM，午夜也可能顯示 24:00。
 */
export function formatBuildTime(raw: string | null | undefined, locale: string): string | null {
  const value = (raw ?? '').trim()
  if (isUnknown(value)) return null
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return null
  return new Intl.DateTimeFormat(locale, {
    timeZone: TAIWAN_TIMEZONE,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).format(date)
}

/** migration 版本；後端查不到時回 0（空表），視為未知。 */
export function formatMigrationVersion(raw: number | null | undefined): string | null {
  return typeof raw === 'number' && Number.isFinite(raw) && raw > 0 ? String(raw) : null
}
