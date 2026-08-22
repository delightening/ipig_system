// CSP cutover smoke test — opens prod URL in 3 Playwright engines (Chromium /
// Firefox / WebKit, covering Chrome/Edge/Safari/Firefox SOP requirement) and
// reports any Content-Security-Policy console messages.
//
// 模式：
//   default: 測試現行 prod headers
//   SIMULATE_CUTOVER=1: 用 route interception 把 CSP header 改為「cutover 後的內容」
//                       來模擬 enforce 行為（dry-run，不需動 prod nginx）
//
// 用法：
//   node scripts/csp-smoke.mjs
//   PROD_URL=https://ipigsystem.asia SIMULATE_CUTOVER=1 node scripts/csp-smoke.mjs
//
// 不需登入；只跑 public landing page（cutover 主要風險是 landing inline / CDN）。
//
// Exit code: 0 = 全 3 engines 乾淨；1 = 任一 engine 有違規。

import { chromium, firefox, webkit } from 'playwright'

const URL = process.env.PROD_URL || 'https://ipigsystem.asia/'
const SIMULATE_CUTOVER = process.env.SIMULATE_CUTOVER === '1'
const ENGINES = [
  { name: 'Chromium (Chrome/Edge)', fn: chromium },
  { name: 'Firefox', fn: firefox },
  { name: 'WebKit (Safari)', fn: webkit },
]

const CSP_RE = /content security policy|refused to (load|execute|apply|evaluate)/i

// Cutover-後的目標 CSP（對齊 R31-10 / R31-C 的 security-headers.conf）
// nonce 由 nginx sub_filter 注入，這裡保留 page 自帶的（route handler 只改 header
// 本身不影響頁面 HTML 內已替換的 nonce 值）
//
// ⚠️ 維護提醒（Gemini PR #410 review）：以下字串硬編碼是 nginx config 的 mirror，
// 修 `frontend/security-headers.conf` 的同時必須同步修這裡，否則 dry-run 失準。
// 沒做 runtime 讀 conf 是刻意決策：smoke 是 cutover gate，要的是「**目標** CSP
// 對 prod **HTML** 的效果」，不是「現行 prod CSP 自我驗證」。
function buildCutoverCsp(originalHeaders) {
  // 嘗試從現行 Report-Only header 抽出 nonce — page 已用該 nonce 簽 script
  const ro = originalHeaders['content-security-policy-report-only'] || ''
  const nonceMatch = ro.match(/'nonce-([a-f0-9]+)'/)
  const nonce = nonceMatch ? nonceMatch[1] : 'simulated-cutover-nonce'
  return [
    "default-src 'self'",
    `script-src 'self' 'nonce-${nonce}' 'strict-dynamic' 'wasm-unsafe-eval'`,
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com data:",
    "img-src 'self' data: blob:",
    "connect-src 'self'",
    "frame-src 'self' blob:",
    "frame-ancestors 'none'",
    'report-uri /api/v1/csp-report',
    'report-to csp-endpoint',
  ].join('; ')
}

async function smokeOne({ name, fn }) {
  const browser = await fn.launch({ headless: true })
  const context = await browser.newContext({ ignoreHTTPSErrors: false })
  const page = await context.newPage()

  const cspMessages = []
  page.on('console', (msg) => {
    const text = msg.text()
    if (CSP_RE.test(text)) {
      cspMessages.push({ type: msg.type(), text })
    }
  })
  page.on('pageerror', (err) => {
    if (CSP_RE.test(String(err))) {
      cspMessages.push({ type: 'pageerror', text: String(err) })
    }
  })

  if (SIMULATE_CUTOVER) {
    // Route interception: 對主 HTML 文件回應改寫 CSP header 為 enforce 版
    await page.route('**/*', async (route, request) => {
      const isMainDoc = request.resourceType() === 'document'
      const response = await route.fetch()
      if (isMainDoc) {
        const original = response.headersArray()
        const originalMap = {}
        original.forEach((h) => {
          originalMap[h.name.toLowerCase()] = h.value
        })
        const newCsp = buildCutoverCsp(originalMap)
        const newHeaders = original
          .filter((h) => {
            const n = h.name.toLowerCase()
            return n !== 'content-security-policy' && n !== 'content-security-policy-report-only'
          })
          .concat([{ name: 'Content-Security-Policy', value: newCsp }])
        await route.fulfill({
          status: response.status(),
          headers: Object.fromEntries(newHeaders.map((h) => [h.name, h.value])),
          body: await response.body(),
        })
      } else {
        await route.continue()
      }
    })
  }

  try {
    await page.goto(URL, { waitUntil: 'networkidle', timeout: 30000 })
    await page.waitForTimeout(2000)
  } catch (e) {
    // Gemini PR #410 review：nav 失敗（404 / connection refused / cert error 等）
    // 推回 cspMessages 視為一筆 violation，main() 的 totalViolations > 0 → exit 1，
    // 防止 CI 把 silent navigation failure 視為「測試通過」的假象。
    console.error(`[${name}] navigation error: ${e.message}`)
    cspMessages.push({ type: 'navigation-error', text: `Navigation failed: ${e.message}` })
  } finally {
    await browser.close()
  }
  return cspMessages
}

async function main() {
  const mode = SIMULATE_CUTOVER ? 'SIMULATE_CUTOVER (dry-run)' : 'current prod headers'
  console.log(`\nCSP smoke test against ${URL} — mode: ${mode}\n${'='.repeat(60)}`)
  let totalViolations = 0
  for (const engine of ENGINES) {
    const msgs = await smokeOne(engine)
    console.log(`\n[${engine.name}] ${msgs.length} CSP message(s)`)
    msgs.forEach((m, i) => console.log(`  ${i + 1}. [${m.type}] ${m.text}`))
    totalViolations += msgs.length
  }
  console.log(`\n${'='.repeat(60)}`)
  console.log(`Total: ${totalViolations} violation(s) across 3 engines`)
  process.exit(totalViolations === 0 ? 0 : 1)
}

main().catch((e) => {
  console.error(e)
  process.exit(2)
})
