import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { AxiosError, AxiosHeaders } from 'axios'
import type { AxiosResponse } from 'axios'
import api, { attemptRefreshWithRetry } from '@/lib/api/client'

function buildErrorResponse(status: number): AxiosError {
    return new AxiosError(
        'fail',
        String(status),
        undefined,
        undefined,
        {
            status,
            statusText: '',
            headers: {},
            config: { headers: new AxiosHeaders() },
            data: {},
        } as AxiosResponse,
    )
}

function buildNetworkError(): AxiosError {
    return new AxiosError('Network Error', 'ERR_NETWORK')
}

function buildOkResponse(): AxiosResponse {
    return {
        data: {
            access_token: 'a',
            refresh_token: 'r',
            token_type: 'Bearer',
            expires_in: 900,
            user: { id: 'u', email: 'u@u', roles: [], permissions: [], is_active: true },
        },
        status: 200,
        statusText: 'OK',
        headers: {},
        config: { headers: new AxiosHeaders() },
    } as AxiosResponse
}

describe('attemptRefreshWithRetry', () => {
    beforeEach(() => {
        vi.useFakeTimers()
    })

    afterEach(() => {
        vi.useRealTimers()
        vi.restoreAllMocks()
    })

    it('returns response on first attempt success', async () => {
        const spy = vi.spyOn(api, 'post').mockResolvedValueOnce(buildOkResponse())

        const result = await attemptRefreshWithRetry()
        expect(spy).toHaveBeenCalledTimes(1)
        expect(result.data.expires_in).toBe(900)
    })

    it('throws immediately on 401 without retry', async () => {
        const err = buildErrorResponse(401)
        const spy = vi.spyOn(api, 'post').mockRejectedValueOnce(err)

        await expect(attemptRefreshWithRetry()).rejects.toBe(err)
        expect(spy).toHaveBeenCalledTimes(1)
    })

    it('throws immediately on 400 without retry', async () => {
        const err = buildErrorResponse(400)
        const spy = vi.spyOn(api, 'post').mockRejectedValueOnce(err)

        await expect(attemptRefreshWithRetry()).rejects.toBe(err)
        expect(spy).toHaveBeenCalledTimes(1)
    })

    it('retries once on 500 then succeeds', async () => {
        const spy = vi
            .spyOn(api, 'post')
            .mockRejectedValueOnce(buildErrorResponse(500))
            .mockResolvedValueOnce(buildOkResponse())

        const promise = attemptRefreshWithRetry()
        await vi.advanceTimersByTimeAsync(1000)
        const result = await promise
        expect(spy).toHaveBeenCalledTimes(2)
        expect(result.data.expires_in).toBe(900)
    })

    it('retries once on network error then succeeds', async () => {
        const spy = vi
            .spyOn(api, 'post')
            .mockRejectedValueOnce(buildNetworkError())
            .mockResolvedValueOnce(buildOkResponse())

        const promise = attemptRefreshWithRetry()
        await vi.advanceTimersByTimeAsync(1000)
        await promise
        expect(spy).toHaveBeenCalledTimes(2)
    })

    it('throws when both attempts fail', async () => {
        const err = buildErrorResponse(500)
        vi.spyOn(api, 'post')
            .mockRejectedValueOnce(err)
            .mockRejectedValueOnce(err)

        // 立刻 attach .catch handler，避免 microtask 階段 rejection 被 Node 觀察為
        // unhandled — CI 會把 PromiseRejectionHandledWarning 視為失敗（exit 1）。
        // await expect.rejects 內部 attach 太晚，會在 fake timer advance 期間被偵測。
        let captured: unknown
        const promise = attemptRefreshWithRetry().catch((e) => {
            captured = e
        })
        await vi.advanceTimersByTimeAsync(1000)
        await promise
        expect(captured).toBe(err)
    })
})
