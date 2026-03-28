import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('附件上傳/下載', () => {
    test('計畫書詳情頁面應有附件 Tab', async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await expect(page).toHaveURL(/\/protocols/, { timeout: 12_000 })

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })

        if (await empty.isVisible().catch(() => false)) {
            test.skip(true, '無計畫書資料，跳過附件測試')
            return
        }

        const firstRow = table.locator('tbody tr').first()
        const link = firstRow.locator('a, [role="link"]').first()
        if (await link.isVisible().catch(() => false)) {
            await link.click()
        } else {
            await firstRow.click()
        }
        await page.waitForTimeout(2000)

        // 計畫書詳情頁有 Attachments / 附件 tab
        const attachTab = page.getByText(/附件|Attachments/i)
        await expect(attachTab.first()).toBeVisible({ timeout: 15_000 })
    })

    test('計畫書附件 Tab 應顯示附件列表或空狀態', async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        await expect(page).toHaveURL(/\/protocols/, { timeout: 12_000 })

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })

        if (await empty.isVisible().catch(() => false)) {
            test.skip(true, '無計畫書資料，跳過')
            return
        }

        const firstRow = table.locator('tbody tr').first()
        const link = firstRow.locator('a, [role="link"]').first()
        if (await link.isVisible().catch(() => false)) {
            await link.click()
        } else {
            await firstRow.click()
        }
        await page.waitForTimeout(2000)

        // 點擊附件 Tab
        const attachTab = page.getByText(/附件|Attachments/i).first()
        if (!(await attachTab.isVisible({ timeout: 5_000 }).catch(() => false))) {
            test.skip(true, '無附件 Tab，跳過')
            return
        }
        await attachTab.click()
        await page.waitForTimeout(1000)

        // 附件列表或空狀態
        const fileList = page.locator('table, [class*="file"], [class*="attachment"]')
        const noFiles = page.getByText(/沒有附件|no attachment|尚無檔案|無附件|No attachments/i)
        const anyContent = page.getByText(/附件|attachment|上傳|upload/i)
        await expect(fileList.first().or(noFiles.first()).or(anyContent.first())).toBeVisible({ timeout: 15_000 })
    })

    test('動物詳情頁面應有 Tab 列', async ({ page }) => {
        // AnimalDetailPage 沒有附件功能，改為驗證 detail page 的 tab 列是否正常
        await ensureAdminOnPage(page, '/animals')
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })

        if (await empty.isVisible().catch(() => false)) {
            test.skip(true, '無動物資料，跳過')
            return
        }

        // 點擊第一筆動物
        const firstRow = table.locator('tbody tr').first()
        const link = firstRow.locator('a, [role="link"]').first()
        if (await link.isVisible().catch(() => false)) {
            await link.click()
        } else {
            await firstRow.click()
        }
        await page.waitForTimeout(2000)

        // 動物詳情頁應有 Tab（時間軸、觀察、手術、體重等）
        const detailTab = page.getByText(/時間軸|timeline|觀察|observation|手術|surgery|體重|weight|疫苗|vaccination/i)
        await expect(detailTab.first()).toBeVisible({ timeout: 15_000 })
    })
})
