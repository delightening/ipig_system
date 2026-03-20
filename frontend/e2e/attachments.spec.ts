import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('附件管理 — 計畫書附件', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/protocols')
        }
        await expect(page).toHaveURL(/\/protocols/, { timeout: 12_000 })
    })

    test('計畫書列表頁應正常載入', async ({ page }) => {
        await expect(
            page.getByText(/計畫書|Protocols|AUP/i).first()
        ).toBeVisible({ timeout: 15_000 })
    })

    test('計畫書詳情應有附件 Tab', async ({ page }) => {
        // 等待列表載入
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 找到第一個可點擊的計畫書連結
        const protocolLink = page.locator('a[href*="/protocols/"]').first()
            .or(page.locator('table tbody tr').first())
        const linkCount = await protocolLink.count()

        if (linkCount > 0) {
            await protocolLink.click()
            await expect(page).toHaveURL(/\/protocols\/[^/]+$/, { timeout: 10_000 })

            // 計畫書詳情頁應有附件 Tab
            const attachmentsTab = page.getByRole('tab', { name: /附件|Attachment/i })
                .or(page.locator('[role="tab"]').filter({ hasText: /附件|Attachment/ }))
            await expect(attachmentsTab.first()).toBeVisible({ timeout: 15_000 })
        }
    })

    test('附件 Tab 應顯示附件表格或空狀態', async ({ page }) => {
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        const protocolLink = page.locator('a[href*="/protocols/"]').first()
            .or(page.locator('table tbody tr').first())
        const linkCount = await protocolLink.count()

        if (linkCount > 0) {
            await protocolLink.click()
            await expect(page).toHaveURL(/\/protocols\/[^/]+$/, { timeout: 10_000 })

            // 點擊附件 Tab
            const attachmentsTab = page.getByRole('tab', { name: /附件|Attachment/i })
                .or(page.locator('[role="tab"]').filter({ hasText: /附件|Attachment/ }))
            await expect(attachmentsTab.first()).toBeVisible({ timeout: 15_000 })
            await attachmentsTab.first().click()

            // 等待載入完成
            await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

            // 應有附件表格或空狀態
            const table = page.locator('table')
            const emptyState = page.getByText(/沒有|無附件|no attachment|尚無/i)
            const uploadButton = page.getByRole('button', { name: /上傳|Upload/i })
            await expect(
                table.or(emptyState).or(uploadButton).first()
            ).toBeVisible({ timeout: 15_000 })
        }
    })

    test('附件 Tab 應有上傳按鈕', async ({ page }) => {
        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        const protocolLink = page.locator('a[href*="/protocols/"]').first()
            .or(page.locator('table tbody tr').first())
        const linkCount = await protocolLink.count()

        if (linkCount > 0) {
            await protocolLink.click()
            await expect(page).toHaveURL(/\/protocols\/[^/]+$/, { timeout: 10_000 })

            const attachmentsTab = page.getByRole('tab', { name: /附件|Attachment/i })
                .or(page.locator('[role="tab"]').filter({ hasText: /附件|Attachment/ }))
            await expect(attachmentsTab.first()).toBeVisible({ timeout: 15_000 })
            await attachmentsTab.first().click()

            // 應有上傳按鈕（可能是 button 或 file input trigger）
            const uploadButton = page.getByRole('button', { name: /上傳|Upload/i })
                .or(page.locator('button').filter({ hasText: /上傳|Upload/ }))
            await expect(uploadButton.first()).toBeVisible({ timeout: 15_000 })
        }
    })
})

test.describe('附件管理 — 動物照片', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/animals')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/animals')
        }
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })
    })

    test('動物詳情頁應可瀏覽', async ({ page }) => {
        // 切換到 All Animals Tab
        const allTab = page.locator('button').filter({ hasText: /All Animals|全部/ })
        await expect(allTab.first()).toBeVisible({ timeout: 15_000 })
        await allTab.first().click()

        await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })

        // 找第一隻動物連結
        const animalLink = page.locator('a[href*="/animals/"]').first()
            .or(page.locator('table tbody tr').first())
        const linkCount = await animalLink.count()

        if (linkCount > 0) {
            await animalLink.click()
            await expect(page).toHaveURL(/\/animals\/[^/]+$/, { timeout: 10_000 })

            // 動物詳情頁應正常載入
            await expect(page.locator('.animate-spin')).toBeHidden({ timeout: 15_000 })
            await expect(page).not.toHaveURL(/\/login/)
        }
    })
})
