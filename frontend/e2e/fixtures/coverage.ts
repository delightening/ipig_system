/**
 * E2E coverage 自動 fixture
 *
 * E2E_COVERAGE=1 時對每個 test 啟動 Playwright V8 JS coverage，
 * 結束時把原始 coverage entries 交給 monocart-reporter 聚合（lcov + html）。
 *
 * 僅 chromium 專案啟用：V8 coverage API 是 chromium-only。
 *
 * 使用方式：spec 或其他 fixture base 改從這裡 import { test, expect }。
 */
import base, { expect } from '@playwright/test'
import { addCoverageReport } from 'monocart-reporter'

const coverageEnabled = process.env.E2E_COVERAGE === '1'

export const test = base.extend<{ coverageAuto: void }>({
    coverageAuto: [
        async ({ page, browserName }, use, testInfo) => {
            const enabled = coverageEnabled && browserName === 'chromium'
            if (enabled) {
                await page.coverage.startJSCoverage({ resetOnNavigation: false })
            }
            await use()
            if (enabled) {
                const entries = await page.coverage.stopJSCoverage()
                await addCoverageReport(entries, testInfo)
            }
        },
        { auto: true },
    ],
})

export { expect }
