import { test as setup } from '@playwright/test'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'
import {
    performLogin,
    getAdminCredentials,
    completeForceChangePassword,
    getCredentialsForUserSetup,
    E2E_NEW_PASSWORD,
} from './auth-helpers'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const authDir = path.join(__dirname, '.auth')

/**
 * Auth Setup：執行一次登入，將 cookie/storageState 存檔
 * 後續所有測試直接載入此 state，不需重複登入。
 *
 * 執行順序：admin 先於 user（兩者皆用 admin 時，admin 完成 force-change 後 user 需用新密碼）
 *
 * 帳密優先順序：
 *   - 環境變數 E2E_USER_* / E2E_ADMIN_*
 *   - 未設時從專案根目錄 .env 載入（ADMIN_INITIAL_PASSWORD、admin@ipig.local）
 */
setup('authenticate as admin', async ({ page }) => {
    const { email } = getAdminCredentials()
    const initialPassword = process.env.ADMIN_INITIAL_PASSWORD
    const altPassword = process.env.E2E_ADMIN_PASSWORD || E2E_NEW_PASSWORD
    const password = initialPassword || altPassword
    if (!password) {
        setup.skip()
        return
    }

    fs.mkdirSync(authDir, { recursive: true })
    // 先試 ADMIN_INITIAL_PASSWORD（fresh DB）；若 400 則試 E2eTest123!（admin 先前已完成 force-change）
    try {
        await performLogin(page, email, initialPassword || altPassword)
    } catch {
        if (initialPassword && altPassword !== initialPassword) {
            await performLogin(page, email, altPassword)
        } else {
            throw new Error('Admin login failed. If admin already changed password, set E2E_ADMIN_PASSWORD=E2eTest123! in .env')
        }
    }
    // 本機環境 admin 首次登入會被導向 force-change-password，需完成變更後才能存取其他頁面
    await completeForceChangePassword(page, password)
    await page.context().storageState({ path: path.join(authDir, 'admin.json') })
})

// user setup 需在 admin 之後（若使用 admin 帳號，admin 完成 force-change 後密碼已變更）
setup('authenticate as user', async ({ page }) => {
    const { email, password } = getCredentialsForUserSetup()
    if (!email || !password) {
        throw new Error(
            'Set E2E_USER_* or E2E_ADMIN_*, or in .env set ADMIN_INITIAL_PASSWORD (admin: admin@ipig.local).\n' +
                'Create test user: cargo run --bin create_test_user <email> <password> <name>',
        )
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await completeForceChangePassword(page, password)
    await page.context().storageState({ path: path.join(authDir, 'user.json') })
})
