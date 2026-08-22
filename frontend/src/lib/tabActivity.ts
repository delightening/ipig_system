const TAB_IDLE_TIMEOUT_MS = 10 * 60 * 60 * 1000

let lastActivity = Date.now()

export function markTabActivity(): void {
  lastActivity = Date.now()
}

export function isTabIdle(): boolean {
  if (typeof window !== 'undefined' && '__TEST_TAB_IDLE__' in window) {
    return !!(window as unknown as Record<string, unknown>).__TEST_TAB_IDLE__
  }
  return Date.now() - lastActivity > TAB_IDLE_TIMEOUT_MS
}
