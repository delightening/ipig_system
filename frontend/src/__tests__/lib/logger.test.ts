import { describe, it, expect, vi, beforeEach } from 'vitest'

describe('logger', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('logs in dev mode', async () => {
    // import.meta.env.DEV is true in test environment by default
    const { logger } = await import('@/lib/logger')
    const spy = vi.spyOn(console, 'log').mockImplementation(() => {})
    logger.log('test message')
    expect(spy).toHaveBeenCalledWith('test message')
  })

  it('warns in dev mode', async () => {
    const { logger } = await import('@/lib/logger')
    const spy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    logger.warn('warning')
    expect(spy).toHaveBeenCalledWith('warning')
  })

  it('errors in dev mode', async () => {
    const { logger } = await import('@/lib/logger')
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    logger.error('error msg')
    expect(spy).toHaveBeenCalledWith('error msg')
  })
})
