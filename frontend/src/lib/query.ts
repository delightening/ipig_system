import { useAuthStore } from '@/stores/auth'

/**
 * P2-R4-19: React Query staleTime 常數
 * - 即時資料（動物狀態）：30s
 * - 列表/計數：5min
 * - 參考資料（sources, species, templates）：10min
 * - 設定/偏好：30min
 */
export const STALE_TIME = {
  /** 即時資料（動物狀態、打卡等） */
  REALTIME: 30 * 1000,
  /** 列表/計數 */
  LIST: 5 * 60 * 1000,
  /** 參考資料（sources, species, templates, panels） */
  REFERENCE: 10 * 60 * 1000,
  /** 設定/偏好 */
  SETTINGS: 30 * 60 * 1000,
} as const

/** Session 即將過期前停止 polling 的緩衝時間 */
const EXPIRY_BUFFER_MS = 60_000

/**
 * Polling 守衛：分頁隱藏或 session 即將過期時回傳 false 停止輪詢。
 * 用於 TanStack Query 的 refetchInterval callback。
 */
export function shouldPoll(intervalMs: number): number | false {
  if (document.hidden) return false
  const { isAuthenticated, sessionExpiresAt } = useAuthStore.getState()
  if (!isAuthenticated) return false
  if (sessionExpiresAt && sessionExpiresAt - Date.now() < EXPIRY_BUFFER_MS) return false
  return intervalMs
}
