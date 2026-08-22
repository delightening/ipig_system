/**
 * 在 chromium 專案開始前重新登入 admin 並覆寫 admin.json，
 * 確保後續使用 admin storageState 的 test 拿到的是剛登入的 session，避免中途失效。
 */
import { test, expect } from './fixtures/coverage'

import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'
import { performLogin, getAdminCredentials } from './auth-helpers'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const authDir = path.join(__dirname, '.auth')

test.use({ storageState: { cookies: [], origins: [] } })

test('refresh admin storageState for chromium project', { timeout: 60_000 }, async ({ page }) => {
    const { email, password } = getAdminCredentials()
    if (!password) {
        test.skip(true, 'ADMIN_INITIAL_PASSWORD or E2E_ADMIN_PASSWORD not set')
        return
    }

    fs.mkdirSync(authDir, { recursive: true })
    await performLogin(page, email, password)
    await page.context().storageState({ path: path.join(authDir, 'admin.json') })

    // 確認已離開登入頁
    await expect(page).not.toHaveURL(/\/login/, { timeout: 5_000 })
})
