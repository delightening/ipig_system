import { defineConfig, devices } from '@playwright/test'

/**
 * Playwright E2E 測試設定
 *
 * 使用方式：
 *   npx playwright test          # 執行所有 E2E 測試
 *   npx playwright test --ui     # 開啟互動式 UI
 *   npx playwright codegen       # 錄製測試
 */
export default defineConfig({
    // 測試目錄
    testDir: './e2e',

    // 完整的重試與超時設定
    fullyParallel: true,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    timeout: 30_000,

    // 報告格式
    reporter: process.env.CI ? 'github' : 'html',

    // 全域設定
    use: {
        // 前端開發伺服器 URL
        baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
        // 截圖（失敗時自動截圖）
        screenshot: 'only-on-failure',
        // 追蹤（失敗時保留）
        trace: 'on-first-retry',
    },

    // 測試的瀏覽器
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],
})
