# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-03-14 (v15)
> **維護慣例：** 完成項目保留於本表並標 [x]，同時於 `docs/PROGRESS.md` §9 最新變更動態 新增對應紀錄；待辦統計僅計「未完成」數量。
> **AI 標註說明：**
>
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
| P1-7 | **電子簽章合規審查** | 21 CFR Part 11 或等效法規合規審查 | 文件 | 無 | 🧠 Claude | [x] |
| P1-8 | **資料保留政策** | 定義各類紀錄的法定保留年限 | 文件 | 無 | 🧠 Claude | [x] |
| P1-12 | **OpenAPI 文件完善 (≥90%)** | 擴展其餘端點的 Schema 與 Path 定義 | 後端 | 無 | 🧠 Claude | [x] |
| P1-30 | **Graceful Shutdown** | `main.rs` 加入 `tokio::signal` + `with_graceful_shutdown()`，支援 SIGTERM/Ctrl+C | 後端 | 無 | 🧠 Claude | [x] |
| P1-31 | **自訂 404 頁面** | `NotFoundPage` 元件取代 catch-all redirect，含「返回上一頁」與「回到首頁」按鈕 | 前端 | 無 | 🧠 Claude | [x] |
| P1-32 | **Session 逾時預警** | `SessionTimeoutWarning` 元件 + auth store `sessionExpiresAt` 追蹤，到期前 60s 顯示倒數 Dialog | 前端 | 無 | 🧠 Claude | [x] |
| P1-33 | **刪除記錄時清理檔案** | `FileService::delete_by_entity()` 方法，動物/觀察紀錄刪除時連帶清理 `attachments` 表與磁碟檔案 | 後端 | 無 | 🧠 Claude | [x] |
| P1-34 | **Optimistic Locking** | `014_optimistic_locking.sql` 新增 `version` 欄位（animals/protocols/observations/surgeries），更新 SQL 含版本檢查 | 後端 | 無 | 🧠 Claude | [x] |
| P1-35 | **原生 confirm() 統一為 Dialog** | `useConfirmDialog` hook + `ConfirmDialog` + `AlertDialog` 元件，已修復 Admin 設施管理元件中的調用錯誤 | 前端 | 無 | 🧠 Claude | [x] |

---

## 🔴 P2 — 中優先 (品質 / 合規 / UX)

| # | 項目 | 說明 | 範圍 | 依賴 | 建議 AI | 狀態 |
|---|------|------|------|------|----------|------|
| P2-36 | **i18n 硬編碼中文補齊** | AnimalDetailPage Tab 標籤 + 404/Session 預警翻譯鍵加入 zh-TW.json 與 en.json | 前端 | 無 | 🧠 Claude | [x] |
| P2-37 | **列表 API 分頁** | `PaginationParams` + `sql_suffix()` 方法，users/warehouses/partners 三個 handler 支援 `?page=&per_page=`（向後相容） | 後端 | 無 | 🧠 Claude | [x] |
| P2-38 | **表單離開前確認** | `useUnsavedChangesGuard` hook（useBlocker + beforeunload）+ `UnsavedChangesDialog` 元件，已整合 ProtocolEditPage | 前端 | 無 | 🧠 Claude | [x] |
| P2-39 | **隱私政策 / 服務條款頁面** | `PrivacyPolicyPage` + `TermsOfServicePage` 靜態頁面，公開路由 `/privacy` `/terms`，登入頁加連結 | 前端 | 無 | 🧠 Claude | [x] |
| P2-40 | **Cookie 同意橫幅** | `CookieConsent` 元件，localStorage 記憶同意狀態，底部半透明橫幅 + 了解更多連結 | 前端 | 無 | 🧠 Claude | [x] |
| P2-41 | **DB Migration Rollback 文件** | `docs/database/DB_ROLLBACK.md` 涵蓋 14 個 migration 的精確回滾 SQL（逆序）+ 建議回退流程 | 文件 | 無 | 🧠 Claude | [x] |
| P2-42 | **`.env.example` 補齊** | 新增 HOST/PORT/DATABASE_MAX_CONNECTIONS/UPLOAD_DIR/GEOIP_DB_PATH 等 9 個缺漏變數 | DevOps | 無 | 🧠 Claude | [x] |
| P2-43 | **倉庫管理頁面重構** | 依據結構（上/中/下）拆分組件，補全倉庫 CRUD 功能與佈局編輯優化 | 前端 | 無 | 🧠 Claude | [x] |

---

## 🔵 P3 — 低優先 (資安 / 基礎設施)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 7 | **SEC-33：敏感操作二級認證** | 高危操作要求重新輸入密碼確認 | 前後端 | 🧠 Claude | [x] |

---

