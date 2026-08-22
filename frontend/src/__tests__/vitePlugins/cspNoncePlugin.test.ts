// R31-8 Vite plugin 單元測試：驗證 CSP nonce placeholder 正確注入
// `<script>` 與 `<link rel="modulepreload">`，已含 nonce 的不重複注入。

import { describe, it, expect } from 'vitest'

import { cspNoncePlugin, CSP_NONCE_PLACEHOLDER } from '../../vitePlugins/cspNoncePlugin'

// 直接呼叫 plugin 的 transformIndexHtml handler。
// Vite plugin transformIndexHtml 可為 object form `{ order, handler }`。
async function runTransform(html: string): Promise<string> {
  const plugin = cspNoncePlugin()
  const hook = plugin.transformIndexHtml
  if (!hook || typeof hook !== 'object' || typeof hook.handler !== 'function') {
    throw new Error('cspNoncePlugin transformIndexHtml shape changed')
  }
  // 第二參數 ctx 在本 plugin 用不到，傳 minimal 物件即可
  const result = await hook.handler.call(
    {} as never,
    html,
    { path: '/index.html', filename: 'index.html', server: undefined } as never,
  )
  if (typeof result === 'string') return result
  if (result && typeof result === 'object' && 'html' in result && typeof result.html === 'string') {
    return result.html
  }
  throw new Error('plugin returned unexpected shape')
}

describe('cspNoncePlugin', () => {
  it('注入 nonce placeholder 到 <script src="...">', async () => {
    const html = '<script src="/main.js"></script>'
    const out = await runTransform(html)
    expect(out).toBe(`<script nonce="${CSP_NONCE_PLACEHOLDER}" src="/main.js"></script>`)
  })

  it('注入 nonce placeholder 到 inline <script>', async () => {
    const html = '<script>console.log("hi")</script>'
    const out = await runTransform(html)
    expect(out).toBe(`<script nonce="${CSP_NONCE_PLACEHOLDER}">console.log("hi")</script>`)
  })

  it('注入 nonce placeholder 到 JSON-LD <script type="application/ld+json">', async () => {
    const html = '<script type="application/ld+json">{"@context":"https://schema.org"}</script>'
    const out = await runTransform(html)
    expect(out).toContain(`<script nonce="${CSP_NONCE_PLACEHOLDER}" type="application/ld+json">`)
  })

  it('注入 nonce placeholder 到 <link rel="modulepreload">', async () => {
    const html = '<link rel="modulepreload" href="/chunk.js">'
    const out = await runTransform(html)
    expect(out).toBe(`<link nonce="${CSP_NONCE_PLACEHOLDER}" rel="modulepreload" href="/chunk.js">`)
  })

  it('不對既有 nonce 的 tag 重複注入', async () => {
    const html = '<script nonce="existing" src="/x.js"></script>'
    const out = await runTransform(html)
    expect(out).toBe(html)
  })

  it('不影響其他 <link>（如 stylesheet / icon）', async () => {
    const html = '<link rel="stylesheet" href="/style.css"><link rel="icon" href="/f.ico">'
    const out = await runTransform(html)
    expect(out).toBe(html)
  })

  it('多個 script + modulepreload 全部注入', async () => {
    const html = [
      '<script type="module" src="/a.js"></script>',
      '<link rel="modulepreload" href="/b.js">',
      '<script>inline()</script>',
    ].join('\n')
    const out = await runTransform(html)
    const matches = out.match(new RegExp(`nonce="${CSP_NONCE_PLACEHOLDER}"`, 'g'))
    expect(matches).toHaveLength(3)
  })
})
