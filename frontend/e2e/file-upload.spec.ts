import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('附件上傳/下載', () => {
    test('動物詳情頁面應有附件區塊', async ({ page }) => {
        // 先到動物列表取得第一筆動物 ID
        await ensureAdminOnPage(page, '/animals')
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })

        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })

        // 若無資料則跳過
        if (await empty.isVisible().catch(() => false)) {
            test.skip(true, '無動物資料，跳過附件測試')
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

        // 應有附件區塊或 Tab（附件 / Attachment / File / 檔案）
        const attachSection = page.getByText(/附件|attachment|file|檔案|document/i)
        await expect(attachSection.first()).toBeVisible({ timeout: 15_000 })
    })

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

        // 應有附件相關 Tab 或區塊
        const attachTab = page.getByText(/附件|attachment|file|檔案/i)
        await expect(attachTab.first()).toBeVisible({ timeout: 15_000 })
    })

    test('附件上傳按鈕應存在', async ({ page }) => {
        await ensureAdminOnPage(page, '/animals')
        const table = page.locator('table')
        const empty = page.getByText(/沒有|無資料|no data|尚無/i)
        await expect(table.or(empty).first()).toBeVisible({ timeout: 15_000 })

        if (await empty.isVisible().catch(() => false)) {
            test.skip(true, '無動物資料，跳過')
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

        // 上傳按鈕或 file input
        const uploadBtn = page.locator('button').filter({ hasText: /上傳|upload|選擇檔案/i })
        const fileInput = page.locator('input[type="file"]')
        const uploadArea = page.getByText(/拖曳|drag|drop|上傳/i)
        await expect(uploadBtn.first().or(fileInput.first()).or(uploadArea.first())).toBeVisible({ timeout: 15_000 })
    })

    test('附件列表或空狀態應顯示', async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
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

        // 附件列表或空狀態
        const fileList = page.locator('table, [class*="file"], [class*="attachment"]')
        const noFiles = page.getByText(/沒有附件|no attachment|no file|尚無檔案|無附件/i)
        const anyContent = page.getByText(/附件|attachment|file|檔案/i)
        await expect(fileList.first().or(noFiles.first()).or(anyContent.first())).toBeVisible({ timeout: 15_000 })
    })
})
