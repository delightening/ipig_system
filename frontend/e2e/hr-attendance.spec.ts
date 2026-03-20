import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('出勤打卡', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/hr/attendance')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/hr/attendance')
        }
        await expect(page).toHaveURL(/\/hr\/attendance/, { timeout: 12_000 })
    })

    test('應顯示出勤管理頁面', async ({ page }) => {
        // 頁面標題或 Tab 作為載入完成指標
        await expect(
            page.getByText(/出勤|Attendance|打卡/i).first()
        ).toBeVisible({ timeout: 15_000 })
    })

    test('應有今日打卡與出勤記錄 Tab', async ({ page }) => {
        // Tab：今日打卡、出勤記錄
        const todayTab = page.getByRole('tab', { name: /今日|Today|打卡/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /今日|Today|打卡/ }))
        const historyTab = page.getByRole('tab', { name: /記錄|History|出勤/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /記錄|History|出勤/ }))

        await expect(todayTab.first()).toBeVisible({ timeout: 15_000 })
        await expect(historyTab.first()).toBeVisible({ timeout: 10_000 })
    })

    test('今日打卡 Tab 應有上班打卡按鈕', async ({ page }) => {
        // 今日打卡區塊應有打卡按鈕（上班/下班）
        const clockInButton = page.getByRole('button', { name: /上班|Clock.?In|打卡/i })
        const clockOutButton = page.getByRole('button', { name: /下班|Clock.?Out/i })

        // 至少有其中一個按鈕應可見（根據當天打卡狀態）
        await expect(
            clockInButton.first().or(clockOutButton.first())
        ).toBeVisible({ timeout: 15_000 })
    })

    test('切換到出勤記錄 Tab 應顯示篩選區', async ({ page }) => {
        // 點擊出勤記錄 Tab
        const historyTab = page.getByRole('tab', { name: /記錄|History|出勤/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /記錄|History|出勤/ }))
        await expect(historyTab.first()).toBeVisible({ timeout: 15_000 })
        await historyTab.first().click()

        // 應有日期範圍篩選
        const dateInput = page.locator('input[type="date"]').first()
        await expect(dateInput).toBeVisible({ timeout: 10_000 })
    })

    test('出勤記錄應有表格', async ({ page }) => {
        // 切換到記錄 Tab
        const historyTab = page.getByRole('tab', { name: /記錄|History|出勤/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /記錄|History|出勤/ }))
        await expect(historyTab.first()).toBeVisible({ timeout: 15_000 })
        await historyTab.first().click()

        // 等待載入
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 應有表格或空狀態
        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|無資料|no data|尚無|no records/i)
        await expect(table.or(emptyState).first()).toBeVisible({ timeout: 15_000 })
    })

    test('應有 Excel 匯出按鈕', async ({ page }) => {
        // 切換到記錄 Tab
        const historyTab = page.getByRole('tab', { name: /記錄|History|出勤/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /記錄|History|出勤/ }))
        await expect(historyTab.first()).toBeVisible({ timeout: 15_000 })
        await historyTab.first().click()

        // 匯出按鈕
        const exportButton = page.getByRole('button', { name: /匯出|Export|Excel|下載/i })
        await expect(exportButton.first()).toBeVisible({ timeout: 10_000 })
    })

    test('管理員應能查看所有人員出勤', async ({ page }) => {
        // 切換到記錄 Tab
        const historyTab = page.getByRole('tab', { name: /記錄|History|出勤/i })
            .or(page.locator('[role="tab"]').filter({ hasText: /記錄|History|出勤/ }))
        await expect(historyTab.first()).toBeVisible({ timeout: 15_000 })
        await historyTab.first().click()

        // admin 應有「查看所有人」開關
        const viewAllSwitch = page.locator('[role="switch"]')
            .or(page.getByText(/所有人|All Users|全部人員/i))
        await expect(viewAllSwitch.first()).toBeVisible({ timeout: 10_000 })
    })
})
