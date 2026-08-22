import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('HR 加班管理', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/hr/overtime')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/hr/overtime')
        }
        await expect(page).toHaveURL(/\/hr\/overtime/, { timeout: 12_000 })
    })

    test('應顯示加班管理頁面', async ({ page }) => {
        const table = page.locator('table')
        const loading = page.getByText(/載入|loading/i)
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(loading).or(empty).first()).toBeVisible({ timeout: 15_000 })
    })

    test('Tab 切換（我的加班/待審核/加班紀錄）', async ({ page }) => {
        // EN: "My Overtime", "Pending Approvals", "All Records"
        // ZH: "我的加班", "待審核", "加班紀錄"
        const tabs = page.locator('button').filter({
            hasText: /My Overtime|Pending|All Records|我的加班|待審核|加班紀錄/,
        })
        await expect(tabs.first()).toBeVisible({ timeout: 15_000 })
        const count = await tabs.count()
        expect(count).toBeGreaterThanOrEqual(2)
    })

    test('新增加班對話框應可開啟', async ({ page }) => {
        const addBtn = page.locator('button').filter({
            hasText: /New|Add|Create|新增|申請加班/,
        })
        await expect(addBtn.first()).toBeVisible({ timeout: 15_000 })

        await addBtn.first().click()
        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 15_000 })
    })

    test('表格或空狀態應顯示', async ({ page }) => {
        // 等待頁面內容載入
        await page.waitForTimeout(1000)
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no overtime|no records/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })
})
