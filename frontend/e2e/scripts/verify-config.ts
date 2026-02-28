/**
 * E2E 配置驗證腳本
 *
 * 用法：
 *   npx tsx frontend/e2e/scripts/verify-config.ts
 *
 * 檢查項目：
 * - JWT_EXPIRATION_MINUTES >= 5
 * - COOKIE_SECURE=false（本機開發）
 * - ADMIN_INITIAL_PASSWORD 已設定
 * - E2E_BASE_URL 正確
 */

import dotenv from 'dotenv'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// 從專案根目錄載入 .env
dotenv.config({ path: path.resolve(__dirname, '../../../.env') })

interface CheckResult {
    name: string
    status: 'PASS' | 'WARN' | 'FAIL'
    message: string
}

const results: CheckResult[] = []

/**
 * 執行檢查並記錄結果
 */
function check(
    name: string,
    condition: boolean,
    passMsg: string,
    failMsg: string,
    critical = true,
) {
    results.push({
        name,
        status: condition ? 'PASS' : critical ? 'FAIL' : 'WARN',
        message: condition ? passMsg : failMsg,
    })
}

// ============================================
// 檢查項目
// ============================================

console.log('\n🔍 開始 E2E 配置驗證...\n')

// 1. JWT TTL
const jwtMinutes = parseInt(process.env.JWT_EXPIRATION_MINUTES || '360', 10)
check(
    'JWT TTL',
    jwtMinutes >= 5,
    `JWT_EXPIRATION_MINUTES = ${jwtMinutes} 分鐘（足夠）`,
    `JWT_EXPIRATION_MINUTES = ${jwtMinutes} 分鐘（建議 >= 5 分鐘）`,
    false,
)

// 2. Cookie Secure
const cookieSecure = process.env.COOKIE_SECURE?.toLowerCase()
check(
    'Cookie Secure',
    cookieSecure === 'false' || cookieSecure === undefined,
    'COOKIE_SECURE=false（正確，適用於本機 http）',
    `COOKIE_SECURE=${process.env.COOKIE_SECURE || '(未設定)'}（本機開發應為 false）`,
    false,
)

// 3. Admin Password
const adminPassword = process.env.ADMIN_INITIAL_PASSWORD
check(
    'Admin Password',
    !!adminPassword && adminPassword.length >= 6,
    'ADMIN_INITIAL_PASSWORD 已設定',
    'ADMIN_INITIAL_PASSWORD 未設定或過短（需 >= 6 字元）',
    true,
)

// 4. E2E Base URL
const e2eUrl = process.env.E2E_BASE_URL || 'http://localhost:8080'
const isLocalhost =
    e2eUrl.includes('localhost') || e2eUrl.includes('127.0.0.1')
check(
    'E2E Base URL',
    isLocalhost,
    `E2E_BASE_URL = ${e2eUrl}（正確）`,
    `E2E_BASE_URL = ${e2eUrl}（請確認是否為本機環境）`,
    false,
)

// 5. Cookie Domain（建議不設定）
const cookieDomain = process.env.COOKIE_DOMAIN
check(
    'Cookie Domain',
    !cookieDomain,
    'COOKIE_DOMAIN 未設定（正確，使用請求 host）',
    `COOKIE_DOMAIN=${cookieDomain}（建議本機不設定）`,
    false,
)

// 6. E2E Admin Email
const adminEmail =
    process.env.E2E_ADMIN_EMAIL ||
    process.env.ADMIN_INITIAL_EMAIL ||
    'admin@ipig.local'
check(
    'E2E Admin Email',
    !!adminEmail,
    `E2E_ADMIN_EMAIL = ${adminEmail}`,
    'E2E_ADMIN_EMAIL 未設定',
    false,
)

// ============================================
// 輸出結果
// ============================================

console.log('='.repeat(60))
console.log('  E2E 配置驗證結果')
console.log('='.repeat(60))
console.log()

let hasFail = false
let hasWarn = false

results.forEach((r) => {
    const icon = r.status === 'PASS' ? '✅' : r.status === 'WARN' ? '⚠️ ' : '❌'
    console.log(`${icon} ${r.name}`)
    console.log(`   ${r.message}`)
    console.log()

    if (r.status === 'FAIL') hasFail = true
    if (r.status === 'WARN') hasWarn = true
})

console.log('='.repeat(60))
console.log()

// ============================================
// 最終結果
// ============================================

if (hasFail) {
    console.error('❌ 配置驗證失敗，請修正上述 FAIL 項目')
    console.error()
    console.error('📚 參考文檔：docs/e2e/README.md')
    console.error()
    process.exit(1)
} else if (hasWarn) {
    console.log('⚠️  配置驗證通過，但有警告項目')
    console.log()
    console.log('建議檢查上述 WARN 項目，確保符合預期')
    console.log()
    console.log('📚 參考文檔：docs/e2e/README.md')
    console.log()
    process.exit(0)
} else {
    console.log('✅ 配置驗證完全通過')
    console.log()
    console.log('現在可以執行 E2E 測試：')
    console.log('  npm run test:e2e')
    console.log()
    process.exit(0)
}
