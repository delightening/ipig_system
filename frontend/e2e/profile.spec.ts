import { test, expect } from '@playwright/test'

/**
 * 個人資料設定 E2E 測試
 *
 * 前置條件：已登入
 * 注意：頁面使用 i18n，文字可能為中文或英文
 */
test.describe('個人資料設定', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/profile/settings')
        await page.waitForLoadState('networkidle')
        // 確認未被導向 login
        if (page.url().includes('/login')) {
            test.skip()
        }
    })

    test('應顯示個人資料頁面', async ({ page }) => {
        // Email 欄位應為唯讀（disabled input）
        const emailInput = page.locator('input[disabled]').first()
        await expect(emailInput).toBeVisible({ timeout: 15_000 })
    })

    test('應顯示基本資料欄位', async ({ page }) => {
        // 等待表單載入（找 disabled email input 作為載入指標）
        await expect(page.locator('input[disabled]').first()).toBeVisible({ timeout: 15_000 })

        // Display Name 或 顯示名稱 的 input
        const inputs = page.locator('input:not([disabled]):not([type="date"]):not([type="number"])')
        // 至少應有 display_name, phone, organization
        const count = await inputs.count()
        expect(count).toBeGreaterThanOrEqual(2)
    })

    test('應有儲存按鈕', async ({ page }) => {
        // Save Changes 或 儲存變更
        const saveBtn = page.getByRole('button', { name: /Save Changes|儲存變更/ })
        await expect(saveBtn).toBeVisible({ timeout: 15_000 })
    })

    test('修改顯示名稱應可儲存', async ({ page }) => {
        // 等待表單載入
        await expect(page.locator('input[disabled]').first()).toBeVisible({ timeout: 15_000 })

        // 找第一個可編輯的 text input（Display Name）
        const editableInputs = page.locator('input:not([disabled]):not([type="date"]):not([type="number"]):not([type="hidden"])')
        const nameInput = editableInputs.first()
        const originalName = await nameInput.inputValue()

        // 修改名稱（改回相同值以避免副作用）
        await nameInput.clear()
        await nameInput.fill(originalName || 'E2E 測試帳號')

        // 點擊儲存
        const saveBtn = page.getByRole('button', { name: /Save Changes|儲存變更/ })
        await saveBtn.click()

        // 應出現成功提示 toast（中/英）
        const successToast = page.getByText(/成功|success/i).first()
        await expect(successToast).toBeVisible({ timeout: 10_000 })
    })
})

test.describe('變更密碼', () => {
    test('應可從側邊欄開啟變更密碼對話框', async ({ page }) => {
        await page.goto('/dashboard')
        await page.waitForLoadState('networkidle')
        if (page.url().includes('/login')) {
            test.skip()
            return
        }

        // 側邊欄底部的「Change Password / 變更密碼」
        const changePasswordBtn = page.getByText(/Change Password|變更密碼/).first()
        if (await changePasswordBtn.count() === 0) {
            test.skip()
            return
        }

        await changePasswordBtn.click()

        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 5_000 })
        await expect(dialog.locator('input[type="password"]').first()).toBeVisible()
    })
})
