/**
 * API 錯誤訊息擷取工具（從 `lib/validation.ts` 抽出，斷開對 zod 的相依）。
 *
 * 40+ 個 callsite 只用 `getApiErrorMessage`，不該因此把整個 zod runtime
 * 拖進 bundle。本檔零 zod、零 form-lib 相依。
 */
import { AxiosError } from 'axios'

import i18n from '@/lib/i18n'

/**
 * 從 Axios 錯誤中取得使用者友善的錯誤訊息
 *
 * 後端回傳格式：
 *   { error: { message: "...", code: 400, blocking: true } }
 *
 * 也支援：
 *   { message: "..." } 或純文字字串
 */
export function getApiErrorMessage(error: unknown, fallback = i18n.t('errors.api.operationFailed')): string {
    if (error instanceof AxiosError) {
        // 網路錯誤（無回應）
        if (!error.response) {
            if (error.code === 'ECONNABORTED' || error.code === 'ETIMEDOUT') {
                return i18n.t('errors.api.timeout')
            }
            if (error.code === 'ERR_NETWORK') {
                return i18n.t('errors.api.cannotConnect')
            }
            return i18n.t('errors.api.networkAbnormal')
        }

        // 有回應時，優先使用後端回傳的訊息
        const data = error.response.data
        if (typeof data === 'string' && data.length > 0 && data.length < 200) return data
        if (data?.error?.message) return data.error.message
        if (data?.message) return data.message

        // 依 HTTP 狀態碼提供預設訊息
        const statusMessages: Record<number, string> = {
            400: i18n.t('errors.api.status400'),
            401: i18n.t('errors.api.status401'),
            403: i18n.t('errors.api.status403'),
            404: i18n.t('errors.api.status404'),
            409: i18n.t('errors.api.status409'),
            413: i18n.t('errors.api.status413'),
            422: i18n.t('errors.api.status422'),
            429: i18n.t('errors.api.status429'),
            500: i18n.t('errors.api.status500'),
            502: i18n.t('errors.api.status502'),
            503: i18n.t('errors.api.status503'),
        }
        const status = error.response.status
        if (statusMessages[status]) return statusMessages[status]
    }
    if (error instanceof Error) return error.message
    return fallback
}
