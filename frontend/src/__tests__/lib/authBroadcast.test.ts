import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
    broadcastAuth,
    onAuthBroadcast,
    isAuthBroadcastSupported,
    type AuthBroadcastMessage,
} from '@/lib/authBroadcast'

describe('authBroadcast', () => {
    let externalChannel: BroadcastChannel | null = null

    beforeEach(() => {
        externalChannel = new BroadcastChannel('sliding-session-auth')
    })

    afterEach(() => {
        externalChannel?.close()
        externalChannel = null
    })

    it('reports BroadcastChannel support in jsdom', () => {
        expect(isAuthBroadcastSupported()).toBe(true)
    })

    it('delivers refreshed message to other listeners', async () => {
        const received: AuthBroadcastMessage[] = []
        externalChannel!.onmessage = (e) => received.push(e.data)

        broadcastAuth({ type: 'refreshed', accessTokenExpiresAt: 12345 })

        await new Promise((r) => setTimeout(r, 10))
        expect(received).toEqual([{ type: 'refreshed', accessTokenExpiresAt: 12345 }])
    })

    it('delivers cleared message to other listeners', async () => {
        const received: AuthBroadcastMessage[] = []
        externalChannel!.onmessage = (e) => received.push(e.data)

        broadcastAuth({ type: 'cleared' })

        await new Promise((r) => setTimeout(r, 10))
        expect(received).toEqual([{ type: 'cleared' }])
    })

    it('onAuthBroadcast receives messages from external posters', async () => {
        const handler = vi.fn()
        const cleanup = onAuthBroadcast(handler)

        externalChannel!.postMessage({ type: 'refreshed', accessTokenExpiresAt: 999 })

        await new Promise((r) => setTimeout(r, 10))
        expect(handler).toHaveBeenCalledWith({ type: 'refreshed', accessTokenExpiresAt: 999 })

        cleanup()
    })

    it('onAuthBroadcast cleanup stops receiving', async () => {
        const handler = vi.fn()
        const cleanup = onAuthBroadcast(handler)
        cleanup()

        externalChannel!.postMessage({ type: 'cleared' })
        await new Promise((r) => setTimeout(r, 10))
        expect(handler).not.toHaveBeenCalled()
    })
})
