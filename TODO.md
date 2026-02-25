# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-02-25
> **AI 標註說明：** 
> - ⚡ **Gemini Flash** (適合樣板編寫、簡單設定、文檔生成)
> - 🧠 **Claude Sonnet/Opus** (適合架構設計、複雜邏輯、安全性強化、大規模重構)

---

## ⛔ 禁止事項

1. 密碼過期策略
2. 密碼歷史紀錄（SEC-38：密碼歷史紀錄）

---

## 🚨 P0 — 上線前必要 (Production Readiness)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|

---

## 🟡 P1 — 上線前強烈建議 (Quality & Compliance)

| # | 項目 | 說明 | 範圍 | 依賴 | 建議 AI | 狀態 |
|---|------|------|------|------|----------|------|
| P1-1 | **前端 E2E 測試 (Playwright)** | 7 spec / 34 tests，含 429 重試 + race condition 修正，連續 3 次 0 failures | 前端 | 無 | 🧠 Claude | [x] |
| P1-2 | **E2E CI 自動化** | `docker-compose.test.yml` + GitHub Actions 整合 | DevOps | P1-1 | ⚡ Flash | [x] |
| P1-7 | **電子簽章合規審查** | 21 CFR Part 11 或等效法規合規審查 | 文件 | 無 | 🧠 Claude | [ ] |
| P1-8 | **資料保留政策** | 定義各類紀錄的法定保留年限 | 文件 | 無 | 🧠 Claude | [ ] |
| P1-12 | **OpenAPI 文件完善 (≥90%)** | 擴展其餘端點的 Schema 與 Path 定義 | 後端 | 無 | 🧠 Claude | [ ] |

---

## 🔵 P3 — 低優先 (資安 / 基礎設施)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 7 | **SEC-33：敏感操作二級認證** | 高危操作要求重新輸入密碼確認 | 前後端 | 🧠 Claude | [ ] |

---

## 🟣 P4 — 中期品質提升 (測試 / 文件 / CI)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 13 | **前端元件庫文件化** | Storybook 建置 | 前端 | ⚡ Flash | [ ] |
| 14 | **前端超長頁面重構** | 漸進式重構巨型組件 (如 `ProtocolEditPage`) | 前端 | 🧠 Claude | [/] |
| 15 | **SEC-39：Two-Factor Authentication** | TOTP 二階段驗證 (Google Authenticator) | 前後端 | 🧠 Claude | [ ] |
| 16 | **SEC-40：Web Application Firewall** | ModSecurity 或 Cloudflare WAF | DevOps | ⚡ Flash | [ ] |

---

## 📊 待辦統計

| 優先級 | 數量 (未完成) |
|--------|------|
| 🚨 P0 上線前必要 | 0 |
| 🟡 P1 上線前建議 | 4 |
| 🔴 P2 中優先 | 0 |
| 🔵 P3 低優先 | 1 |
| 🟣 P4 品質提升 | 0 |
| ⚪ P5 長期演進 | 3 |
| **合計（未完成）** | **8** |

---

## 變更紀錄 (最新)

| 日期 | 內容 |
|------|------|
| 2026-02-25 | ⚡ Flash：完成 P1-5 後端壓力測試基準建立 (k6)，已遷移至 PROGRESS.md。 |
| 2026-02-25 | 🧠 Claude：P1-1 E2E 穩定化 — 429 rate limit 重試、React state race condition fallback、連續 3 次 0 failures。 |
| 2026-02-25 | 🧠 Claude：P1-1 Playwright E2E 測試擴充（7 spec, 34 tests, auth setup + 6 critical flows）。 |
| 2026-02-25 | 🧹 整理：將 P0-6, P0-7, P1-6 已完成項目遷移至 `PROGRESS.md`。 |
| 2026-02-25 | ⚡ Flash 任務第二波：完成 P0-6 跨瀏覽器相容性驗證、P1-6 GLP 驗證文件 (IQ/OQ/PQ) 生成。 |
| 2026-02-25 | ⚡ Flash 任務第一波：完成 Brotli、具名隧道腳本、CI/CD DB 整合、操作手冊與 Grafana 分配。 |
| 2026-02-25 | 🏷️ AI 標註：新增建議使用的 AI 模型標註。 |
