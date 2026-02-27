import { defineConfig, devices } from '@playwright/test'
import path from 'path'
import { fileURLToPath } from 'url'
import dotenv from 'dotenv'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
// 從專案根目錄載入 .env，使 E2E_BASE_URL、ADMIN_INITIAL_PASSWORD 等可用
dotenv.config({ path: path.resolve(__dirname, '../.env') })

/**
 * Playwright E2E 測試設定
 *
 * 使用方式：
 *   npx playwright test          # 執行所有 E2E 測試
 *   npx playwright test --ui     # 開啟互動式 UI
 *   npx playwright codegen       # 錄製測試
 *
 * 帳密與 URL：優先讀取環境變數；未設時從專案根目錄 .env 載入
 *   （E2E_BASE_URL、E2E_ADMIN_EMAIL、E2E_ADMIN_PASSWORD、ADMIN_INITIAL_PASSWORD）
 */
const authDir = path.join(__dirname, 'e2e', '.auth')

export default defineConfig({
    testDir: './e2e',
    fullyParallel: false,
    retries: process.env.CI ? 2 : 0,
    workers: 1,
    timeout: 30_000,

    reporter: process.env.CI
        ? [['github'], ['html', { open: 'never' }]]
        : 'html',

    use: {
        baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
        screenshot: 'only-on-failure',
        trace: 'on-first-retry',
    },

    projects: [
        // Auth setup：登入並儲存 cookie state（429 時需重試等待，給足 timeout）
        {
            name: 'auth-setup',
            testMatch: /auth\.setup\.ts/,
            timeout: 60_000,
        },

        // 主要瀏覽器（admin 相關 spec 使用 fixtures/admin-context 共用同一 context，只登入一次）
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
            dependencies: ['auth-setup'],
            testIgnore: [/auth\.setup\.ts/, /login\.spec\.ts/, /auth-refresh\.spec\.ts/],
        },
        {
            name: 'chromium-login',
            use: { ...devices['Desktop Chrome'] },
            dependencies: ['chromium'],
            testMatch: /login\.spec\.ts/,
        },
        {
            name: 'firefox',
            use: {
                ...devices['Desktop Firefox'],
                storageState: path.join(authDir, 'user.json'),
            },
            dependencies: ['auth-setup'],
            testIgnore: [/auth\.setup\.ts/, /login\.spec\.ts/],
        },
        {
            name: 'firefox-login',
            use: { ...devices['Desktop Firefox'] },
            dependencies: ['firefox'],
            testMatch: /login\.spec\.ts/,
        },
        {
            name: 'webkit',
            use: {
                ...devices['Desktop Safari'],
                storageState: path.join(authDir, 'user.json'),
            },
            dependencies: ['auth-setup'],
            testIgnore: [/auth\.setup\.ts/, /login\.spec\.ts/],
        },
        {
            name: 'webkit-login',
            use: { ...devices['Desktop Safari'] },
            dependencies: ['webkit'],
            testMatch: /login\.spec\.ts/,
        },
    ],
})
