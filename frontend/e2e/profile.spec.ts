import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('個人資料設定', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/profile/settings')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 8_000 })
        // 個人資料頁載入後才有 disabled email 欄位（依賴 /me API）
        await expect(page.locator('input[disabled]').first(), '應已登入且進入個人資料頁').toBeVisible({ timeout: 20_000 })
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
        await page.waitForLoadState('domcontentloaded')
        await expect(page.locator('img[src*="pigmodel"]').first()).toBeVisible({ timeout: 15_000 })
        await expect(page).not.toHaveURL(/\/login/, { timeout: 5_000 })

        const changePasswordBtn = page.getByText(/Change Password|變更密碼/).first()
        await expect(changePasswordBtn, '側邊欄應有變更密碼按鈕').toBeVisible({ timeout: 10_000 })

        await changePasswordBtn.click()

        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 5_000 })
        await expect(dialog.locator('input[type="password"]').first()).toBeVisible()
    })
})
