import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('HR 出勤打卡', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/hr/attendance')
        await expect(page).toHaveURL(/\/hr\/attendance/, { timeout: 12_000 })
    })

    test('出勤頁面應顯示', async ({ page }) => {
        const table = page.locator('table')
        const loading = page.getByText(/載入|loading/i)
        const empty = page.getByText(/沒有|無資料|no data|尚無|no record/i)
        await expect(table.or(loading).or(empty).first()).toBeVisible({ timeout: 15_000 })
    })

    test('應有打卡按鈕或打卡記錄', async ({ page }) => {
        // 打卡按鈕（上班 / 下班 / Clock In / Clock Out）或打卡紀錄表格
        const clockBtn = page.locator('button').filter({
            hasText: /打卡|上班|下班|Clock In|Clock Out|Check In|簽到/i,
        })
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(clockBtn.first().or(table).or(empty)).toBeVisible({ timeout: 15_000 })
    })

    test('日期選擇應可運作', async ({ page }) => {
        // 尋找日期輸入或日期選擇器按鈕
        const dateInput = page.locator('input[type="date"], input[placeholder*="日期"], input[placeholder*="date" i]')
        const datePicker = page.locator('button').filter({
            hasText: /\d{4}[-/]\d{2}|選擇日期|pick date/i,
        })
        const calendarIcon = page.locator('[data-testid*="date"], [aria-label*="date" i], [aria-label*="日期"]')

        const target = dateInput.or(datePicker).or(calendarIcon).first()
        await expect(target).toBeVisible({ timeout: 15_000 })

        // 點擊日期選擇器確認不會崩潰
        await target.click()
        await page.waitForTimeout(500)
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('出勤紀錄表格或空狀態應顯示', async ({ page }) => {
        await page.waitForTimeout(1000)
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無|no attendance|no record/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })
    })
})
