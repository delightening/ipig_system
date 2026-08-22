/**
 * 將一段 body HTML 開到新分頁並觸發瀏覽器列印（純前端列印）。
 *
 * 關鍵：空白新分頁沒有 app 的 Tailwind / design token CSS，故把當前文件的所有
 * `<style>` 與 `<link rel="stylesheet">` 複製過去，新分頁才有樣式。
 *
 * CSP 安全：列印由 opener 端在新分頁 `load` 後呼叫 `win.print()`，**不在新文件內嵌
 * 任何 inline `<script>`/`onload`**，避免撞 strict CSP（`script-src` 無 `unsafe-inline`）。
 */
export function openPrintWindow(opts: { bodyHtml: string; title: string }): void {
  const { bodyHtml, title } = opts
  const win = window.open('', '_blank')
  if (!win) {
    throw new Error('彈出視窗被阻擋；請允許後重試')
  }

  // 複製當前文件的所有 stylesheet（link + style）→ 新分頁套用相同樣式
  const styleTags = Array.from(
    document.querySelectorAll('link[rel="stylesheet"], style'),
  )
    .map((el) => el.outerHTML)
    .join('\n')

  const doc = win.document
  doc.open()
  doc.write(
    `<!DOCTYPE html><html lang="zh-Hant"><head><meta charset="utf-8" />` +
      `<title>${escapeHtml(title)}</title>${styleTags}` +
      `<style>body{margin:0;background:#fff}@media print{@page{margin:12mm}}</style>` +
      `</head><body>${bodyHtml}</body></html>`,
  )
  doc.close()

  let printed = false
  const triggerPrint = () => {
    if (printed) return
    printed = true
    win.focus()
    win.print()
  }
  // window `load` 會等 stylesheet（含 <link>）載入完成才觸發 → 確保有樣式才列印
  win.addEventListener('load', triggerPrint, { once: true })
  // fallback：load 已錯過（同步寫入、stylesheet 全來自快取）時，1.5s 後仍列印
  window.setTimeout(triggerPrint, 1500)
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"]/g, (c) =>
    c === '&' ? '&amp;' : c === '<' ? '&lt;' : c === '>' ? '&gt;' : '&quot;',
  )
}
