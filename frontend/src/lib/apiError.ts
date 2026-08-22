/**
 * API 錯誤訊息擷取工具（從 `lib/validation.ts` 抽出，斷開對 zod 的相依）。
 *
 * 40+ 個 callsite 只用 `getApiErrorMessage`，不該因此把整個 zod runtime
 * 拖進 bundle。本檔零 zod、零 form-lib 相依。
 */
import { AxiosError } from 'axios'

/**
 * 從 Axios 錯誤中取得使用者友善的錯誤訊息
 *
 * 後端回傳格式：
 *   { error: { message: "...", code: 400, blocking: true } }
 *
 * 也支援：
 *   { message: "..." } 或純文字字串
 */
export function getApiErrorMessage(error: unknown, fallback = '操作失敗，請稍後再試'): string {
    if (error instanceof AxiosError) {
        // 網路錯誤（無回應）
        if (!error.response) {
            if (error.code === 'ECONNABORTED' || error.code === 'ETIMEDOUT') {
                return '請求逾時，請檢查網路連線後再試'
            }
            if (error.code === 'ERR_NETWORK') {
                return '無法連線至伺服器，請確認網路狀態'
            }
            return '網路連線異常，請稍後再試'
        }

        // 有回應時，優先使用後端回傳的訊息
        const data = error.response.data
        if (typeof data === 'string' && data.length > 0 && data.length < 200) return data
        if (data?.error?.message) return data.error.message
        if (data?.message) return data.message

        // 依 HTTP 狀態碼提供預設訊息
        const statusMessages: Record<number, string> = {
            400: '請求格式有誤，請檢查輸入內容',
            401: '登入已過期，請重新登入',
            403: '權限不足，無法執行此操作',
            404: '找不到相關資料',
            409: '資料衝突，請重新整理後再試',
            413: '上傳的檔案太大，請縮小檔案後重試',
            422: '提交的資料不符合格式要求',
            429: '操作過於頻繁，請稍後再試',
            500: '伺服器發生錯誤，請稍後再試',
            502: '伺服器暫時無法服務，請稍後再試',
            503: '系統維護中，請稍後再試',
        }
        const status = error.response.status
        if (statusMessages[status]) return statusMessages[status]
    }
    if (error instanceof Error) return error.message
    return fallback
}
