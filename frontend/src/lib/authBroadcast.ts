/**
 * Auth Broadcast Channel（sliding session B1）
 *
 * 用 BroadcastChannel 在同 origin 的多分頁間同步 auth 狀態，避免：
 * 1. 多分頁同時觸發 /auth/refresh → 後端 R46 race window 內視為併發
 *    （5s 內不算 reuse），雖無安全告警但會產生不必要的 audit 噪音與 DB 寫入
 * 2. 一個分頁登出後，其他分頁仍以為自己登入 → 下次 API call 才知道
 *
 * 設計：
 * - 用 BroadcastChannel('sliding-session-auth')，舊瀏覽器不支援時靜默退化
 *   （degrade 後 R46 race window + Promise singleton 仍能兜底）
 * - subscriber 收到訊息時**直接 setState**，不呼叫會 re-broadcast 的方法，
 *   避免 ping-pong 迴圈
 * - refresh 事件只接受比本地更新的 expiry（防止舊訊息覆蓋）
 * - **入口 type guard 驗證**：同源任何來源都可 post，必須驗證後才轉發給
 *   handler（CodeRabbit Critical review on PR #428）。R58 移除 Zod 後改用
 *   hand-rolled type guard，行為等價。
 */

export type AuthBroadcastMessage =
    | { type: 'refreshed'; accessTokenExpiresAt: number }
    | { type: 'cleared' }

const CHANNEL_NAME = 'sliding-session-auth'

const channel: BroadcastChannel | null =
    typeof BroadcastChannel !== 'undefined' ? new BroadcastChannel(CHANNEL_NAME) : null

function isAuthBroadcastMessage(data: unknown): data is AuthBroadcastMessage {
    if (typeof data !== 'object' || data === null) return false
    const obj = data as Record<string, unknown>
    if (obj.type === 'cleared') return true
    if (obj.type === 'refreshed') {
        return (
            typeof obj.accessTokenExpiresAt === 'number' &&
            Number.isInteger(obj.accessTokenExpiresAt) &&
            obj.accessTokenExpiresAt > 0
        )
    }
    return false
}

/** 從本分頁發送 auth 變更給其他分頁。瀏覽器不支援時靜默 no-op。 */
export function broadcastAuth(message: AuthBroadcastMessage): void {
    try {
        channel?.postMessage(message)
    } catch {
        // BroadcastChannel.postMessage 在 channel 關閉後 throw DOMException；
        // auth flow 不該因此中斷
    }
}

/**
 * 訂閱來自其他分頁的 auth 廣播。回傳 cleanup function。
 * 瀏覽器不支援 BroadcastChannel 時回傳 no-op cleanup。
 *
 * Type guard 驗證 event.data — 同源惡意來源 post 畸形 / 偽造 payload 會被
 * 靜默 drop，不會污染 auth 狀態。
 */
export function onAuthBroadcast(
    handler: (message: AuthBroadcastMessage) => void,
): () => void {
    if (!channel) return () => {}
    const listener = (event: MessageEvent<unknown>) => {
        if (!isAuthBroadcastMessage(event.data)) return
        handler(event.data)
    }
    channel.addEventListener('message', listener)
    return () => channel.removeEventListener('message', listener)
}

/** 測試用：是否在當前 runtime 支援 BroadcastChannel */
export function isAuthBroadcastSupported(): boolean {
    return channel !== null
}

// 跨 tab refresh 互斥：記錄最近一次「其他 tab 廣播 refreshed 的時間」，
// 用於 navigator.locks 持有期間內 skip 重複 refresh（同 lock 期間，第一個 tab
// refresh 完成廣播後，後續 tab 進 lock 時看見近期 refresh 直接 retry 原請求）。
let lastBroadcastRefreshAt = 0

if (channel) {
    channel.addEventListener('message', (event: MessageEvent<unknown>) => {
        if (!isAuthBroadcastMessage(event.data)) return
        if (event.data.type === 'refreshed') {
            lastBroadcastRefreshAt = Date.now()
        }
    })
}

/** 距上次「其他 tab refreshed 廣播」的毫秒數；無紀錄則回 Infinity。 */
export function msSinceLastBroadcastRefresh(): number {
    if (lastBroadcastRefreshAt === 0) return Infinity
    return Date.now() - lastBroadcastRefreshAt
}

/** 本 tab refresh 成功時呼叫，與廣播一致地更新本地時間戳。 */
export function markLocalRefresh(): void {
    lastBroadcastRefreshAt = Date.now()
}
