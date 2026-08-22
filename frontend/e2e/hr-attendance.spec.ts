import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('HR 出勤打卡', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/hr/attendance')
        await expect(page).toHaveURL(/\/hr\/attendance/, { timeout: 12_000 })
    })

    test('今日打卡頁面應顯示', async ({ page }) => {
        // 頁面標題「出勤管理」應可見
        await expect(page.getByRole('heading', { name: /出勤管理|Attendance/i })).toBeVisible({ timeout: 15_000 })
    })

    test('應有打卡按鈕或打卡記錄', async ({ page }) => {
        const clockBtn = page.locator('button').filter({
            hasText: /打卡|上班|下班|Clock In|Clock Out|Check In|簽到/i,
        })
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(clockBtn.first().or(table).or(empty)).toBeVisible({ timeout: 15_000 })
    })

    test('切換至出勤記錄 Tab 後日期選擇應可運作', async ({ page }) => {
        // 點擊「出勤記錄」tab（history tab）
        const historyTab = page.locator('[role="tab"]').filter({
            hasText: /出勤記錄|history|歷史/i,
        })
        await expect(historyTab.first()).toBeVisible({ timeout: 10_000 })
        await historyTab.first().click()
        await page.waitForTimeout(500)

        // history tab 內有日期輸入
        const dateInput = page.locator('input[type="date"]')
        await expect(dateInput.first()).toBeVisible({ timeout: 15_000 })

        // 點擊日期選擇器確認不會崩潰
        await dateInput.first().click()
        await page.waitForTimeout(500)
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('出勤記錄 Tab 應顯示表格或空狀態', async ({ page }) => {
        // 點擊「出勤記錄」tab
        const historyTab = page.locator('[role="tab"]').filter({
            hasText: /出勤記錄|history|歷史/i,
        })
        await expect(historyTab.first()).toBeVisible({ timeout: 10_000 })
        await historyTab.first().click()
        await page.waitForTimeout(1000)

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no attendance|no record|查無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })
})
