/**
 * 審查者分組鍵：系統內審查者用 reviewer_id、院外審查者（補登）用 reviewer_name 派生。
 * 與後端 `services/protocol/review.rs` 的 reviewer_key 對齊，讓前端分組 / 補登還原一致。
 */
export function reviewerKey(c: {
  reviewer_id: string | null
  reviewer_name?: string | null
}): string {
  return c.reviewer_id ?? `name:${c.reviewer_name ?? ''}`
}
