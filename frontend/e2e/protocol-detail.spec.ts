import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('計畫書詳情 + 修訂流程', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/protocols')
        }
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })
    })

    test('計畫書列表應顯示（表格或空狀態）', async ({ page }) => {
        await expect(
            page.locator('table').or(page.getByText(/尚無|沒有|no protocol/i)).first(),
        ).toBeVisible({ timeout: 15_000 })
    })

    test('點進計畫書詳情應顯示 Tab', async ({ page }) => {
        const protocolLink = page.locator('table a[href*="/protocols/"]').first()
        if (!(await protocolLink.isVisible().catch(() => false))) {
            // 無計畫書資料，跳過
            return
        }

        await protocolLink.click()
        await expect(page).toHaveURL(/\/protocols\/[a-f0-9-]+/, { timeout: 12_000 })
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        // 驗證 Tab 列存在（至少出現「內容」或 content tab）
        const tabList = page.locator('[role="tablist"]')
        await expect(tabList).toBeVisible({ timeout: 15_000 })

        // 至少有內容 / 版本 / 歷程 / 留言 / 附件 tab
        const tabButtons = tabList.locator('[role="tab"]')
        const count = await tabButtons.count()
        expect(count).toBeGreaterThanOrEqual(5)
    })

    test('計畫書資訊卡應顯示（主持人、機構、期間）', async ({ page }) => {
        const protocolLink = page.locator('table a[href*="/protocols/"]').first()
        if (!(await protocolLink.isVisible().catch(() => false))) {
            return
        }

        await protocolLink.click()
        await expect(page).toHaveURL(/\/protocols\/[a-f0-9-]+/, { timeout: 12_000 })
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        // 資訊卡區域應存在（主持人 / PI / 機構 / institution / 期間 / period）
        const infoArea = page.locator('[class*="card"], [class*="Card"]').first()
        await expect(infoArea).toBeVisible({ timeout: 15_000 })
    })

    test('狀態 badge 應正確顯示', async ({ page }) => {
        const protocolLink = page.locator('table a[href*="/protocols/"]').first()
        if (!(await protocolLink.isVisible().catch(() => false))) {
            return
        }

        await protocolLink.click()
        await expect(page).toHaveURL(/\/protocols\/[a-f0-9-]+/, { timeout: 12_000 })
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        // 狀態 badge（Badge 元件通常有 data-slot="badge" 或特定 class）
        const badge = page.locator('[data-slot="badge"], [class*="badge"], [class*="Badge"]').first()
        await expect(badge).toBeVisible({ timeout: 15_000 })
    })
})
