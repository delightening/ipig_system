/**
 * R57-14: Sliding session E2E — per-tab idle logout + cross-tab broadcast.
 *
 * Uses `window.__TEST_TAB_IDLE__` flag (tabActivity.ts test hook) to simulate
 * 10h idle without actually waiting. Tests verify:
 * 1. Idle tab → visibilitychange → redirects to /login
 * 2. Active tab stays alive when idle tab gets cleared
 * 3. Cross-tab broadcast: logout in Tab A clears Tab B
 */
import { test, expect } from './fixtures/admin-context'

test.describe('Sliding session per-tab idle', () => {
  test('idle tab redirects to login on visibilitychange', async ({ page }) => {
    await page.goto('/dashboard')
    await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })

    // Simulate idle: set __TEST_TAB_IDLE__ = true
    await page.evaluate(() => {
      ;(window as Record<string, unknown>).__TEST_TAB_IDLE__ = true
    })

    // Trigger visibilitychange (simulates tab coming back to foreground after idle)
    //
    // 這裡的 evaluate 是「給刺激」，不是斷言——真正的驗收是下面那行 toHaveURL。
    // 而這個刺激會導致導向登入頁，也就是說 evaluate 是在跟自己觸發的導航賽跑：
    // 若導航搶在結果序列化回 Node 之前發生，Playwright 會丟
    // "Execution context was destroyed, most likely because of a navigation."
    // ——那不是失敗，那正是預期行為來得夠快。2026-08-20 於 CI 實際發生
    // （初次 + 2 次 retry 共三次，PR #8 的 run 32324818577）。
    //
    // 兩道防護，因為無法在沙盒重現、不賭單一機制：
    //   ① setTimeout 0：讓 evaluate 先返回，listener 於下一個 macrotask 才跑，
    //      消掉「listener 同步觸發導航」這條路徑。
    //   ② 仍 catch 該則錯誤：導航也可能來自 goto('/dashboard') 之後就已排隊的
    //      401 → client.ts 的 `window.location.href = '/login?reason=session_expired'`
    //      整頁重載，那條與本次 dispatch 無關，setTimeout 擋不住。
    // 只吞這一種錯誤訊息，其餘照樣往上拋，避免遮蔽真失敗。
    await page
      .evaluate(() => {
        Object.defineProperty(document, 'hidden', { value: false, writable: true })
        setTimeout(() => document.dispatchEvent(new Event('visibilitychange')), 0)
      })
      .catch((err: unknown) => {
        if (!/Execution context was destroyed/.test(String(err))) throw err
      })

    // Should redirect to login (clearAuthLocal → isAuthenticated=false → Navigate)
    await expect(page).toHaveURL(/\/login/, { timeout: 10_000 })
  })

  test('non-idle tab stays alive after visibilitychange', async ({ page }) => {
    await page.goto('/dashboard')
    await expect(page).not.toHaveURL(/\/login/, { timeout: 10_000 })

    // Ensure NOT idle
    await page.evaluate(() => {
      ;(window as Record<string, unknown>).__TEST_TAB_IDLE__ = false
    })

    // Trigger visibilitychange
    await page.evaluate(() => {
      Object.defineProperty(document, 'hidden', { value: false, writable: true })
      document.dispatchEvent(new Event('visibilitychange'))
    })

    // Should stay on dashboard
    await page.waitForTimeout(2_000)
    await expect(page).not.toHaveURL(/\/login/)
  })
})

test.describe('Cross-tab broadcast', () => {
  test.fixme('logout in Tab A clears Tab B', async ({ context }) => {
    const tabA = await context.newPage()
    const tabB = await context.newPage()

    await tabA.goto('/dashboard')
    await tabB.goto('/dashboard')

    await expect(tabA).not.toHaveURL(/\/login/, { timeout: 10_000 })
    await expect(tabB).not.toHaveURL(/\/login/, { timeout: 10_000 })

    // Tab A broadcasts 'cleared' via BroadcastChannel
    await tabA.evaluate(() => {
      const ch = new BroadcastChannel('sliding-session-auth')
      ch.postMessage({ type: 'cleared' })
      ch.close()
    })

    // Tab B should receive and redirect to login
    await expect(tabB).toHaveURL(/\/login/, { timeout: 10_000 })

    await tabA.close()
    await tabB.close()
  })

  test.fixme('refresh broadcast to idle Tab B clears it instead of extending', async ({ context }) => {
    const tabA = await context.newPage()
    const tabB = await context.newPage()

    await tabA.goto('/dashboard')
    await tabB.goto('/dashboard')

    await expect(tabA).not.toHaveURL(/\/login/, { timeout: 10_000 })
    await expect(tabB).not.toHaveURL(/\/login/, { timeout: 10_000 })

    // Mark Tab B as idle
    await tabB.evaluate(() => {
      ;(window as Record<string, unknown>).__TEST_TAB_IDLE__ = true
    })

    // Tab A broadcasts 'refreshed' (simulates successful token refresh)
    await tabA.evaluate(() => {
      const ch = new BroadcastChannel('sliding-session-auth')
      ch.postMessage({ type: 'refreshed', accessTokenExpiresAt: Date.now() + 900_000 })
      ch.close()
    })

    // Tab B should logout (idle tab doesn't extend)
    await expect(tabB).toHaveURL(/\/login/, { timeout: 10_000 })

    // Tab A should stay alive
    await expect(tabA).not.toHaveURL(/\/login/)

    await tabA.close()
    await tabB.close()
  })
})
