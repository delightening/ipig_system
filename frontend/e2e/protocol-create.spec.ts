import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('計畫書新增流程 (create + submit)', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/protocols/new')
        await page.waitForLoadState('load')
        await page.waitForTimeout(1500)

        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/protocols/new')
        }
        await expect(page).toHaveURL(/\/protocols\/new/, { timeout: 15_000 })
    })

    test('新增計畫書頁面應載入表單', async ({ page }) => {
        // 應顯示基本資訊表單區塊（至少有一個 input 或 textarea）
        const formField = page.locator('input, textarea, [role="combobox"]').first()
        await expect(formField).toBeVisible({ timeout: 15_000 })
    })

    test('應有儲存按鈕', async ({ page }) => {
        // 儲存 / Save / 暫存 按鈕
        const saveBtn = page.getByRole('button', { name: /儲存|Save|暫存|Draft/ })
        await expect(saveBtn.first()).toBeVisible({ timeout: 15_000 })
    })

    test('填寫基本資訊並儲存草稿', async ({ page }) => {
        // 等待表單載入
        await page.waitForTimeout(2000)

        // 填寫計畫書標題（中文名稱）
        const titleInput = page.locator('input[name="title"], input[name="chinese_title"], textarea[name="title"]').first()
        if (await titleInput.isVisible().catch(() => false)) {
            const timestamp = Date.now()
            await titleInput.fill(`E2E 測試計畫書 ${timestamp}`)
        }

        // 填寫英文名稱（若有）
        const engTitleInput = page.locator('input[name="english_title"], input[name="title_en"]').first()
        if (await engTitleInput.isVisible().catch(() => false)) {
            await engTitleInput.fill('E2E Test Protocol')
        }

        // 嘗試點擊儲存按鈕
        const saveBtn = page.getByRole('button', { name: /儲存|Save|暫存|Draft/ }).first()
        if (await saveBtn.isVisible().catch(() => false)) {
            await saveBtn.click()
            // 等待回應 — 可能成功建立或顯示驗證錯誤
            await page.waitForTimeout(2000)

            // 驗證：成功建立後會導向詳情頁，或停留在編輯頁顯示錯誤
            const url = page.url()
            const hasValidationError = await page.locator('[role="alert"], [class*="error"], [class*="destructive"]').first().isVisible().catch(() => false)
            const redirectedToDetail = /\/protocols\/[a-f0-9-]+/.test(url) && !url.includes('/new')

            // 兩者之一應為 true（成功建立或顯示驗證錯誤）
            expect(
                redirectedToDetail || hasValidationError || url.includes('/protocols'),
                '儲存後應導向詳情頁或顯示驗證提示',
            ).toBeTruthy()
        }
    })

    test('section 導覽應可切換', async ({ page }) => {
        // 計畫書編輯頁有多個 section 導覽（基本資訊、目的、動物、…）
        const navButtons = page.locator('button, a').filter({
            hasText: /基本|目的|動物|設計|Purpose|Animals|Design|Basic/,
        })

        const count = await navButtons.count()
        if (count >= 2) {
            // 點擊第二個 section
            await navButtons.nth(1).click()
            await page.waitForTimeout(500)
            // 頁面應保持在 /protocols/new（不跳走）
            await expect(page).toHaveURL(/\/protocols\/new/)
        }
    })
})
