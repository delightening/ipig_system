import { test as setup } from '@playwright/test'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'
import { performLogin, getAdminCredentials } from './auth-helpers'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const authDir = path.join(__dirname, '.auth')

/**
 * Auth Setup：執行一次登入，將 cookie/storageState 存檔
 * 後續所有測試直接載入此 state，不需重複登入。
 *
 * 帳密優先順序：
 *   - 環境變數 E2E_USER_* / E2E_ADMIN_*
 *   - 未設時從專案根目錄 .env 載入（ADMIN_INITIAL_PASSWORD、admin@ipig.local）
 */
setup('authenticate as user', async ({ page }) => {
    let email = process.env.E2E_USER_EMAIL
    let password = process.env.E2E_USER_PASSWORD
    if (!email || !password) {
        const admin = getAdminCredentials()
        email = admin.email
        password = admin.password
    }
    if (!email || !password) {
        throw new Error(
            'Set E2E_USER_* or E2E_ADMIN_*, or in .env set ADMIN_INITIAL_PASSWORD (admin: admin@ipig.local).\n' +
                'Create test user: cargo run --bin create_test_user <email> <password> <name>',
        )
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await page.context().storageState({ path: path.join(authDir, 'user.json') })
})

setup('authenticate as admin', async ({ page }) => {
    const { email, password } = getAdminCredentials()
    if (!password) {
        setup.skip()
        return
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await page.context().storageState({ path: path.join(authDir, 'admin.json') })
})