## 🟣 P4 — 中期品質提升 (測試 / 文件 / CI)

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 17 | **基礎映像與 CVE 週期檢查** | 每季或基礎映像大改時，檢查 [georgjung/nginx-brotli](https://hub.docker.com/r/georgjung/nginx-brotli/tags) 是否有新 tag；若有則升級 frontend Dockerfile 的 FROM，並從 `.trivyignore` 移除 CVE-2026-25646。詳見 `docs/security-compliance/security.md`。**2026-02-28 已升級至 1.29.5-alpine（Alpine 3.23.3），CVE 仍存在（libpng 1.6.54→需 1.6.55），下次 Q2 檢查。** | DevOps | 🧠 Claude | [x] |
| 18 | **E2E Rate Limiting / Session 穩定化** | ~~解決 shared context 下 Session 過期誤判導致大量重新登入~~。已修復：admin-context 改用 auth.setup 儲存的 storageState 免重複登入；API rate limit 120→600/min；login.spec 加入 credential fallback。34/34 連續通過、22s 完成。 | 前端 | 🧠 Claude | [x] |
| 19 | **Prometheus 服務部署** | `docker-compose.monitoring.yml` overlay 新增 Prometheus + Grafana 服務，`deploy/prometheus.yml` 配置 scrape，Grafana provisioning 自動註冊 datasource + dashboard（10 panels：Request Rate / Latency P50-P99 / Error Rate / Status Codes / Heatmap / DB Pool / Pool Utilization / Top Endpoints）。 | DevOps | 🧠 Claude | [x] |
| 20 | **後端 API 整合測試** | `backend/tests/` 建立 6 個整合測試檔案（api_auth / api_health / api_animals / api_protocols / api_users / api_reports），共用 `TestApp` 測試基礎架構（spawn Axum + random port + test DB）。重構 `lib.rs` 使 crate 同時支援 library + binary。 | 後端 | 🧠 Claude | [x] |
| 21 | **效能基準報告文件化** | `docs/assessments/PERFORMANCE_BENCHMARK.md` 正式報告（8 章節：摘要/環境/方法/指標/閾值/資源/限制/結論）。k6 腳本優化：改用 `setup()` 共用 token 消除 rate limit 串連失敗。 | 文件 | 🧠 Claude | [x] |

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| 13 | **前端元件庫文件化** | Storybook 10 建置，15 個 Stories（Button/Badge/Card/Checkbox/Input/Skeleton/Switch + Select/Dialog/Slider/Tabs/AlertDialog/FormField/LoadingOverlay/Textarea） | 前端 | ⚡ Flash | [x] |
| 14 | **前端超長頁面重構** | 漸進式重構巨型組件。**AnimalDetailPage 1,945→748 行（-61%），抽離 7 個 Tab 元件。ProtocolDetailPage 1,929→647 行（-66%），抽離 6 個 Tab 元件（VersionsTab/HistoryTab/CommentsTab/ReviewersTab/CoEditorsTab/AttachmentsTab）。** | 前端 | 🧠 Claude | [x] |
| 15 | **SEC-39：Two-Factor Authentication** | TOTP 2FA 全端實作：後端 `totp-rs` + DB migration + 4 個 API（setup/confirm/disable/verify）+ 登入流程 2FA 檢查 + 備用碼；前端 QR Code 設定 + TOTP 登入驗證 + Profile 頁 2FA 管理 | 前後端 | 🧠 Claude | [x] |
| 16 | **SEC-40：Web Application Firewall** | `docker-compose.waf.yml` overlay 部署 OWASP ModSecurity CRS v4，含 iPig 自訂排除規則（JSON/密碼/上傳/富文本）+ WAF 文件 | DevOps | ⚡ Flash | [x] |
| P5-43 | **ARIA 無障礙標籤** | 12 個檔案新增 23 個 `aria-label`（編輯/刪除/檢視/關閉/導航按鈕） | 前端 | 🧠 Claude | [x] |
| P5-44 | **表單即時驗證回饋** | Input/Textarea 新增 `error` prop 紅框樣式，`FormField` 通用元件含 label + 錯誤訊息 | 前端 | 🧠 Claude | [x] |
| P5-45 | **磁碟空間監控告警** | `scripts/monitor/check_disk_space.sh` 含 uploads 大小 + 磁碟使用率檢查 + Prometheus textfile 輸出 | DevOps | 🧠 Claude | [x] |
| P5-46 | **LICENSE 檔案** | MIT License，2026 iPig System Contributors | 文件 | 🧠 Claude | [x] |
| P5-47 | **index.html Meta Tags** | title「豬博士 iPig 系統」+ description + theme-color + favicon 更新 | 前端 | 🧠 Claude | [x] |
| P5-48 | **useState → Custom Hooks 重構規劃** | 規劃文件 `docs/development/REFACTOR_PLAN_USESTATE_TO_HOOKS.md`，Phase 1–2 完成：useToggle / useDialogSet / useListFilters，遷移 10+ 元件 | 前端 | 🧠 Claude | [x] |

---

## 🔴 P2-R3 — 第三輪改善（品質與維運）

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| P2-R3-11 | **Protocol `any` 型別消除** | ProtocolEditPage/ProtocolContentView/CommentsTab/AttachmentsTab/ReviewCommentsReport/ReviewersTab 共消除 ~44 處 `: any`，改用具體介面（AxiosError/ProtocolWorkingContent 子型別/VetReviewAssignment 等）| 前端 | 🧠 Claude | [x] |
| P2-R3-14 | **Error Boundary 分層** | 新增 `PageErrorBoundary` 元件，於 MainLayout Suspense 外層包裹，捕捉所有 lazy-loaded 頁面的 render 錯誤 | 前端 | 🧠 Claude | [x] |

---

## 🟢 P0–P2 改進計劃（市場基準檢視，2026-03-01）

> 依據 `docs/IMPROVEMENT_PLAN_MARKET_REVIEW.md` 完成項目。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P1-M0 | **稽核日誌匯出 API** | `GET /admin/audit-logs/export?format=csv\|json`，權限 `audit.logs.export` | [x] |
| P1-M1 | **API 版本路徑** | `/api/v1/` 前綴，前端 baseURL 更新 | [x] |
| P1-M2 | **GDPR 資料主體權利** | `GET /me/export`、`DELETE /me/account`，隱私政策補充 | [x] |
| P1-M3 | **維運文件 OPERATIONS.md** | 服務擁有者、on-call、升級流程、故障排除 | [x] |
| P1-M4 | **憑證輪換文件** | `docs/security-compliance/CREDENTIAL_ROTATION.md` 已存在 | [x] |
| P1-M5 | **Dependabot Phase 2 收尾** | zod 4、zustand 5、date-fns 4 已升級 | [x] |
| P2-M2 | **人員訓練紀錄模組** | migration 020、training_records 表、CRUD API、TrainingRecordsPage | [x] |
| P2-M3 | **設備校準紀錄模組** | migration 021、equipment + equipment_calibrations、EquipmentPage | [x] |
| P2-M4 | **稽核日誌 UI 使用者篩選** | AuditLogsPage 新增「操作者」篩選 | [x] |
| P2-M5 | **SOC2_READINESS.md** | Trust Services Criteria 對照文件 | [x] |

---

## 🟣 R4-100 — 邁向 100% 目標（依據 IMPROVEMENT_PLAN_R4 §7）

> 詳見 [IMPROVEMENT_PLAN_R4.md](development/IMPROVEMENT_PLAN_R4.md) §7。兩軌可並行。

### 7.1 核心業務邏輯覆蓋率 100%

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R4-100-T1 | **product service 單元測試** | ProductService 核心邏輯（list/validate、code 解析）提取可測函式 + 5–8 個測試 | 後端 | 🧠 Claude | [x] |
| R4-100-T2 | **partner service 單元測試** | PartnerService 核心邏輯（code 解析、正則）+ 5–8 個測試 | 後端 | 🧠 Claude | [x] |
| R4-100-T3 | **user/role service 單元測試** | UserService、RoleService 可提取邏輯 + 測試 | 後端 | 🧠 Claude | [x] |
| R4-100-T4 | **animal 核心 services 單元測試** | animal/core, observation, medical 等可提取邏輯 (2026-03-09 已完成服務拆分重構) | 後端 | 🧠 Claude | [x] |
| R4-100-T5 | **protocol/document/hr services 單元測試** | 分批補齊 protocol/*, document/*, hr/* | 後端 | 🧠 Claude | [x] |
| R4-100-T6 | **cargo-tarpaulin 覆蓋率量測** | 安裝 tarpaulin，CI 中量測行覆蓋率並設門檻 | DevOps | 🧠 Claude | [x] |

### 7.2 API 文件（OpenAPI）100% 端點文件化

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R4-100-O1 | **products handler OpenAPI** | list/create/get/update/delete, import, with-sku 全端點 #[utoipa::path] + openapi.rs 註冊 | 後端 | 🧠 Claude | [x] |
| R4-100-O2 | **partners handler OpenAPI** | list/create/get/update/delete, import, generate-code 全端點 | 後端 | 🧠 Claude | [x] |
| R4-100-O3 | **documents/storage_location handler OpenAPI** | documents CRUD + submit/approve/cancel；storage_location 全端點 | 後端 | 🧠 Claude | [x] |
| R4-100-O4 | **SKU handler OpenAPI** | categories, subcategories, generate, validate, preview 全端點 | 後端 | 🧠 Claude | [x] |
| R4-100-O5 | **animal 子模組 handler OpenAPI** | observation, surgery, weight, vaccination, transfer, sacrifice, pathology 等 | 後端 | 🧠 Claude | [x] |
| R4-100-O6 | **HR/notifications/admin handler OpenAPI** | leave, overtime, attendance；notifications；admin audit 等 | 後端 | 🧠 Claude | [x] |
| R4-100-O7 | **reports/accounting/treatment_drugs 等 OpenAPI** | 其餘端點補齊 | 後端 | 🧠 Claude | [x] |

---

## 🟠 R6 — 第六輪改善計劃（2026-03）

> 依據 `docs/PROGRESS.md` 專案評估產出。重點：前端可維護性、useState 重構延續、元件品質。

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R6-1 | **useState → hooks 擴展** | AccountingReportPage、TransferTab、HrOvertimePage、EquipmentPage、AdminAuditPage 等高複雜頁面（各 7–15 處 useState），依據 `REFACTOR_PLAN_USESTATE_TO_HOOKS.md` 繼續 Phase 5 | 前端 | 🧠 Claude | [x] |
| R6-2 | **useDateRangeFilter / useTabState** | 建立 `useDateRangeFilter`、`useTabState` 並套用至 HrLeavePage、AdminAuditPage、BloodTestCostReportPage、AuditLogsPage、AccountingReportPage | 前端 | 🧠 Claude | [x] |
| R6-3 | **Skeleton DOM nesting 修正** | skeleton.stories.tsx「行內骨架」`<div>` 於 `<p>` 內造成 validateDOMNesting 警告，改為 `<span>` 或調整結構 | 前端 | ⚡ Flash | [x] |
| R6-4 | **財務模組 Phase 2–5 評估** | AP/AR/GL 後續階段（ap_payments、ar_receipts、trial-balance 等）實作評估，依業務需求排程 | 全端 | 🧠 Claude | [x] |
| R6-5 | **Dependabot Phase 2.5 依賴評估** | printpdf 0.9、utoipa 5、axum-extra 0.12、tailwind-merge 3 升級可行性評估，詳見 `DEPENDABOT_MIGRATION_PLAN.md` | 全端 | 🧠 Claude | [x] |
| R6-6 | **資料庫輸出與歷史重新填寫** | 建立資料庫匯出 API、讓系統可讀取過去資料，並依歷史內容預填表單（手術複製、請假預填、Protocol 複製等），詳見 `docs/development/DATA_EXPORT_IMPORT_DESIGN.md` | 全端 | 🧠 Claude | [x] |
| R6-7 | **日曆功能審視與重構** | 前端元件拆分、Hooks 抽象、後端 Trait 解耦、實作事件預覽 Popover 與單元測試 | 全端 | 🧠 Claude | [x] |
| R6-8 | **設施管理 Migration 補建** | species/facilities/buildings/zones/pens/departments 6 張表在 routes.rs 已有 handler，但 migrations/ 中缺少 CREATE TABLE。需新增 migration 檔案建立完整表結構 | 後端 | 🧠 Claude | [x] |
| R6-9 | **採購單未入庫通知** | 實作已核准採購單未入庫之通知提醒、排程檢查、手動觸發 API 與前端入庫狀態標籤顯示 | 全端 | 🧠 Claude | [x] |
| R6-10 | **採購入庫品項篩選** | 實作 GRN 僅能選擇 PO 內「已核准但未入庫」之品項，強化數據一致性 | 全端 | 🧠 Claude | [x] |

---

## 🔒 R7 — 第七輪改善（安全性原始碼審視，2026-03-08）

> 依據 `docs/development/IMPROVEMENT_PLAN_R7.md` 全面原始碼審視發現。

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R7-P0 | **SQL 拼接修復** | `data_import.rs` 中 `format!()` SQL 改為參數化查詢，消除 SQL injection 風險 | 後端 | 🧠 Claude | [x] |
| R7-P1-1 | **密碼洩露修復** | `create_admin.rs` 不再將管理員密碼明文印至 stdout | 後端 | 🧠 Claude | [x] |
| R7-P1-2 | **TRUST_PROXY 預設值** | `config.rs` 中 `trust_proxy` 由 `true` 改預設 `false` | 後端 | 🧠 Claude | [x] |
| R7-P4-1 | **ETag 常數化** | `etag.rs` 改用 `constants::ETAG_VERSION` 取代硬編碼字串 | 後端 | 🧠 Claude | [x] |
| R7-P4-2 | **Auth Rate Limit 降低** | 認證端點 rate limit 由 100/min 降至 30/min | 後端 | 🧠 Claude | [x] |

---

## 🔧 R8 — 代碼規範重構（2026-03，掃描自動產出）

> 來源：01a-1 目錄掃描 + 01a-2 風格採樣。依優先序排列，高→低。

### 🔴 高優先（架構層）

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R8-1 | **`routes.rs` 依業務域拆分** | `api_routes()` 單一函式超過 1,000 行，應拆成 `routes/animal.rs`、`routes/hr.rs`、`routes/protocol.rs` 等子 Router，再在 `routes/mod.rs` 組裝 | 後端 | 🧠 Claude | [x] |
| R8-2 | **`main.rs` 啟動邏輯提取** | `main()` 約 276 行（migration、middleware、CORS、server 混在一起），`log_startup_config_check()` 約 117 行；應提取至 `startup/` 模組 | 後端 | 🧠 Claude | [x] |
| R8-3 | **建立 `repositories/` 層** | SQL 查詢直接寫在 `services/`，缺少 Repository 層；依規範「相同 SQL SELECT ≥2 次必須提取」，建立 `repositories/animal.rs`、`repositories/protocol.rs` 等，遷移重複 SQL | 後端 | 🧠 Claude | [x] |
| R8-4 | **`utils/access.rs` → `services/access.rs`** | `check_resource_access` 依賴 `CurrentUser`（middleware 型別）與 `AppError`，是業務邏輯非純函式，應移至 `services/access.rs` | 後端 | 🧠 Claude | [x] |

### 🟠 中優先（模組/元件層）

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R8-5 | **`services/animal/core.rs` 拆分** | 684 行，多個方法超過 50 行上限（`list()` 估計 130+ 行含大型 SQL）；依操作類型拆分或提取至 repository 層 | 後端 | 🧠 Claude | [x] |
| R8-6 | **`App.tsx` Route 元件拆離** | 453 行（超過 300 行），`ProtectedRoute`、`ForcePasswordRoute`、`DashboardRoute`、`AdminRoute` 四個內聯元件應移至獨立檔案；`getHomeRedirect` 與 `DashboardRoute` 中角色陣列重複 | 前端 | 🧠 Claude | [x] |
| R8-7 | **`lib/api.ts` 依業務域拆分** | 514 行（超過 300 行），同時含 Axios 設定、interceptors、7 個業務域 API 函式、型別 re-export；拆分為 `lib/api/client.ts`、`lib/api/bloodTest.ts`、`lib/api/animal.ts` 等，`index.ts` 統一匯出 | 前端 | 🧠 Claude | [x] |

### 🟡 低優先（細節/一致性）

| # | 項目 | 說明 | 範圍 | 建議 AI | 狀態 |
|---|------|------|------|----------|------|
| R8-8 | **`AnimalsPage.tsx` / `ProtocolsPage.tsx` 拆分** | 分別為 581 行、375 行，超過 300 行上限；提取子元件或邏輯 hooks | 前端 | 🧠 Claude | [x] |
| R8-9 | **型別 import 路徑統一** | `AnimalsPage.tsx`、`ProtocolsPage.tsx` 等從 `@/lib/api` 取得型別，應改從 `@/types/*` import；移除 `AnimalsPage.tsx` 中未使用的 `import axios` | 前端 | 🧠 Claude | [x] |
| R8-10 | **內嵌常數移至 `constants.ts`** | `ProtocolsPage.tsx` 中 17 行 `statusColors` 常數（及其他頁面同類問題）應移至同層 `constants.ts` 或 `lib/constants/` | 前端 | 🧠 Claude | [x] |
| R8-11 | **`use chrono::Datelike` import 位置修正** | `services/protocol/core.rs` 第 20 行在函式體內 `use chrono::Datelike`，應移至檔案頂部 import 區段 | 後端 | 🧠 Claude | [x] |

---

## 📊 待辦統計

| 優先級 | 數量 (未完成) |
|--------|------|
| 🚨 P0 上線前必要 | 0 |
| 🟡 P1 上線前建議 | 0 |
| 🔴 P2 中優先 | 0 |
| 🔵 P3 低優先 | 0 |
| 🟣 P4 品質提升 | 0 |
| 🟣 R4-100 邁向 100% | 0 |
| ⚪ P5 長期演進 | 0 |
| 🟠 R6 第六輪改善 | 0 |
| 🔒 R7 安全審視 | 0 |
| 🔧 R8 代碼規範重構 | 0 |
| **合計（未完成）** | **0** |

---

## 變更紀錄 (最新)

| 2026-03-14 | 🧠 Claude：儀表板 Widget 捲動體驗優化 — 統一所有 Dashboard Widget 的樣式，確保 `CardContent` 具備 `flex-1 overflow-auto` 捲動條，且 `Card` 標題固定不隨內容捲動。涵蓋「我的計畫」、「動物用藥」、「請假餘額」、「醫事評論」及所有 ERP 內嵌 Widget。 |
| 2026-03-14 | 🧠 Claude：修復 `010_treatment_drug_final.sql` 編碼問題 — 修正非 UTF-8 亂碼內容，確保資料庫遷移能順利通過 Docker 建置。 |
| 2026-03-14 | 🧠 Claude：R4-100-T5 protocol/document/hr 服務單元測試完成 — protocol/numbering 提取 `parse_no_sequence`/`format_protocol_no` + 8 測試；protocol/status 測試 `validate_protocol_content` 7 測試；hr/leave 測試 `is_half_hour_multiple`/`effective_hours` 7 測試；hr/overtime 提取 `overtime_multiplier`/`comp_time_hours_for_type`/`calc_hours_from_minutes` + 8 測試；hr/attendance 測試 `is_ip_in_ranges`/`attendance_status_display` 8 測試；hr/balance 提取 `compute_leave_expiry` + 4 測試；document/grn 提取 `next_seq_from_last_no`/`receipt_status_label` + 8 測試。共 50 個新單元測試，cargo check --tests 通過。 |
| 2026-03-14 | 🧠 Claude：R4-100-T6 cargo-tarpaulin CI 覆蓋率量測 — ci.yml 新增 `backend-coverage` job，`SQLX_OFFLINE=true` 僅跑 lib 單元測試，`--fail-under 25` 設定門檻，產出 XML 報告並上傳為 artifact（保留 14 天）。 |
| 2026-03-14 | 🧠 Claude：品項選擇與單據關連優化 — (1) 在新增明細彈窗加入動態品類篩選（Tabs）；(2) 修正 GRN 來源單據選擇邏輯與 API 400 報錯，確保僅能選擇匹配供應商且已核准的 PO；(3) 修復 Inventory Low-Stock API 500 報錯；(4) 修正 `poReceiptStatus` 未傳遞至 `DocumentLineEditor` 導致待入庫明細未顯示的漏洞。 |
| 2026-03-14 | 🧠 Claude：品項選擇品類篩選優化 (已修正實作) — 在新增明細彈窗加入動態品類篩選（ Tabs），修正調用錯誤 API 的問題，整合 `useSkuCategories` 並修改庫存 API 以支援 `category_code` 過濾，大幅提升 UX。 |
| 2026-03-14 | 🧠 Claude：採購入庫品項篩選強化 — 修正 `GRN` 品項篩選邏輯。新增「來源採購單」選擇 UI 與連動篩選，修正 `poReceiptStatus` 查詢參數，確保 GRN 僅能選擇關聯 PO 之待入庫品項並自動帶入數據。 |
| 2026-03-14 | 🧠 Claude：單據頁面 UI 體驗優化 (V2) — 隱藏銷貨單/出庫單重複的客戶下拉選單；為調撥單新增來源與目標儲位的批次套用功能；將表頭儲位選擇重新標註為「批次套用儲位 (選填)」，並實作新增明細時自動繼承批次儲位的優化，提升同一採購單多儲位的輸入彈性。 |
| 2026-03-14 | 🧠 Claude：修復單據編輯頁面儲位選單問題 — 修正「批次套用儲位」選單選取後 UI 未更新標籤的問題，透過新增 `batchStorageLocationId` 狀態實現正確的 UI 綁定。 |
| 2026-03-14 | 🧠 Claude：專屬計畫載入效能優化 — 擴展 `PO`/`PR` 單據類型的計畫載入觸發條件，解耦 Loading 狀態並解決 UI 始終顯示「載入中」的問題。 |
| 2026-03-14 | 🧠 Claude：庫存導向式品項挑選 — 強化後端 `get_on_hand` API 以支援批號效期細項；改造前端明細挑選彈窗，在涉及現有庫存的單據中自動顯示庫存清單，並實現品項、批號、效期與儲位的一鍵填充。 |
| 2026-03-13 | 🧠 Claude：單據欄位規範調整與邏輯增強 — 實作依單據類型動態切換欄位必填與可見性（倉庫、貨架、計畫、供應商）、實作批號效期強制校驗、IACUC 銷貨警告、庫存流水計畫追蹤。 |
| 2026-03-13 | 🧠 Claude：前端編譯錯誤修復 — 修正 `DocumentEditPage.tsx` 漏掉了 `setFormData` 的解構問題，恢復前端 `npm run build` 與 Docker 建置。 |
| 2026-03-13 | 🧠 Claude：測試基礎設施修復 — 修正 `backend/tests/common/mod.rs` 中 `ensure_admin_user` 函數參數遺漏問題，恢復整合測試代碼編譯。 |
| 2026-03-13 | 🧠 Claude：採購單未入庫通知與狀態顯示 — 實作 `notify_po_pending_receipt` 邏輯、每日 09:00 排程、手動觸發 API；前端新增 `receipt_status` 型別支援與單據列表彩色狀態標籤。 |
| 2026-03-13 | 🧠 Claude：ERP 庫存管理與視覺體驗優化 — 解決「庫存查詢」下拉選單透明重疊問題（引入 Popover.Portal + Glassmorphism）；重塑 Empty State 與表格 Layout；新增「未分配庫存查詢」端點與前端支援；下拉選單穩定性優化；Migration 整合清理。 |
| 2026-03-10 | 🧠 Claude：系統內所有電話欄位（使用者、交易夥伴、動物來源、AUP 計畫主持人/資助者）新增選填「分機」欄位，同步更新前後端型別定義、資料庫 Migration、PDF 產生邏輯與 UI 輸入框。 |
| 2026-03-10 | 🧠 Claude：AUP 計畫主持人電話新增「分機」欄位，同步修復前端類型定義 (phone_ext) 與 `CreateProductPage.tsx` 缺失的 `useEffect` 匯入，確保 Docker 編譯通過。 |
| 2026-03-09 | 🧠 Claude：重構動物服務模組，將 AnimalService 拆分為 9 個專屬 Service，提升代碼組織與可測試性。 |
| 2026-03-09 | 📄 請假管理動作成功後自動重新整理頁面，確保餘額與狀態完全同步。 |
| 2026-03-09 | 📄 API 規格文件全面對齊程式碼（第二輪）— 轉讓端點修正、移除未實現端點、補齊 care-records/treatment-drugs/SSE 等 12 組未記錄端點、ENUM/權限代碼修正、設施遷移待辦新增 |
| 2026-03-08 | 🔒 R7 安全審視完成 — R7-P0 SQL injection 修復、R7-P1 密碼洩露/TRUST_PROXY 修復、R7-P4 ETag 常數化/Auth rate limit 降低；文件全面對齊程式碼 |
| 2026-03-02 | 📄 文件同步：PROGRESS.md 更新至 v5（2026-03-02 動物欄位修正申請）；Profiling_Spec 規格同步；R6 待辦統計校正 |
| 2026-03-01 | 🧠 Claude：R6 第六輪改善全部完成 — R6-4 產出 `docs/assessments/R6-4_FINANCE_PHASE2_5_ASSESSMENT.md`；R6-5 產出 `docs/assessments/R6-5_DEPENDABOT_PHASE25_ASSESSMENT.md` |
| 2026-03-01 | 🧠 Claude：R6 第六輪改善執行 — R6-1 EquipmentPage/TrainingRecordsPage；R6-2 useDateRangeFilter、useTabState 建立並套用 8 頁；R6-3 InlineSkeleton 改 span |
| 2026-03-01 | 🧠 Claude：建立 R6 第六輪改善計劃 — R6-1 useState→hooks 擴展、R6-2 useDateRangeFilter/useTabState、R6-3 Skeleton DOM 修正、R6-4 財務模組評估、R6-5 Dependabot Phase 2.5 評估。依據專案評估產出 |
| 2026-03-01 | 🧠 Claude：財務 SOC2 QAU 三項規劃完成 — QAU 角色/儀表板（022、GET /qau/dashboard、QAUDashboardPage）；SOC2 憑證輪換腳本、SLA.md、DR_DRILL_CHECKLIST；財務 AP/AR/GL（023–024、AccountingService、AccountingReportPage）。詳見 `docs/PROGRESS.md` §9 |
| 2026-03-01 | 🧠 Claude：P0–P2 改進計劃全部完成 — P1-M0 稽核匯出 API、P1-M1 API 版本、P1-M2 GDPR、P1-M3 OPERATIONS.md、P1-M4 憑證輪換、P1-M5 Dependabot；P2-M2 人員訓練紀錄、P2-M3 設備校準、P2-M4 稽核 UI 使用者篩選、P2-M5 security-compliance/SOC2_READINESS.md。詳見 `docs/development/IMPROVEMENT_PLAN_MARKET_REVIEW.md` |
| 2026-02-28 | 🧠 Claude：第三輪系統改善 20 項（P0-R3-1~4 安全 + P1-R3-5~10 效能 + P2-R3-11~20 品質/維運）— SQL QueryBuilder 統一/IDOR 修補/expect() 清理/非 root 容器/搜尋 debounce/staleTime 調優/AnimalsPage 拆分/DashMap Rate Limiter/DB Pool 指標/Skeleton Loading/Protocol any 消除/審計日誌/常數提取/Error Boundary/SSL 範本/備份驗證/Loki 日誌/環境驗證/無障礙/API 一致性。詳見 `docs/development/IMPROVEMENT_PLAN_R3.md` |
| 2026-02-28 | 🧠 Claude：第二輪系統改善 15 項（P0-R2-1~2 安全 + P1-R2-3~8 效能/可靠性 + P2-R2-9~15 品質/維運）— DOMPurify XSS 防護/Rate Limiting 分級/jsPDF 動態導入/動物列表分頁/健康檢查深度擴充/Alertmanager 告警/SMTP 重試/Query Key Factory/Zod 表單驗證/i18n 補齊/Zustand Selector/DB 維護自動化/Dependabot/零停機遷移策略/架構圖。詳見 `docs/development/IMPROVEMENT_PLAN_R2.md` |
| 2026-02-28 | 🧠 Claude：系統改善 14 項（P0-S1~S3 安全性 + P1-S4~S8 效能 + P2-S9~S14 品質）— Docker 網路隔離/DB 埠口/Secrets + N+1 修復/批次 INSERT/移除 .expect()/複合索引 + is_admin()/UserResponse 提取/TypeScript 嚴格化/API 錯誤統一/MainLayout 拆分/Memoization/cargo-chef。詳見 `docs/development/IMPROVEMENT_PLAN_R1.md` |
| 2026-02-28 | 🧠 Claude：完成最終 3 項 P5 待辦 — (1) P5-13 Storybook 15 個 Stories；(2) P5-15 TOTP 2FA 全端實作（後端 totp-rs + 4 API + 登入流程 + 備用碼，前端 QR Code + TOTP 驗證 + Profile 管理）；(3) P5-16 WAF OWASP ModSecurity CRS v4 overlay 部署 + 自訂排除規則 |
| 2026-02-28 | 🧠 Claude：系統設定全端串接 — 後端新增 `GET/PUT /admin/system-settings` API + 10 項 DB seed；前端 SettingsPage 四大區塊（基本/庫存/郵件/安全）全部從 API 載入與儲存；通知路由管理 UI 改善（收合分類/Switch/角色名稱/ConfirmDialog/grid layout）|
| 2026-02-28 | 🧠 Claude：P5-14 ProtocolDetailPage 重構 1,929→647 行（-66%），抽離 VersionsTab/HistoryTab/CommentsTab/ReviewersTab/CoEditorsTab/AttachmentsTab 6 個元件至 `components/protocol/` |
| 2026-02-28 | 🧠 Claude：JWT 預設過期時間從 15 分鐘調整為 360 分鐘（6 小時），更新後端 config / 前端 session fallback / .env / docker-compose 等 7 個檔案 |
| 2026-02-28 | 🧠 Claude：完成 18 項品質補強計畫 — **高影響 6 項**：P1-30 Graceful Shutdown / P1-31 自訂 404 頁面 / P1-32 Session 逾時預警 / P1-33 刪除記錄清理檔案 / P1-34 Optimistic Locking / P1-35 confirm() 統一 Dialog。**中影響 7 項**：P2-36 i18n 補齊 / P2-37 API 分頁 / P2-38 表單離開確認 / P2-39 隱私政策 / P2-40 Cookie 同意 / P2-41 Rollback 文件 / P2-42 .env 補齊。**低影響 5 項**：P5-43 ARIA 標籤 / P5-44 驗證回饋 / P5-45 磁碟監控 / P5-46 LICENSE / P5-47 Meta Tags。|
| 2026-02-28 | 🧠 Claude：完成交付前補強 3 項 — (1) P4-19 Prometheus + Grafana 部署（`docker-compose.monitoring.yml` + `deploy/prometheus.yml` + Grafana provisioning + 10-panel dashboard）；(2) P4-20 後端 API 整合測試（`lib.rs` 重構 + `TestApp` infra + 6 個測試檔 25+ test cases，`cargo check --tests` 通過）；(3) P4-21 效能基準報告（`docs/assessments/PERFORMANCE_BENCHMARK.md` 8 章節正式報告 + k6 腳本 setup() token sharing 優化）。|
| 2026-02-28 | 🧠 Claude：解決 3 個市場交付阻擋項 — (1) 獸醫建議/觀察紀錄檔案上傳下載串接完成（後端新增 `ObservationAttachment` FileCategory + `/observations/:id/attachments` 路由，前端 VetRecommendationDialog 與 ObservationFormDialog 串接 multipart 上傳與下載）；(2) USER_GUIDE.md 從 26 行擴充至完整操作手冊（9 章節含 AUP/動物/ERP/HR/報表/系統管理/FAQ）；(3) docker-compose.prod.yml 補齊所有服務的 CPU/記憶體限制與 json-file 日誌輪轉。|
| 2026-02-28 | 🧠 Claude：完成 P5-14 前端超長頁面重構 — AnimalDetailPage 1,945→748 行（-61%），抽離 7 個 Tab 元件至 `components/animal/`（Observations/Surgeries/Weights/Vaccinations/Sacrifice/AnimalInfo/PathologyTab），TypeScript 零錯誤通過。 |
| 2026-02-28 | 🧠 Claude：完成 P4-17 基礎映像與 CVE 週期檢查 — Dockerfile 版本釘選至 `georgjung/nginx-brotli:1.29.5-alpine`（Alpine 3.23.3），Trivy 掃描確認 CVE-2026-25646 仍存在（libpng 1.6.54-r0→需 1.6.55-r0），.trivyignore 保留並更新註解，下次 Q2 檢查。 |
| 2026-02-27 | 🧠 Claude：完成 P4-18 E2E Rate Limiting / Session 穩定化 — admin-context 改用 storageState 檔案免重複登入、API rate limit 120→600/min、login.spec credential fallback。34/34 連續通過、22s 完成。 |
| 2026-02-27 | 🧠 Claude：E2E 測試總結計畫實施 — 新增 P4-18 Rate Limiting/Session 穩定化待辦；`docs/e2e/README.md` 故障排除 §5 補充 Session 過期導致 429 連鎖失敗說明。 |
| 2026-02-25 | 🧠 Claude：完成 P3-7 SEC-33 敏感操作二級認證 — 後端 confirm-password + reauth token，前後端刪除使用者／重設密碼／模擬登入／刪除角色皆需重新輸入密碼確認。 |
| 2026-02-25 | 🧠 Claude：完成 P1-7 電子簽章合規審查（21 CFR Part 11），新增 `docs/security-compliance/ELECTRONIC_SIGNATURE_COMPLIANCE.md`。 |
| 2026-02-25 | 🧠 Claude：完成 P1-12 OpenAPI 完善 — 新增電子簽章（10 paths + 2 附註）、動物管理（9 paths）及對應 Schema。 |
| 2026-02-25 | 🧠 Claude：修正 CI `sqlx-cli` 安裝錯誤，增加 `--force` 以應對快取衝突。 |
| 2026-02-25 | 🧠 Claude：完成 P1-8 資料保留政策 (Data Retention Policy) 定義。 |
| 2026-02-25 | 🧠 Claude：修正 CI Trivy 掃描參數一致性並清理 `.trivyignore` 無效編號。 |
| 2026-02-25 | ⚡ Flash：完成 P1-5 後端壓力測試基準建立 (k6)，已遷移至 PROGRESS.md。 |
| 2026-02-25 | 🧠 Claude：P1-1 E2E 穩定化 — 429 rate limit 重試、React state race condition fallback、連續 3 次 0 failures。 |
| 2026-02-25 | 🧠 Claude：P1-1 Playwright E2E 測試擴充（7 spec, 34 tests, auth setup + 6 critical flows）。 |
| 2026-02-25 | 🧹 整理：將 P0-6, P0-7, P1-6 已完成項目遷移至 `PROGRESS.md`。 |
| 2026-02-25 | ⚡ Flash 任務第二波：完成 P0-6 跨瀏覽器相容性驗證、P1-6 GLP 驗證文件 (IQ/OQ/PQ) 生成。 |
| 2026-02-25 | ⚡ Flash 任務第一波：完成 Brotli、具名隧道腳本、CI/CD DB 整合、操作手冊與 Grafana 分配。 |
| 2026-02-25 | 🏷️ AI 標註：新增建議使用的 AI 模型標註。 |
