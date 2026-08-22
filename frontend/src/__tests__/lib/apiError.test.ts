import { describe, it, expect } from 'vitest'
import { getApiErrorMessage } from '@/lib/apiError'
import { AxiosError, AxiosHeaders } from 'axios'

describe('getApiErrorMessage', () => {
    it('returns fallback for unknown error', () => {
        expect(getApiErrorMessage(null)).toBe('操作失敗，請稍後再試')
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
        expect(getApiErrorMessage(error)).toBe('登入已過期，請重新登入')
    })

    it('returns status-based message for 429', () => {
        const error = new AxiosError('test', 'ERR', undefined, undefined, {
            data: {},
            status: 429,
            statusText: 'Too Many Requests',
            headers: {},
            config: { headers: new AxiosHeaders() },
        })
        expect(getApiErrorMessage(error)).toBe('操作過於頻繁，請稍後再試')
    })

    it('handles network error (no response)', () => {
        const error = new AxiosError('Network Error', 'ERR_NETWORK')
        expect(getApiErrorMessage(error)).toBe('無法連線至伺服器，請確認網路狀態')
    })

    it('handles timeout error', () => {
        const error = new AxiosError('timeout', 'ECONNABORTED')
        expect(getApiErrorMessage(error)).toBe('請求逾時，請檢查網路連線後再試')
    })
})
