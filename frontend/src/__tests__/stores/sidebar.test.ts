import { describe, it, expect, beforeEach } from 'vitest'
import { useSidebarStore } from '@/stores/sidebar'

describe('useSidebarStore', () => {
  beforeEach(() => {
    // Reset store state between tests
    useSidebarStore.setState({ sidebarOpen: true })
  })

  it('defaults to open', () => {
    expect(useSidebarStore.getState().sidebarOpen).toBe(true)
  })

  it('sets sidebar open state', () => {
    useSidebarStore.getState().setSidebarOpen(false)
    expect(useSidebarStore.getState().sidebarOpen).toBe(false)

    useSidebarStore.getState().setSidebarOpen(true)
    expect(useSidebarStore.getState().sidebarOpen).toBe(true)
  })

  it('toggles sidebar', () => {
    useSidebarStore.getState().toggleSidebar()
    expect(useSidebarStore.getState().sidebarOpen).toBe(false)

    useSidebarStore.getState().toggleSidebar()
    expect(useSidebarStore.getState().sidebarOpen).toBe(true)
  })
})
