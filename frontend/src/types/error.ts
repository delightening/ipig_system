import { AxiosError } from 'axios'
import { isAxiosError } from '@/lib/api'

export interface ApiErrorPayload {
  error: {
    message: string
    code: number
    blocking: boolean
    warning_type?: string
    existing_animals?: unknown[]
    /** warning_type='unsettled_items' 時的明細：卡在該使用者身上、無人能接手的單據 */
    items?: string[]
  }
}

/** P2-R4-16: API 錯誤型別，onError 等處請使用 unknown 以保持型別安全 */
export type ApiError = AxiosError<ApiErrorPayload>

export function getErrorMessage(error: unknown): string {
  if (isAxiosError(error)) {
    const data = error.response?.data as ApiErrorPayload | undefined
    return data?.error?.message || error.message
  }
  if (error instanceof Error) return error.message
  return '未知錯誤'
}
