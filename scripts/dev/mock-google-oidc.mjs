#!/usr/bin/env node
// R121：本機模擬用的「假 Google」OIDC 服務。零相依，只綁 127.0.0.1。
//
// 用途：還沒有真 Google OAuth 憑證時，在開發機跑完整的 Google 登入流程。
// 後端需設 GOOGLE_OAUTH_DEV_MOCK_BASE_URL=http://127.0.0.1:<PORT>（僅 COOKIE_SECURE=false 時允許，
// 見 backend/src/config/google_oauth.rs）。
//
// 🔴 這不是真的 Google，也不做任何身分驗證——畫面上點哪個帳號就是哪個帳號。只能在本機用。
//
// 行為（對齊 Google 的 Authorization Code + PKCE）：
//   GET  /o/oauth2/v2/auth   帳號選擇頁（驗 client_id / response_type / S256）
//   POST /o/oauth2/v2/auth   使用者選了帳號或按取消 → 302 回 redirect_uri
//   POST /token              授權碼換 ID token（驗 client secret、redirect_uri、PKCE；授權碼一次性、60 秒）
//
// 環境變數：MOCK_GOOGLE_PORT（預設 9099）、MOCK_GOOGLE_CLIENT_ID、MOCK_GOOGLE_CLIENT_SECRET。

import http from 'node:http'
import { createHash, randomBytes } from 'node:crypto'

const PORT = Number(process.env.MOCK_GOOGLE_PORT ?? 9099)
const HOST = '127.0.0.1'
const ISSUER = `http://${HOST}:${PORT}`
const CLIENT_ID = process.env.MOCK_GOOGLE_CLIENT_ID ?? 'mock-client-id.apps.googleusercontent.com'
const CLIENT_SECRET = process.env.MOCK_GOOGLE_CLIENT_SECRET ?? 'mock-client-secret'
const CODE_TTL_MS = 60_000

/** 預設可選的假帳號；也可在畫面上手動輸入任意 email。 */
const ACCOUNTS = [
  { sub: 'mock-sub-alice', email: 'alice.google@example.com', verified: true, note: '已有系統帳號（會自動綁定）' },
  { sub: 'mock-sub-nobody', email: 'nobody@example.com', verified: true, note: '沒有系統帳號（應被拒）' },
  { sub: 'mock-sub-unverified', email: 'alice.google@example.com', verified: false, note: 'email 未驗證（應被拒）' },
  { sub: 'mock-sub-intruder', email: 'alice.google@example.com', verified: true, note: '同 email 的第二個 Google 帳號（應被拒）' },
]

/** code → { clientId, redirectUri, nonce, codeChallenge, account, expiresAt } */
const codes = new Map()

const b64url = (buf) => Buffer.from(buf).toString('base64url')

const escapeHtml = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c])

function send(res, status, body, headers = {}) {
  res.writeHead(status, { 'Cache-Control': 'no-store', ...headers })
  res.end(body)
}

function sendJson(res, status, obj) {
  send(res, status, JSON.stringify(obj), { 'Content-Type': 'application/json' })
}

function redirect(res, location) {
  send(res, 302, '', { Location: location })
}

async function readForm(req) {
  const chunks = []
  for await (const chunk of req) chunks.push(chunk)
  return new URLSearchParams(Buffer.concat(chunks).toString('utf8'))
}

function withQuery(base, params) {
  const url = new URL(base)
  for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
  return url.toString()
}

function validateAuthRequest(params) {
  if (params.get('client_id') !== CLIENT_ID) return 'client_id 不符'
  if (params.get('response_type') !== 'code') return 'response_type 必須是 code'
  if (params.get('code_challenge_method') !== 'S256') return '必須使用 PKCE S256'
  if (!params.get('code_challenge')) return '缺少 code_challenge'
  if (!params.get('state')) return '缺少 state'
  if (!params.get('nonce')) return '缺少 nonce'
  const redirectUri = params.get('redirect_uri') ?? ''
  let parsed
  try {
    parsed = new URL(redirectUri)
  } catch {
    return 'redirect_uri 不是合法網址'
  }
  if (!['localhost', '127.0.0.1'].includes(parsed.hostname)) return 'redirect_uri 只能是本機'
  return null
}

const HIDDEN_KEYS = ['client_id', 'redirect_uri', 'state', 'nonce', 'code_challenge', 'code_challenge_method', 'response_type']

function hiddenFields(params) {
  return HIDDEN_KEYS.map(
    (k) => `<input type="hidden" name="${k}" value="${escapeHtml(params.get(k) ?? '')}">`,
  ).join('')
}

