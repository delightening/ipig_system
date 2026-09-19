import { describe, it, expect, vi } from 'vitest'
import { getApiErrorMessage } from '@/lib/apiError'
import { AxiosError, AxiosHeaders } from 'axios'

// i18n.t() 回傳 key 本身；同專案其他測試的慣例，避免文案改寫影響斷言，
// 也擋掉 lib/i18n 的 init 副作用（語言偵測結果依環境而異）。
vi.mock('@/lib/i18n', () => ({
    default: { t: (key: string) => key },
}))

describe('getApiErrorMessage', () => {
    it('returns fallback for unknown error', () => {
        expect(getApiErrorMessage(null)).toBe('errors.api.operationFailed')
    })

    it('returns custom fallback', () => {
        expect(getApiErrorMessage(null, 'Custom')).toBe('Custom')
    })

    it('returns Error.message for plain Error', () => {
        expect(getApiErrorMessage(new Error('boom'))).toBe('boom')
    })

    it('extracts message from AxiosError with response.data.error.message', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: { error: { message: 'Server says no' } },
            status: 400,
            statusText: 'Bad Request',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('Server says no')
    })

    it('extracts message from AxiosError with response.data.message', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: { message: 'Simple message' },
            status: 400,
            statusText: 'Bad Request',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('Simple message')
    })

    it('extracts string response data', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: 'Plain text error',
            status: 400,
            statusText: 'Bad Request',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('Plain text error')
    })

    it('returns status-based message for 401', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: {},
            status: 401,
            statusText: 'Unauthorized',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('errors.api.status401')
    })

    it('returns status-based message for 429', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: {},
            status: 429,
            statusText: 'Too Many Requests',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('errors.api.status429')
    })

    it('handles network error (no response)', () => {
        const error = new AxiosError('Network Error', 'ERR_NETWORK')
        expect(getApiErrorMessage(error)).toBe('errors.api.cannotConnect')
    })

    it('handles timeout error', () => {
        const error = new AxiosError('timeout', 'ECONNABORTED')
        expect(getApiErrorMessage(error)).toBe('errors.api.timeout')
    })
})
