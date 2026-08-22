import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useProactiveRefresh } from '@/hooks/useProactiveRefresh'
import { useAuthStore } from '@/stores/auth'

describe('useProactiveRefresh', () => {
    const refreshSession = vi.fn().mockResolvedValue(true)

    beforeEach(() => {
        vi.useFakeTimers()
        useAuthStore.setState({
            accessTokenExpiresAt: null,
            refreshSession,
            isImpersonating: false,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)
    })

    afterEach(() => {
        vi.useRealTimers()
        refreshSession.mockClear()
    })

    it('does nothing when not authenticated', () => {
        renderHook(() => useProactiveRefresh(false))
        vi.advanceTimersByTime(20 * 60 * 1000)
        expect(refreshSession).not.toHaveBeenCalled()
    })

    it('bootstraps a refresh when accessTokenExpiresAt is null', () => {
        renderHook(() => useProactiveRefresh(true))
        expect(refreshSession).toHaveBeenCalledTimes(1)
    })

    // 模擬登入為純 access-only token（無 refresh token），主動續期會 400；一律跳過
    it('does not bootstrap a refresh when impersonating', () => {
        useAuthStore.setState({
            isImpersonating: true,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)

        renderHook(() => useProactiveRefresh(true))
        vi.advanceTimersByTime(20 * 60 * 1000)
        expect(refreshSession).not.toHaveBeenCalled()
    })

    it('does not schedule a TTL refresh when impersonating', () => {
        useAuthStore.setState({
            accessTokenExpiresAt: Date.now() + 15 * 60 * 1000,
            isImpersonating: true,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)

        renderHook(() => useProactiveRefresh(true))
        vi.advanceTimersByTime(20 * 60 * 1000)
        expect(refreshSession).not.toHaveBeenCalled()
    })

    it('schedules refresh at ~80% TTL when expiry is known', () => {
        useAuthStore.setState({
            accessTokenExpiresAt: Date.now() + 15 * 60 * 1000,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)

        renderHook(() => useProactiveRefresh(true))
        expect(refreshSession).not.toHaveBeenCalled()

        // 80% of 15 min = 12 min；11:59 還沒 fire
        vi.advanceTimersByTime(11 * 60 * 1000 + 59 * 1000)
        expect(refreshSession).not.toHaveBeenCalled()

        // 12:01 已 fire
        vi.advanceTimersByTime(2 * 1000)
        expect(refreshSession).toHaveBeenCalledTimes(1)
    })

    it('refreshes immediately if remaining TTL is under MIN_REFRESH_DELAY_MS', () => {
        useAuthStore.setState({
            accessTokenExpiresAt: Date.now() + 5_000,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)

        renderHook(() => useProactiveRefresh(true))
        expect(refreshSession).toHaveBeenCalledTimes(1)
    })

    it('clears timer on unmount before fire', () => {
        useAuthStore.setState({
            accessTokenExpiresAt: Date.now() + 15 * 60 * 1000,
        } as Partial<ReturnType<typeof useAuthStore.getState>>)

        const { unmount } = renderHook(() => useProactiveRefresh(true))
        vi.advanceTimersByTime(5 * 60 * 1000)
        unmount()
        vi.advanceTimersByTime(10 * 60 * 1000)
        expect(refreshSession).not.toHaveBeenCalled()
    })

    // D1: visibilitychange triggers
    describe('visibilitychange (D1)', () => {
        function setDocumentHidden(hidden: boolean) {
            Object.defineProperty(document, 'hidden', {
                configurable: true,
                get: () => hidden,
            })
            document.dispatchEvent(new Event('visibilitychange'))
        }

        it('refreshes on visibility=visible when token already expired', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() - 1000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(true))
            refreshSession.mockClear() // 排除主排程 immediate refresh

            setDocumentHidden(false)
            expect(refreshSession).toHaveBeenCalledTimes(1)
        })

        it('refreshes on visibility=visible when ttl < 30s', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() + 15 * 60 * 1000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(true))
            refreshSession.mockClear()

            // 把 expiry 縮到 20s 內，模擬 timer 因 sleep 而落後
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() + 20_000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)
            refreshSession.mockClear()

            setDocumentHidden(false)
            expect(refreshSession).toHaveBeenCalledTimes(1)
        })

        it('does not refresh on visibility=visible when ttl is healthy', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() + 15 * 60 * 1000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(true))
            refreshSession.mockClear()

            setDocumentHidden(false)
            expect(refreshSession).not.toHaveBeenCalled()
        })

        it('does not refresh on visibility=hidden', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() - 1000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(true))
            refreshSession.mockClear()

            setDocumentHidden(true)
            expect(refreshSession).not.toHaveBeenCalled()
        })

        it('does not refresh when not authenticated', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() - 1000,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(false))
            refreshSession.mockClear()

            setDocumentHidden(false)
            expect(refreshSession).not.toHaveBeenCalled()
        })

        it('does not refresh on visibility=visible when impersonating', () => {
            useAuthStore.setState({
                accessTokenExpiresAt: Date.now() - 1000,
                isImpersonating: true,
            } as Partial<ReturnType<typeof useAuthStore.getState>>)

            renderHook(() => useProactiveRefresh(true))
            refreshSession.mockClear()

            setDocumentHidden(false)
            expect(refreshSession).not.toHaveBeenCalled()
        })
    })
})
