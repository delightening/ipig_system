import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('動物列表', () => {
    test.beforeEach(async ({ page }) => {
        await ensureAdminOnPage(page, '/animals')
        if (page.url().includes('/login')) {
            await ensureAdminOnPage(page, '/animals')
        }
        await expect(page).toHaveURL(/\/animals/, { timeout: 12_000 })
        // 用 data-testid 而非顯示文字：狀態 tab 的文案會隨 i18n 變，
        // 而以文字比對容易誤中棟別 tag（棟名來自 DB，不受 locale 影響）
        await expect(page.getByTestId('status-tab').first()).toBeVisible({ timeout: 15_000 })
    })

    test('應顯示動物列表頁面', async ({ page }) => {
        // 搜尋輸入框（頁面載入完成的指標）
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })
    })

    test('應有狀態篩選 Tab', async ({ page }) => {
        // 狀態 tab 與棟別 tag 已分列，這裡只驗狀態那一列（data-status 為 status enum，不隨 i18n 變動）
        await expect(page.getByTestId('status-tab').first()).toBeVisible({ timeout: 15_000 })
        await expect(page.locator('[data-testid="status-tab"][data-status="all"]')).toBeVisible({ timeout: 15_000 })
    })

    test('搜尋應能過濾動物', async ({ page }) => {
        const searchInput = page.locator('input[placeholder]').first()
        await expect(searchInput).toBeVisible({ timeout: 15_000 })

        await searchInput.fill('001')
        await page.waitForTimeout(800)
        await expect(page).not.toHaveURL(/\/login/)
    })

    test('品種篩選應可運作', async ({ page }) => {
        await expect(page.locator('input[placeholder]').first()).toBeVisible({ timeout: 15_000 })

        const breedSelect = page.locator('[role="combobox"]').first()
        await expect(breedSelect, '動物列表應有品種篩選 combobox').toBeVisible({ timeout: 10_000 })

        await breedSelect.click()
        await expect(page.locator('[role="option"]').first()).toBeVisible({ timeout: 5_000 })
    })

    test('切換到 All Animals Tab 應顯示表格', async ({ page }) => {
        const allTab = page.locator('[data-testid="status-tab"][data-status="all"]')
        await expect(allTab).toBeVisible({ timeout: 15_000 })

        await allTab.click()
        // 空狀態現用共用 TableEmptyRow（在 <Table> 內）→ 空時同時有 table + 訊息，故 .first() 避免 strict mode 兩元素衝突
        await expect(page.locator('table').or(page.getByText(/沒有|無資料|no data|尚無|no animals/i)).first()).toBeVisible({ timeout: 15_000 })

        const table = page.locator('table')
        const emptyState = page.getByText(/沒有|無資料|no data|尚無|no animals/i)
        await expect(table.or(emptyState).first()).toBeVisible({ timeout: 15_000 })
    })

    test('棟別 Tab 應可切換且反映在網址上', async ({ page }) => {
        // 單一「Pen View」tab 已拆成各棟舍 tag。棟名來自 /facilities/buildings，
        // 會隨設施設定改名或增減，因此不以顯示文字比對，改用 data-testid。
        const buildingTab = page.getByTestId('building-tab')
        await expect(buildingTab.first(), '應顯示棟別 tag').toBeVisible({ timeout: 15_000 })

        await buildingTab.first().click()

        // 每個 tag 對應一個可分享的網址；building 必須有值（不能是空的 building=）
        await expect(page).toHaveURL(/status=pen&building=[^&#]+/, { timeout: 10_000 })
    })
})
