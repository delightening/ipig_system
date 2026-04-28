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
 *
 * 跨瀏覽器：預設僅跑 Chromium（避免 session 過期與瀏覽器未安裝問題）。
 * 啟用 Firefox/WebKit：設定 PLAYWRIGHT_FIREFOX=1、PLAYWRIGHT_WEBKIT=1
 */
const authDir = path.join(__dirname, 'e2e', '.auth')

/**
 * Firefox/WebKit 預設為 opt-in（需明確設定環境變數才啟用）。
 * 原因：(1) workers=1 序列執行 100 tests 耗時約 2 分鐘，storageState 中的 JWT
 * 往往已接近過期，導致後執行的瀏覽器 session 失效；(2) Windows 可能未安裝。
 */
const runFirefox = process.env.PLAYWRIGHT_FIREFOX === '1'
const runWebKit = process.env.PLAYWRIGHT_WEBKIT === '1'
const collectCoverage = process.env.E2E_COVERAGE === '1'
const baseURL = process.env.E2E_BASE_URL || 'http://localhost:8080'

// 從 baseURL 解析 origin，避免 entryFilter 寫死 host 在 BASE_URL 被覆寫時靜默漏接 coverage
const baseOrigin = (() => {
    try {
        return new URL(baseURL).origin
    } catch {
        return baseURL
    }
})()

// monocart-reporter：聚合 V8 coverage entries（fixtures/coverage.ts 寫入），
// 透過 hidden source map 還原到 src/*.ts，輸出 lcov + html。
const coverageReporter: any[] = collectCoverage
    ? [
          [
              'monocart-reporter',
              {
                  name: 'E2E Coverage',
                  outputFile: './monocart-report/index.html',
                  coverage: {
                      entryFilter: (entry: { url: string }) =>
                          entry.url.startsWith(baseOrigin) &&
                          !entry.url.includes('node_modules'),
                      sourceFilter: (sourcePath: string) => sourcePath.search(/src\//) !== -1,
                      reports: [['lcovonly', { file: 'lcov.info' }], ['v8'], ['console-summary']],
                      outputDir: './monocart-report/coverage',
                  },
              },
          ],
      ]
    : []

export default defineConfig({
    testDir: './e2e',
    fullyParallel: false,
    retries: process.env.CI ? 2 : 1,
    workers: 1,
    timeout: 30_000,

    reporter: process.env.CI
        ? [['github'] as any, ['html', { open: 'never' }] as any, ...coverageReporter]
        : collectCoverage
          ? [['html'] as any, ...coverageReporter]
          : 'html',

    use: {
        baseURL,
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
        ...(runFirefox
            ? [
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
              ]
            : []),
        ...(runWebKit
            ? [
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
              ]
            : []),
    ],
})
