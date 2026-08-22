// R31-8：CSP nonce placeholder 注入 Vite plugin
//
// 目的：nginx `sub_filter` 只能改 HTML 層的靜態 tag，**抓不到** Vite code-split
// 後 `import()` runtime 動態建立的 `<script>`，但可以抓 build 時就寫進 index.html
// 的 `<script>` 與 `<link rel="modulepreload">`。
//
// 方案：build 時在所有 `<script>` 與 `<link rel="modulepreload">` 注入
// `nonce="__CSP_NONCE__"`，nginx 在送出 response 時用 `sub_filter __CSP_NONCE__ $cspNonce`
// 替換為 per-request unique nonce。動態 import 的 chunk 由 `'strict-dynamic'`
// 讓 nonce script 信任傳遞 — 不需另外注入。
//
// CSP 切換階段：placeholder 寫進 prod build，但只有 nginx 在 Report-Only header
// 設定 nonce 時才會被信任；舊 enforce header 仍含 'unsafe-inline'，安全可逆。

import type { Plugin } from 'vite'

export const CSP_NONCE_PLACEHOLDER = '__CSP_NONCE__'

// 比對 `<script ...>`（任何屬性，含 type=module / type=application/ld+json）
// 與 `<link ... rel="modulepreload" ...>`。已含 nonce 的不重複注入。
const SCRIPT_RE = /<script(?![^>]*\snonce=)([^>]*)>/gi
const MODULEPRELOAD_RE = /<link(?![^>]*\snonce=)([^>]*\srel=["']?modulepreload["']?[^>]*)>/gi

export function injectNoncePlaceholder(html: string): string {
  return html
    .replace(SCRIPT_RE, `<script nonce="${CSP_NONCE_PLACEHOLDER}"$1>`)
    .replace(MODULEPRELOAD_RE, `<link nonce="${CSP_NONCE_PLACEHOLDER}"$1>`)
}

export function cspNoncePlugin(): Plugin {
  return {
    name: 'csp-nonce-placeholder',
    // `enforce: 'post'` 確保在 Vite 內建 build-html plugin（注入 modulepreload / script）
    // 之後執行，否則 build 時生成的 tag 會錯過注入。
    enforce: 'post',
    transformIndexHtml: {
      order: 'post',
      handler(html) {
        return injectNoncePlaceholder(html)
      },
    },
  }
}
