import axios, { AxiosError } from 'axios'

export interface ApiErrorPayload {
  error: {
    message: string
    code: number
    blocking: boolean
    warning_type?: string
    existing_animals?: unknown[]
  }
}

export type ApiError = AxiosError<ApiErrorPayload>

export function getErrorMessage(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const data = error.response?.data as ApiErrorPayload | undefined
    return data?.error?.message || error.message
  }
  if (error instanceof Error) return error.message
  return '未知錯誤'
}
