import { defineConfig, devices } from '@playwright/test'
import path from 'path'
import { fileURLToPath } from 'url'

/**
 * Playwright E2E 測試設定
 *
 * 使用方式：
 *   npx playwright test          # 執行所有 E2E 測試
 *   npx playwright test --ui     # 開啟互動式 UI
 *   npx playwright codegen       # 錄製測試
 *
 * 環境變數：
 *   E2E_BASE_URL       前端 URL（預設 http://localhost:8080）
 *   E2E_USER_EMAIL     一般測試帳號
 *   E2E_USER_PASSWORD  一般測試密碼
 *   E2E_ADMIN_EMAIL    管理員帳號
 *   E2E_ADMIN_PASSWORD 管理員密碼
 */

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const authDir = path.join(__dirname, 'e2e', '.auth')

export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    timeout: 30_000,

    reporter: process.env.CI ? 'github' : 'html',

    use: {
        baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
        screenshot: 'only-on-failure',
        trace: 'on-first-retry',
    },

    projects: [
        // Auth setup：登入並儲存 cookie state
        {
            name: 'auth-setup',
            testMatch: /auth\.setup\.ts/,
        },

        // 主要瀏覽器（依賴 auth-setup）
        {
            name: 'chromium',
            use: {
                ...devices['Desktop Chrome'],
                storageState: path.join(authDir, 'user.json'),
            },
            dependencies: ['auth-setup'],
            testIgnore: /auth\.setup\.ts/,
        },
        {
            name: 'firefox',
            use: {
                ...devices['Desktop Firefox'],
                storageState: path.join(authDir, 'user.json'),
            },
            dependencies: ['auth-setup'],
            testIgnore: /auth\.setup\.ts/,
        },
        {
            name: 'webkit',
            use: {
                ...devices['Desktop Safari'],
                storageState: path.join(authDir, 'user.json'),
            },
            dependencies: ['auth-setup'],
            testIgnore: /auth\.setup\.ts/,
        },
    ],
})
