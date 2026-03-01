import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useApiError } from '@/hooks/useApiError'

const toastMock = vi.fn()
vi.mock('@/components/ui/use-toast', () => ({
  toast: (...args: unknown[]) => toastMock(...args),
}))
vi.mock('@/lib/logger', () => ({
  logger: { error: vi.fn() },
}))
vi.mock('@/lib/validation', () => ({
  getApiErrorMessage: vi.fn((_err: unknown) => 'API error message'),
}))

describe('useApiError', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('returns handleError and withErrorHandling', () => {
    const { result } = renderHook(() => useApiError())
    expect(result.current).toHaveProperty('handleError')
    expect(result.current).toHaveProperty('withErrorHandling')
    expect(typeof result.current.handleError).toBe('function')
    expect(typeof result.current.withErrorHandling).toBe('function')
  })

  it('handleError shows toast and logs error', async () => {
    const { logger } = await import('@/lib/logger')
    const { result } = renderHook(() => useApiError())

    const err = new Error('test')
    result.current.handleError(err)

    expect(toastMock).toHaveBeenCalledWith({
      title: '操作失敗',
      description: 'API error message',
      variant: 'destructive',
    })
    expect(logger.error).toHaveBeenCalledWith('[API Error]', err)
  })

  it('handleError uses custom title when provided', () => {
    const { result } = renderHook(() => useApiError())

    result.current.handleError(new Error('test'), '自訂標題')

    expect(toastMock).toHaveBeenCalledWith({
      title: '自訂標題',
      description: 'API error message',
      variant: 'destructive',
    })
  })

  it('withErrorHandling returns result on success', async () => {
    const { result } = renderHook(() => useApiError())

    const res = await result.current.withErrorHandling(
      () => Promise.resolve({ id: 1 }),
      '成功'
    )

    expect(res).toEqual({ id: 1 })
    expect(toastMock).toHaveBeenCalledWith({ title: '成功', description: '成功' })
  })

  it('withErrorHandling returns null and shows toast on error', async () => {
    const { result } = renderHook(() => useApiError())

    const res = await result.current.withErrorHandling(
      () => Promise.reject(new Error('fail'))
    )

    expect(res).toBeNull()
    expect(toastMock).toHaveBeenCalledWith({
      title: '操作失敗',
      description: 'API error message',
      variant: 'destructive',
    })
  })
})
