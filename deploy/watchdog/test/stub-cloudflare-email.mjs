/**
 * `cloudflare:email` 在 Node 底下不存在，只有 workerd 提供。
 * 這個 stub 讓 worker.js 能被 `node --test` 直接 import。
 *
 * 只需要 EmailMessage 這一個 export——worker.js 用它包裝 MIME 後交給
 * `env.ALERT_EMAIL.send()`，而 `ALERT_EMAIL` 在測試裡本來就是假的。
 */
export class EmailMessage {
  constructor(from, to, raw) {
    this.from = from;
    this.to = to;
    this.raw = raw;
  }
}
