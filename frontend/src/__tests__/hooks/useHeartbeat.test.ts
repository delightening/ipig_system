import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useHeartbeat } from '@/hooks/useHeartbeat'

const mockPost = vi.fn()
vi.mock('@/lib/api', () => ({
  default: {
    post: (...args: unknown[]) => mockPost(...args),
  },
}))

describe('useHeartbeat', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    mockPost.mockResolvedValue({})
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  it('does not send heartbeat when not authenticated', () => {
    renderHook(() => useHeartbeat(false))
    vi.advanceTimersByTime(65_000)
    expect(mockPost).not.toHaveBeenCalled()
  })

  it('sends initial heartbeat when authenticated', () => {
    renderHook(() => useHeartbeat(true))
    expect(mockPost).toHaveBeenCalledWith('/auth/heartbeat')
  })

  it('adds activity listeners when authenticated', () => {
    const addSpy = vi.spyOn(window, 'addEventListener')
    renderHook(() => useHeartbeat(true))
    expect(addSpy).toHaveBeenCalledWith('mousemove', expect.any(Function), { passive: true })
    expect(addSpy).toHaveBeenCalledWith('keydown', expect.any(Function), { passive: true })
    expect(addSpy).toHaveBeenCalledWith('click', expect.any(Function), { passive: true })
    addSpy.mockRestore()
  })
})