function chooserPage(params) {
  const hidden = hiddenFields(params)
  const rows = ACCOUNTS.map(
    (a) => `
      <form method="post" class="acct">
        ${hidden}
        <input type="hidden" name="sub" value="${escapeHtml(a.sub)}">
        <input type="hidden" name="email" value="${escapeHtml(a.email)}">
        <input type="hidden" name="verified" value="${a.verified}">
        <button type="submit" data-sub="${escapeHtml(a.sub)}">
          <strong>${escapeHtml(a.email)}</strong>
          <span>${a.verified ? '已驗證' : '未驗證'} · sub=${escapeHtml(a.sub)}</span>
          <em>${escapeHtml(a.note)}</em>
        </button>
      </form>`,
  ).join('')
  return `<!doctype html>
<html lang="zh-Hant"><head><meta charset="utf-8"><title>假 Google 登入（本機模擬）</title>
<style>
  body{font-family:system-ui,sans-serif;max-width:560px;margin:40px auto;padding:0 16px}
  .warn{background:#fff4e5;border:1px solid #f0a020;padding:10px 12px;border-radius:6px}
  .acct button{display:block;width:100%;text-align:left;margin:8px 0;padding:10px 12px;border:1px solid #ccc;border-radius:6px;background:#fff;cursor:pointer}
  .acct button span,.acct button em{display:block;font-size:12px;color:#555}
  fieldset{margin-top:16px;border:1px solid #ccc;border-radius:6px}
</style></head><body>
<h1>假 Google 登入</h1>
<p class="warn">⚠️ 這是本機模擬用的假 Google，不是真的 Google。點哪個帳號就以該帳號身分回到系統。</p>
<h2>選擇帳號</h2>
${rows}
<fieldset><legend>自訂帳號</legend>
  <form method="post">
    ${hidden}
    <label>email <input name="email" required placeholder="someone@example.com"></label><br>
    <label>sub <input name="sub" required placeholder="mock-sub-xxx"></label><br>
    <label><input type="checkbox" name="verified" value="true" checked> email 已驗證</label><br>
    <button type="submit">以此帳號登入</button>
  </form>
</fieldset>
<form method="post" style="margin-top:16px">
  ${hidden}
  <input type="hidden" name="cancel" value="1">
  <button type="submit" id="cancel">取消</button>
</form>
</body></html>`
}

function handleAuthorizeGet(res, url) {
  const error = validateAuthRequest(url.searchParams)
  if (error) return send(res, 400, `invalid_request: ${error}`, { 'Content-Type': 'text/plain; charset=utf-8' })
  send(res, 200, chooserPage(url.searchParams), { 'Content-Type': 'text/html; charset=utf-8' })
}

async function handleAuthorizePost(req, res) {
  const form = await readForm(req)
  const error = validateAuthRequest(form)
  if (error) return send(res, 400, `invalid_request: ${error}`, { 'Content-Type': 'text/plain; charset=utf-8' })

  const redirectUri = form.get('redirect_uri')
  const state = form.get('state')
  if (form.get('cancel')) return redirect(res, withQuery(redirectUri, { error: 'access_denied', state }))

  const account = {
    sub: form.get('sub') ?? '',
    email: form.get('email') ?? '',
    verified: form.get('verified') === 'true',
  }
  if (!account.sub || !account.email) return send(res, 400, 'sub 與 email 必填')

  const code = b64url(randomBytes(24))
  codes.set(code, {
    clientId: form.get('client_id'),
    redirectUri,
    nonce: form.get('nonce'),
    codeChallenge: form.get('code_challenge'),
    account,
    expiresAt: Date.now() + CODE_TTL_MS,
  })
  console.log(`[mock-google] 發出授權碼 → ${account.email}（sub=${account.sub}, verified=${account.verified}）`)
  redirect(res, withQuery(redirectUri, { code, state }))
}

function idTokenFor(entry) {
  const now = Math.floor(Date.now() / 1000)
  const header = b64url(JSON.stringify({ alg: 'RS256', typ: 'JWT', kid: 'mock' }))
  const payload = b64url(
    JSON.stringify({
      iss: ISSUER,
      aud: entry.clientId,
      azp: entry.clientId,
      sub: entry.account.sub,
      email: entry.account.email,
      email_verified: entry.account.verified,
      nonce: entry.nonce,
      iat: now,
      exp: now + 3600,
    }),
  )
  // 後端依 OIDC Core §3.1.3.7(6) 不驗簽章（token 由後端直連 token endpoint 取得），此處放占位字串
  return `${header}.${payload}.mock-signature`
}

async function handleToken(req, res) {
  const form = await readForm(req)
  const fail = (why) => {
    console.warn(`[mock-google] token 交換拒絕：${why}`)
    sendJson(res, 400, { error: 'invalid_grant', error_description: why })
  }
  if (form.get('grant_type') !== 'authorization_code') return fail('grant_type 不符')
  if (form.get('client_id') !== CLIENT_ID || form.get('client_secret') !== CLIENT_SECRET)
    return fail('client 認證失敗')

  const code = form.get('code') ?? ''
  const entry = codes.get(code)
  codes.delete(code) // 一次性：不論成敗都作廢
  if (!entry) return fail('授權碼不存在或已使用')
  if (entry.expiresAt < Date.now()) return fail('授權碼已過期')
  if (entry.redirectUri !== form.get('redirect_uri')) return fail('redirect_uri 與授權時不同')

  const verifier = form.get('code_verifier') ?? ''
  const challenge = b64url(createHash('sha256').update(verifier).digest())
  if (challenge !== entry.codeChallenge) return fail('PKCE code_verifier 不符')

  console.log(`[mock-google] 簽發 ID token → ${entry.account.email}`)
  sendJson(res, 200, {
    access_token: 'mock-access-token',
    token_type: 'Bearer',
    expires_in: 3600,
    scope: 'openid email',
    id_token: idTokenFor(entry),
  })
}

const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? '/', ISSUER)
    if (url.pathname === '/o/oauth2/v2/auth' && req.method === 'GET') return handleAuthorizeGet(res, url)
    if (url.pathname === '/o/oauth2/v2/auth' && req.method === 'POST') return await handleAuthorizePost(req, res)
    if (url.pathname === '/token' && req.method === 'POST') return await handleToken(req, res)
    if (url.pathname === '/healthz') return send(res, 200, 'ok')
    send(res, 404, 'not found')
  } catch (err) {
    console.error('[mock-google] 內部錯誤', err)
    send(res, 500, 'internal error')
  }
})

server.listen(PORT, HOST, () => {
  console.log(`[mock-google] 假 Google 已啟動：${ISSUER}（client_id=${CLIENT_ID}）`)
})
