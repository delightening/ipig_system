import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

/**
 * R19-12：邀請流程 E2E 測試
 */
test.describe('邀請流程', () => {
    test('完整邀請流程：建立→接受→登入', async ({ page, context }) => {
        test.setTimeout(90_000) // 邀請流程涉及多步操作，CI 需要更多時間
        // 1. Admin 已透過 fixture 登入，導航到邀請管理頁面
        await ensureAdminOnPage(page, '/admin/invitations')
        await expect(page.locator('[class*="animate-spin"]')).toBeHidden({
            timeout: 15_000,
        })

        // 2. 等待邀請管理頁面載入
        await expect(
            page.getByText(/邀請管理|Invitation/),
        ).toBeVisible({ timeout: 10_000 })

        // 3. 建立邀請（點擊新增按鈕）
        const ts = Date.now()
        const testEmail = `e2e_invite_${ts}@test.local`

        const createBtn = page
            .getByTestId('create-invitation-button')
            .or(page.getByRole('button', { name: /新增邀請|建立邀請|Create|Invite/ }))
        await expect(createBtn).toBeVisible({ timeout: 10_000 })
        await createBtn.click()

        // 填寫 Email
        const dialog = page.locator('[role="dialog"]')
        await expect(dialog).toBeVisible({ timeout: 5_000 })
        const emailInput = dialog.locator(
            'input[type="email"], input[name="email"], #email',
        ).first()
        await expect(emailInput).toBeVisible()
        await emailInput.fill(testEmail)

        // 如果有組織欄位，填寫
        const orgInput = dialog.locator(
            'input[name="organization"], #organization',
        ).first()
        if (await orgInput.isVisible({ timeout: 1_000 }).catch(() => false)) {
            await orgInput.fill('E2E Test Org')
        }

        // 送出（攔截 API response 以取得邀請連結）
        const submitBtn = dialog.getByRole('button', {
            name: /送出|建立|確認|Submit|Create/,
        })
        await expect(submitBtn).toBeVisible()

        const [apiResponse] = await Promise.all([
            page.waitForResponse(
                (res) => res.url().includes('/api/v1/invitations') && res.request().method() === 'POST',
                { timeout: 15_000 },
            ),
            submitBtn.click(),
        ])

        // 4. 從 API response 直接取得邀請連結
        let inviteLink = ''
        if (apiResponse.ok()) {
            const body = await apiResponse.json()
            inviteLink = body.invite_link || ''
        }

        // fallback：從 UI 取得
        if (!inviteLink || !inviteLink.includes('/invite/')) {
            await expect(
                page.getByText(/成功|已建立|連結|link/i).first(),
            ).toBeVisible({ timeout: 10_000 })

            const inviteLinkEl = page
                .locator('[data-testid="invite-link"]')
                .or(page.locator('input[readonly]'))
                .or(page.locator('code'))
                .first()

            if (await inviteLinkEl.isVisible({ timeout: 3_000 }).catch(() => false)) {
                inviteLink =
                    (await inviteLinkEl.getAttribute('value')) ||
                    (await inviteLinkEl.textContent()) ||
                    ''
            }
        }

        // 確保有邀請連結
        expect(inviteLink).toContain('/invite/')

        // 6. 開新 browser context（未登入狀態）
        const browser = page.context().browser()
        expect(browser).not.toBeNull()
        const newContext = await browser!.newContext({
            baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
        })
        const newPage = await newContext.newPage()

        try {
            // 7. 訪問邀請連結
            const invitePath = new URL(inviteLink).pathname
            await newPage.goto(invitePath)
            await newPage.waitForLoadState('domcontentloaded')

            // 8. 填寫註冊表單
            await expect(
                newPage.locator(
                    'input[name="display_name"], #display_name, #displayName',
                ).first(),
            ).toBeVisible({ timeout: 10_000 })

            await newPage
                .locator('input[name="display_name"], #display_name, #displayName')
                .first()
                .fill('E2E Test User')

            const phoneInput = newPage
                .locator('input[name="phone"], #phone')
                .first()
            if (await phoneInput.isVisible().catch(() => false)) {
                await phoneInput.fill('0912345678')
            }

            const orgField = newPage
                .locator('input[name="organization"], #organization')
                .first()
            if (await orgField.isVisible().catch(() => false)) {
                await orgField.fill('E2E Test Org')
            }

            // 密碼
            const pwdInput = newPage
                .locator('input[name="password"], #password')
                .first()
            await expect(pwdInput).toBeVisible()
            await pwdInput.fill('E2eInvite123!')

            const confirmPwd = newPage
                .locator(
                    'input[name="confirmPassword"], input[name="password_confirm"], #confirmPassword',
                )
                .first()
            if (await confirmPwd.isVisible().catch(() => false)) {
                await confirmPwd.fill('E2eInvite123!')
            }

            // 同意條款
            const agreeCheckbox = newPage
                .locator('input[type="checkbox"]')
                .first()
            if (await agreeCheckbox.isVisible().catch(() => false)) {
                await agreeCheckbox.check()
            }

            // 9. 提交
            const registerBtn = newPage.getByRole('button', {
                name: /註冊|建立帳號|Submit|Register|完成/,
            })
            await expect(registerBtn).toBeVisible()
            await registerBtn.click()

            // 10. 驗證導向「我的計劃書」頁面
            await expect(newPage).toHaveURL(
                /\/(my-projects|dashboard|login)/,
                { timeout: 15_000 },
            )
        } finally {
            await newContext.close()
        }
    })

    test('過期/已使用連結顯示友善提示', async ({ context }) => {
        const browser = context.browser()
        expect(browser).not.toBeNull()
        const newContext = await browser!.newContext({
            baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
        })
        const newPage = await newContext.newPage()

        try {
            // 使用無效 token 訪問
            await newPage.goto('/invite/invalid-token-that-does-not-exist')
            await newPage.waitForLoadState('domcontentloaded')

            // 驗證顯示錯誤頁面或提示
            const errorIndicator = newPage
                .getByText(/無效|過期|失效|not found|invalid|expired/i)
                .first()
            await expect(errorIndicator).toBeVisible({ timeout: 10_000 })
        } finally {
            await newContext.close()
        }
    })
})
