# 第七輪改善計劃 (R7) — 2026-03-08 全面審視

> **產出者：** Claude Opus 4.6
> **基準：** 專案整體完成度 100%、TODO.md 未完成 4 項、6 輪改善已執行
> **方法：** 全面原始碼審視 (208 Rust 檔 / 57K 行、284 TS/TSX 檔 / 69K 行)、架構分析、CI/CD 審計、依賴評估

---

## 目錄

| # | 區塊 | 重點 |
|---|------|------|
| 1 | [現狀摘要](#1-現狀摘要) | 專案整體健康度評估 |
| 2 | [P0 — 上線阻擋項](#2-p0--上線阻擋項) | 立即處理 |
| 3 | [P1 — 程式碼品質與可維護性](#3-p1--程式碼品質與可維護性) | 短期改善 |
| 4 | [P2 — 架構與效能](#4-p2--架構與效能) | 中期優化 |
| 5 | [P3 — 測試與可靠性](#5-p3--測試與可靠性) | 測試覆蓋與穩定性 |
| 6 | [P4 — 安全強化](#6-p4--安全強化) | 安全縱深防禦 |
| 7 | [P5 — 開發體驗與 CI/CD](#7-p5--開發體驗與cicd) | 工程效率 |
| 8 | [P6 — 長期演進](#8-p6--長期演進) | 技術債清償與未來規劃 |
| 9 | [延續項目 (TODO.md 未完成)](#9-延續項目) | 既有待辦的處理建議 |

---

## 1. 現狀摘要

### 優勢

- **零 `unwrap()`**：整個後端 208 個 Rust 檔案中無任何 `unwrap()` 調用，錯誤處理品質優良
- **零 `any` 型別**：前端 284 個 TS/TSX 檔案中已完全消除 `: any` 型別，TypeScript 嚴格模式維護良好
- **完整的 CI 防護**：cargo audit、cargo deny、SQL injection guard、unsafe guard、Trivy 容器掃描、npm audit
- **成熟的 Docker 部署**：三層網路隔離、Docker Secrets、healthcheck、資源限制、GPG 加密備份
- **全面的 OpenAPI 文件**：utoipa 產生 Swagger UI，R4 已達 100% 端點覆蓋
- **GLP 合規基礎**：電子簽章、稽核日誌 HMAC 完整性、資料保留政策、21 CFR Part 11 評估

### 需關注項目

| 面向 | 現況 | 風險等級 |
|------|------|----------|
| 後端 `expect()` | 74 處 | 中（服務穩定性） |
| 後端 `.clone()` | 363 處 | 低（效能微調） |
| 巨型檔案 | 後端 5 檔 >1000 行、前端 5 檔 >900 行 | 中（可維護性） |
| CI 觸發 | push/PR 觸發已註解 `#取消`，僅 `workflow_dispatch` | 高（品質閘門失效） |
| Migration 數量 | 僅 10 個，但 PROGRESS.md 提到 020、021、022 等 | 中（遷移腳本可能未追蹤） |
| 根目錄 CSV/dump 檔 | 3 個 CSV + 1 個 .dump 散落根目錄 | 低（專案整潔度） |

---

## 2. P0 — 上線阻擋項

### R7-P0-1：恢復 CI 自動觸發

**現況：** `ci.yml` 的 `push` 與 `pull_request` 觸發被註解（`#取消`），CI 僅能手動觸發。
**風險：** 任何人可直接推送未經檢查的程式碼至 main。
**建議：** 至少恢復 `pull_request` 觸發，限定 `main` 分支。

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:
```

**範圍：** `.github/workflows/ci.yml`
**預估工作量：** 5 分鐘

### R7-P0-2：清理根目錄敏感/大型檔案

**現況：**
- `old_ipig.dump` (461 KB) — 舊資料庫 dump
- `20260305_TU-04-00-01_*.csv` (30 KB) — 業務資料
- `匯入用.csv` (12 KB) — 業務資料
- `product_import_from_stocklist.csv` (30 KB)
- `test-login.json` — 測試用登入憑證
- `last_error.json` — 除錯殘留

**風險：** dump 可能包含敏感資料，CSV 含業務資訊，不應進 Git。
**建議：** 將這些加入 `.gitignore` 並移至 `data/` 或從 repo 移除。

**範圍：** `.gitignore`、根目錄清理
**預估工作量：** 15 分鐘

---

## 3. P1 — 程式碼品質與可維護性

### R7-P1-1：後端 `expect()` 漸進式清理

**現況：** 74 處 `expect()`，各 crash 一次就是 500 + 服務中斷。
**建議：** 分批替換為 `?` + `AppError`，優先處理 handlers 與 services 中的 expect。
**範圍：** `backend/src/handlers/`、`backend/src/services/`
**預估工作量：** 2–3 小時

### R7-P1-2：後端巨型檔案拆分

| 檔案 | 行數 | 建議拆分 |
|------|------|----------|
| `services/product.rs` | 1,363 | 拆為 `product/core.rs` + `product/import.rs` + `product/validation.rs` |
| `routes.rs` | 1,212 | 按模組拆為 `routes/auth.rs`, `routes/animal.rs`, `routes/erp.rs` 等 |
| `services/auth.rs` | 1,051 | 拆為 `auth/login.rs` + `auth/token.rs` + `auth/two_factor.rs` |
| `handlers/signature.rs` | 1,040 | 拆為 `signature/create.rs` + `signature/verify.rs` |

**範圍：** `backend/src/`
**預估工作量：** 4–6 小時

### R7-P1-3：前端巨型頁面持續重構

| 檔案 | 行數 | 建議 |
|------|------|------|
| `CreateProductPage.tsx` | 1,541 | 拆分表單區塊為獨立元件（BasicInfo/SkuSection/PricingSection） |
| `ProductsPage.tsx` | 1,001 | 抽離 table columns、filter 邏輯至 hooks |
| `AdminAuditPage.tsx` | 970 | 抽離 chart 配置、filter panel |
| `SectionDesign.tsx` | 922 | 拆分各 section 為子元件 |
| `SettingsPage.tsx` | 922 | 每個設定區塊獨立元件 |

**範圍：** `frontend/src/pages/`
**預估工作量：** 6–8 小時

### R7-P1-4：Migration 檔案一致性檢查

**現況：** `backend/migrations/` 僅有 10 個檔案 (001–010)，但 PROGRESS.md 和 TODO.md 提到 migration 014、020、021、022、023、024。
**建議：** 確認是否有遷移檔案遺失或被合併，確保 migration 序號連續且完整。
**範圍：** `backend/migrations/`
**預估工作量：** 30 分鐘調查

---

## 4. P2 — 架構與效能

### R7-P2-1：後端 `.clone()` 審計

**現況：** 363 處 `.clone()`，部分可能可用 `&str`、`Arc`、`Cow` 或 borrow 替代。
**建議：** 用 clippy 的 `redundant_clone` lint 找出不必要的 clone，分批修復。
**範圍：** `backend/src/`
**預估工作量：** 2–3 小時

### R7-P2-2：資料庫連線池監控強化

**現況：** `error.rs` 已處理 `PoolTimedOut`/`PoolClosed`，但缺乏主動監控。
**建議：**
- 在 `/metrics` 暴露連線池使用率（idle/active/max）
- Grafana dashboard 加入 pool utilization panel
- Alertmanager 設定 pool 使用率 >80% 告警

**範圍：** `backend/src/`, `deploy/`
**預估工作量：** 1–2 小時

### R7-P2-3：前端 Bundle Size 分析與優化

**現況：** 依賴數量龐大（FullCalendar 5 套件、recharts、jspdf、html2canvas 等），可能導致首屏載入緩慢。
**建議：**
- 執行 `npx vite-bundle-visualizer` 分析 bundle 組成
- 確認大型套件（jspdf、html2canvas、recharts、FullCalendar）均已 lazy import
- 考慮 route-level code splitting 確保首屏僅載入必要程式碼

**範圍：** `frontend/`
**預估工作量：** 2–3 小時

### R7-P2-4：API Response 壓縮

**現況：** 前端 nginx 已設定 Brotli 壓縮（`georgjung/nginx-brotli`），但後端 API 直連時可能未壓縮。
**建議：** 確認 `tower-http` 的 `CompressionLayer` 是否已啟用，若無則加入。
**範圍：** `backend/src/main.rs`
**預估工作量：** 15 分鐘

---

## 5. P3 — 測試與可靠性

### R7-P3-1：完成 TODO.md 既有測試項 (R4-100-T4, T5)

**現況：** animal 核心 services 和 protocol/document/hr services 的單元測試未完成。
**建議：** 依 TODO.md R4-100-T4、R4-100-T5 執行。
**範圍：** `backend/src/services/`
**預估工作量：** 4–6 小時

### R7-P3-2：cargo-tarpaulin 覆蓋率量測 (R4-100-T6)

**現況：** TODO.md 已列但未完成。
**建議：**
- CI 中加入 tarpaulin 步驟
- 設定最低覆蓋率門檻（建議初始 60%，逐步提升）
- 產出覆蓋率報告上傳至 GitHub Actions artifact

**範圍：** `.github/workflows/ci.yml`、`backend/`
**預估工作量：** 1–2 小時

### R7-P3-3：前端元件單元測試

**現況：** 有 Storybook stories 和 E2E，但缺少 Vitest 元件單元測試。
**建議：**
- 為核心 hooks（`useConfirmDialog`, `useUnsavedChangesGuard`, `useDateRangeFilter`）寫單元測試
- 為 `auth store` 寫狀態管理測試
- 為 `api.ts` 的 interceptor 寫測試

**範圍：** `frontend/src/`
**預估工作量：** 3–4 小時

### R7-P3-4：Python 整合測試維護

**現況：** `tests/` 有 Python 整合測試但未整合至 CI。
**建議：** 評估是否仍需維護 Python 測試，或統一用 Rust 整合測試 + Playwright E2E 取代。若保留，加入 CI。
**範圍：** `tests/`、`.github/workflows/ci.yml`
**預估工作量：** 2 小時評估

---

## 6. P4 — 安全強化

### R7-P4-1：Docker Secrets 擴展

**現況：** 僅 `google_service_account` 使用 Docker Secrets，`POSTGRES_PASSWORD` 和 `JWT_SECRET` 仍用 `.env`。
**建議：** 將敏感變數遷移至 Docker Secrets（至少 prod compose）。
**範圍：** `docker-compose.prod.yml`
**預估工作量：** 1 小時

### R7-P4-2：API Rate Limiting 精細化

**現況：** Rate limit 統一 600/min。
**建議：**
- 登入端點降至 10/min（防暴力破解）
- 資料匯出端點降至 5/min（防資料竊取）
- 一般 API 維持 600/min

**範圍：** `backend/src/middleware/rate_limiter.rs`
**預估工作量：** 1–2 小時

### R7-P4-3：Content Security Policy (CSP) Header

**現況：** 未確認是否有 CSP header。
**建議：** 在 nginx 設定嚴格的 CSP header，限制 script-src、style-src、img-src。
**範圍：** `frontend/nginx.conf`
**預估工作量：** 1 小時

### R7-P4-4：RUSTSEC Advisory 追蹤

**現況：** CI 忽略 RUSTSEC-2023-0071（rsa 時序攻擊）和 RUSTSEC-2024-0370（proc-macro-error 不維護）。
**建議：** 追蹤上游修復進度，定期檢查是否可移除 ignore。
**範圍：** 文件記錄
**預估工作量：** 15 分鐘

---

## 7. P5 — 開發體驗與 CI/CD

### R7-P5-1：CD Pipeline 增加 Smoke Test

**現況：** `cd.yml` build & push image 後無驗證。
**建議：** 部署後執行基本 smoke test（health check + 登入流程）。
**範圍：** `.github/workflows/cd.yml`
**預估工作量：** 1–2 小時

### R7-P5-2：Pre-commit Hook 強化

**現況：** `.husky/` 有 pre-commit hook，`lint-staged` 僅涵蓋前端。
**建議：** 加入後端 `cargo fmt --check` 和 `cargo clippy` 的 pre-commit 檢查。
**範圍：** `.husky/pre-commit`
**預估工作量：** 15 分鐘

### R7-P5-3：開發環境 Docker Compose 優化

**現況：** `web-dev` service 每次啟動都 `npm install -g npm@latest && npm install`。
**建議：** 使用 `node_modules` volume 快取 + 僅在 `package.json` 變更時重新安裝。
**範圍：** `docker-compose.yml`
**預估工作量：** 30 分鐘

---

## 8. P6 — 長期演進

### R7-P6-1：React 19 升級評估

**現況：** React 18.2.0，React 19 已穩定。
**建議：** 評估 React 19 遷移影響（特別是 Radix UI 相容性、React Hook Form 支援）。
**範圍：** 評估文件
**預估工作量：** 2 小時評估

### R7-P6-2：Tailwind CSS v4 升級評估

**現況：** Tailwind 3.4.1，v4 已發布。
**建議：** 評估 PostCSS → Lightning CSS 遷移、tailwind.config.js → CSS 原生設定的影響。
**範圍：** 評估文件
**預估工作量：** 2 小時評估

### R7-P6-3：Axum / tower-http 版本升級

**現況：** `axum 0.7`、`axum-extra 0.9`、`tower-http 0.5`。R6-5 已評估 axum-extra 0.12 升級。
**建議：** 追蹤 axum 0.8 發布時程，準備升級計劃。
**範圍：** `backend/Cargo.toml`
**預估工作量：** 待 axum 0.8 發布後評估

### R7-P6-4：資料庫輸出與歷史預填 (R6-6 延續)

**現況：** TODO.md R6-6 未完成，設計文件 `DATA_EXPORT_IMPORT_DESIGN.md` 已存在。
**建議：** 繼續執行既有設計，實作匯出 API 與歷史預填功能。
**範圍：** 全端
**預估工作量：** 依設計文件評估

---

## 9. 延續項目

### TODO.md 未完成項處理建議

| # | 項目 | 本輪建議 |
|---|------|----------|
| R4-100-T4 | animal 核心 services 單元測試 | → R7-P3-1 |
| R4-100-T5 | protocol/document/hr services 單元測試 | → R7-P3-1 |
| R4-100-T6 | cargo-tarpaulin 覆蓋率量測 | → R7-P3-2 |
| R6-6 | 資料庫輸出與歷史預填 | → R7-P6-4 |

---

## 執行優先順序建議

```
Week 1:  R7-P0-1 (CI 恢復) → R7-P0-2 (清理) → R7-P1-4 (migration 檢查)
Week 2:  R7-P1-1 (expect 清理) → R7-P3-1 (單元測試) → R7-P3-2 (tarpaulin)
Week 3:  R7-P1-2 (巨型檔案) → R7-P1-3 (前端重構)
Week 4:  R7-P4-1~3 (安全強化) → R7-P5-1~3 (CI/CD 改善)
Week 5+: R7-P2-1~4 (效能) → R7-P6-1~4 (長期演進)
```

---

## 總結

iPig 系統整體品質**優於多數同規模專案**，具備完善的安全防護、合規文件與自動化基礎。本輪改善重點在於：

1. **修復 CI 觸發機制** — 當前最大風險點
2. **持續消除技術債** — expect()、巨型檔案、clone()
3. **補齊測試覆蓋** — 完成既有 TODO 中的測試項目
4. **安全縱深強化** — Docker Secrets、CSP、Rate Limiting 精細化
5. **為未來升級做準備** — React 19、Tailwind v4、Axum 0.8
