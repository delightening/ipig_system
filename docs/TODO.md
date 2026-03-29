# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-03-29 (v26)
> **維護慣例：** 完成項目標 [x] + 更新待辦統計 + 在 `docs/PROGRESS.md` §9 新增變更紀錄。詳見 `CLAUDE.md`「文件記錄規則」。
> **章節排列：** 禁止事項 → P0~P5（優先級）→ 歷史改善計畫 → R6~R13+（輪次嚴格遞增）→ 待辦統計 → 變更紀錄（封存）

---

## ⛔ 禁止事項

1. 密碼過期策略
2. 密碼歷史紀錄（SEC-38：密碼歷史紀錄）

---

## 🚨 P0 — 上線前必要 (Production Readiness)

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P0-1 | **CI 自動觸發恢復** | `ci.yml` push/pull_request 觸發恢復，限定 main 分支；加入 `--locked` flag | [x] |
| P0-2 | **SQL 字串拼接殘留修復** | `core.rs:139` 已為參數化查詢；`data_import.rs` 表名/欄名來自白名單 + `debug_assert` 防護 | [x] |

---

## 🟡 P1 — 上線前強烈建議 (Quality & Compliance)

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P1-1 | **前端 E2E 測試 (Playwright)** | 7 spec / 34 tests，含 429 重試 + race condition 修正 | [x] |
| P1-2 | **E2E CI 自動化** | `docker-compose.test.yml` + GitHub Actions 整合（依賴 P1-1） | [x] |
| P1-7 | **電子簽章合規審查** | 21 CFR Part 11 或等效法規合規審查 | [x] |
| P1-8 | **資料保留政策** | 定義各類紀錄的法定保留年限 | [x] |
| P1-12 | **OpenAPI 文件完善 (≥90%)** | 擴展其餘端點的 Schema 與 Path 定義 | [x] |
| P1-30 | **Graceful Shutdown** | `main.rs` 加入 `tokio::signal` + `with_graceful_shutdown()` | [x] |
| P1-31 | **自訂 404 頁面** | `NotFoundPage` 元件取代 catch-all redirect | [x] |
| P1-32 | **Session 逾時預警** | `SessionTimeoutWarning` 元件，到期前 60s 顯示倒數 Dialog | [x] |
| P1-33 | **刪除記錄時清理檔案** | `FileService::delete_by_entity()`，連帶清理 attachments 表與磁碟檔案 | [x] |
| P1-34 | **Optimistic Locking** | `014_optimistic_locking.sql` 新增 `version` 欄位，更新 SQL 含版本檢查 | [x] |
| P1-35 | **原生 confirm() 統一為 Dialog** | `useConfirmDialog` hook + `ConfirmDialog` + `AlertDialog` 元件 | [x] |

---

## 🔴 P2 — 中優先 (品質 / 合規 / UX)

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P2-36 | **i18n 硬編碼中文補齊** | AnimalDetailPage Tab 標籤 + 404/Session 預警翻譯鍵 | [x] |
| P2-37 | **列表 API 分頁** | `PaginationParams` + `sql_suffix()`，users/warehouses/partners 支援分頁 | [x] |
| P2-38 | **表單離開前確認** | `useUnsavedChangesGuard` hook + `UnsavedChangesDialog` 元件 | [x] |
| P2-39 | **隱私政策 / 服務條款頁面** | 靜態頁面，公開路由 `/privacy` `/terms` | [x] |
| P2-40 | **Cookie 同意橫幅** | `CookieConsent` 元件，localStorage 記憶同意狀態 | [x] |
| P2-41 | **DB Migration Rollback 文件** | `DB_ROLLBACK.md` 涵蓋 14 個 migration 的回滾 SQL | [x] |
| P2-42 | **`.env.example` 補齊** | 新增 9 個缺漏環境變數 | [x] |
| P2-43 | **倉庫管理頁面重構** | 拆分組件，補全倉庫 CRUD 與佈局編輯 | [x] |

---

## 🔵 P3 — 低優先 (資安 / 基礎設施)

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P3-1 | **SEC-33：敏感操作二級認證** | 高危操作要求重新輸入密碼確認 | [x] |

---

## 🟣 P4 — 中期品質提升 (測試 / 文件 / CI)

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P4-1 | **基礎映像與 CVE 週期檢查** | 每季檢查 nginx-brotli tag；2026-02-28 已升級至 1.29.5-alpine，下次 Q2 檢查 | [x] |
| P4-2 | **E2E Rate Limiting / Session 穩定化** | admin-context 改用 storageState；rate limit 120→600/min；34/34 通過 | [x] |
| P4-3 | **Prometheus 服務部署** | `docker-compose.monitoring.yml` + Grafana provisioning（10 panels） | [x] |
| P4-4 | **後端 API 整合測試** | 6 個整合測試檔案 + `TestApp` 測試基礎架構 | [x] |
| P4-5 | **效能基準報告文件化** | `PERFORMANCE_BENCHMARK.md` 正式報告 + k6 腳本優化 | [x] |

---

## ⚪ P5 — 長期演進

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P5-1 | **前端元件庫文件化** | Storybook 10 建置，15 個 Stories | [x] |
| P5-2 | **前端超長頁面重構** | AnimalDetailPage -61%、ProtocolDetailPage -66%，各抽離 6-7 個 Tab 元件 | [x] |
| P5-3 | **SEC-39：Two-Factor Authentication** | TOTP 2FA 全端實作（totp-rs + QR Code + 備用碼） | [x] |
| P5-4 | **SEC-40：Web Application Firewall** | WAF 改由 Cloudflare WAF 處理，已移除 ModSecurity overlay | [x] |
| P5-5 | **ARIA 無障礙標籤** | 12 個檔案新增 23 個 `aria-label` | [x] |
| P5-6 | **表單即時驗證回饋** | `FormField` 通用元件含 label + 錯誤訊息 | [x] |
| P5-7 | **磁碟空間監控告警** | `check_disk_space.sh` + Prometheus textfile 輸出 | [x] |
| P5-8 | **LICENSE 檔案** | MIT License，2026 iPig System Contributors | [x] |
| P5-9 | **index.html Meta Tags** | title + description + theme-color + favicon | [x] |
| P5-10 | **useState → Custom Hooks 重構規劃** | Phase 1–2 完成：useToggle / useDialogSet / useListFilters | [x] |

---

## 🔴 P2-R3 — 第三輪改善（品質與維運）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| P2-R3-11 | **Protocol `any` 型別消除** | 6 個檔案消除 ~44 處 `: any`，改用具體介面 | [x] |
| P2-R3-14 | **Error Boundary 分層** | 新增 `PageErrorBoundary` 元件，捕捉 lazy-loaded 頁面錯誤 | [x] |

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

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R4-100-T1 | **product service 單元測試** | ProductService 核心邏輯 + 5–8 個測試 | [x] |
| R4-100-T2 | **partner service 單元測試** | PartnerService 核心邏輯 + 5–8 個測試 | [x] |
| R4-100-T3 | **user/role service 單元測試** | UserService、RoleService 可提取邏輯 + 測試 | [x] |
| R4-100-T4 | **animal 核心 services 單元測試** | animal/core, observation, medical 等 | [x] |
| R4-100-T5 | **protocol/document/hr services 單元測試** | 分批補齊 protocol/*, document/*, hr/* | [x] |
| R4-100-T6 | **cargo-tarpaulin 覆蓋率量測** | CI 中量測行覆蓋率並設門檻 | [x] |

### 7.2 API 文件（OpenAPI）100% 端點文件化

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R4-100-O1 | **products handler OpenAPI** | CRUD + import + with-sku 全端點 | [x] |
| R4-100-O2 | **partners handler OpenAPI** | CRUD + import + generate-code 全端點 | [x] |
| R4-100-O3 | **documents/storage_location OpenAPI** | documents CRUD + submit/approve/cancel | [x] |
| R4-100-O4 | **SKU handler OpenAPI** | categories, subcategories, generate, validate, preview | [x] |
| R4-100-O5 | **animal 子模組 handler OpenAPI** | observation, surgery, weight, vaccination 等 | [x] |
| R4-100-O6 | **HR/notifications/admin handler OpenAPI** | leave, overtime, attendance, notifications, audit | [x] |
| R4-100-O7 | **reports/accounting/treatment_drugs 等 OpenAPI** | 其餘端點補齊 | [x] |

---

## 🟠 R6 — 第六輪改善計劃（2026-03）

> 依據 `docs/PROGRESS.md` 專案評估產出。重點：前端可維護性、useState 重構延續、元件品質。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R6-1 | **useState → hooks 擴展** | 5 個高複雜頁面 useState 重構，Phase 5 完成 | [x] |
| R6-2 | **useDateRangeFilter / useTabState** | 建立 2 個 hook 並套用至 5 個頁面 | [x] |
| R6-3 | **Skeleton DOM nesting 修正** | InlineSkeleton `<div>` 改 `<span>` | [x] |
| R6-4 | **財務模組 Phase 2–5 評估** | AP/AR/GL 後續階段評估，依業務需求排程 | [x] |
| R6-5 | **Dependabot Phase 2.5 依賴評估** | printpdf 0.9、utoipa 5 等升級可行性評估 | [x] |
| R6-6 | **資料庫輸出與歷史重新填寫** | 匯出 API + 歷史預填表單 | [x] |
| R6-7 | **日曆功能審視與重構** | 元件拆分、Hooks 抽象、後端 Trait 解耦 | [x] |
| R6-8 | **設施管理 Migration 補建** | 6 張表 CREATE TABLE migration 新增 | [x] |
| R6-9 | **採購單未入庫通知** | 排程檢查 + 手動觸發 API + 狀態標籤 | [x] |
| R6-10 | **採購入庫品項篩選** | GRN 僅能選擇 PO 內已核准但未入庫品項 | [x] |

---

## 🔒 R7 — 第七輪改善（安全性原始碼審視，2026-03-08）

> 依據 `docs/development/IMPROVEMENT_PLAN_R7.md` 全面原始碼審視發現。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R7-1 | **SQL 拼接修復** | `data_import.rs` `format!()` SQL 改為參數化查詢 | [x] |
| R7-2 | **密碼洩露修復** | `create_admin.rs` 不再將密碼明文印至 stdout | [x] |
| R7-3 | **TRUST_PROXY 預設值** | `config.rs` `trust_proxy` 預設改為 `false` | [x] |
| R7-4 | **ETag 常數化** | 改用 `constants::ETAG_VERSION` 取代硬編碼 | [x] |
| R7-5 | **Auth Rate Limit 降低** | 認證端點 rate limit 100→30/min | [x] |

---

## 🔧 R8 — 代碼規範重構（2026-03，掃描自動產出）

> 來源：01a-1 目錄掃描 + 01a-2 風格採樣。依優先序排列，高→低。

### 🔴 高優先（架構層）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R8-1 | **`routes.rs` 依業務域拆分** | 拆成 routes/animal.rs 等子 Router + routes/mod.rs 組裝 | [x] |
| R8-2 | **`main.rs` 啟動邏輯提取** | 提取至 `startup/` 模組 | [x] |
| R8-3 | **建立 `repositories/` 層** | 遷移重複 SQL 至 repositories/ | [x] |
| R8-4 | **`utils/access.rs` → `services/access.rs`** | 權限檢查移至 service 層 | [x] |

### 🟠 中優先（模組/元件層）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R8-5 | **`services/animal/core.rs` 拆分** | 684 行→拆分或提取至 repository 層 | [x] |
| R8-6 | **`App.tsx` Route 元件拆離** | 4 個內聯元件移至獨立檔案 | [x] |
| R8-7 | **`lib/api.ts` 依業務域拆分** | 拆為 client.ts + 業務域 API 檔案 + index.ts | [x] |

### 🟡 低優先（細節/一致性）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R8-8 | **`AnimalsPage/ProtocolsPage` 拆分** | 超過 300 行上限，提取子元件 | [x] |
| R8-9 | **型別 import 路徑統一** | 改從 `@/types/*` import，移除未使用 import | [x] |
| R8-10 | **內嵌常數移至 `constants.ts`** | statusColors 等常數移至 lib/constants/ | [x] |
| R8-11 | **chrono import 位置修正** | 函式體內 `use` 移至檔案頂部 | [x] |

---

## 🔒 R9 — 安全與品質修復（2026-03-15，程式碼審查產出）

> 依據程式碼審查發現的安全漏洞與品質問題。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R9-1 | **IDOR 漏洞修復** | attachment 加入 entity_type 資源級權限檢查 | [x] |
| R9-2 | **上傳 handler 去重** | 抽取通用 `handle_upload()`，upload.rs 606→420 行 | [x] |
| R9-3 | **DB 錯誤碼修正** | 23505→409、23503/23502/23514→400 | [x] |
| R9-4 | **歡迎信安全改善** | `send_welcome_email` 改用密碼重設連結取代明文密碼 | [x] |
| R9-5 | **ERP/HR 整合測試覆蓋** | 完成：erp-inventory + erp-grn + hr-overtime + hr-attendance + file-upload 共 5 個 E2E spec | [x] |

### R9 審查—已知漏洞擱置（開發階段擱置，上線前必做）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R9-C1 | ~~生產環境 WAF 改為 On~~ | WAF 改由 Cloudflare WAF 處理，ModSecurity overlay 已移除 | [x] |
| R9-C2 | **CI 密碼改 GitHub Secrets** | `.github/workflows/ci.yml` 中 JWT_SECRET、DEV_USER_PASSWORD、ADMIN_INITIAL_PASSWORD 改為 GitHub Secrets 並輪替 | [x] |

---

## 🔒 R10 — 程式碼審查 Medium/Low（2026-03-15）

> 依據 `docs/2026_March15_code_review_1.md`，Medium/Low 納入待辦追蹤。

### Medium Severity

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R10-M1 | **Rate limiter 改 Redis** | 單機部署暫不需要，推遲至多節點部署時 | 推遲 |
| R10-M2 | **N+1 修正** | 確認已用 LEFT JOIN + 子查詢，無 N+1 | [x] |
| R10-M3 | **大檔案串流驗證** | MIME 預檢 + 欄位級大小檢查 | [x] |
| R10-M4 | **unwrap 精簡** | 已清零（0 處 unwrap） | [x] |
| R10-M5 | **CSRF 強化** | Signed Double Submit Cookie + 8 個新測試 | [x] |
| R10-M6 | **useUserManagement Zod** | createUser/updateUser Zod schema 驗證 | [x] |
| R10-M7 | **file-upload MIME** | ALLOWED_MIME_TYPES 白名單 + 副檔名檢查 | [x] |
| R10-M8 | **Session timeout 強化** | 推遲，依合規需求決定 | 推遲 |
| R10-M9 | **Alert 門檻** | CPU/Memory/P95/Error rate/Disk 門檻收緊 | [x] |
| R10-M10 | **Prometheus/Grafana 認證** | 確認已安全（環境變數密碼 + 本機綁定） | [x] |

### Low Severity / Suggestions

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R10-L1 | **auth handler 拆分** | 734→7 檔，每檔 ≤227 行 | [x] |
| R10-L2 | **auth service 拆分** | 1006→6 檔，每檔 ≤292 行 | [x] |
| R10-L3 | **signature 拆分** | handler 560→7 檔，service 899→4 檔 | [x] |
| R10-L4 | **product service 拆分** | 832→3 檔 | [x] |
| R10-L5 | **外部 error tracking** | 推遲至上線後 | 推遲 |
| R10-L6 | **Cookie consent 實際阻擋** | 雙按鈕重寫，Google Fonts 動態注入 | [x] |
| R10-L7 | **密碼複雜度** | ≥10 字元 + 大小寫 + 數字 + 黑名單 + 強度指示器 | [x] |
| R10-L8 | **Watchtower 輪詢間隔** | 30→3600 秒 | [x] |
| R10-L9 | **login_events 複合索引** | migration 016 新增 2 個複合索引 | [x] |
| R10-L10 | **JSONB schema validation** | 5 個驗證函式 + 11 個測試 | [x] |

---

## 🔧 R11 — 技術債掃描 + Git 修復（2026-03）

> 來源：2026-03-14 靜態分析掃描 + Git 環境修復。依違反 CLAUDE.md 代碼規範嚴重程度排列。
> 後端函數上限 50 行，前端元件上限 300 行（JSX return 80 行），Hook 上限 300 行。

### Git 與環境配置修復

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R11-0 | **Git 分支衝突修復** | 解決本地與遠端分支領先/落後問題，設定 `pull.rebase true` 策略，清理 `.git/index.lock` 與 `templetes/` 衝突目錄 | [x] |

### 🔴 高優先（後端極長函數 >100 行）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R11-1 | **`pdf/service.rs` 拆分** | 578 行→依章節拆出子函數 | [x] |
| R11-2 | **`import_export.rs` 拆分** | import_basic_data 327 行 + import_weight_data 172 行→輔助函式 | [x] |
| R11-3 | **`services/product.rs` 拆分** | 6 個長函數→CSV/Excel parser 獨立模組 | [x] |
| R11-4 | **`handlers/signature.rs` 拆分** | 簽署驗證邏輯移至 services/signature.rs | [x] |
| R11-5 | **`services/accounting.rs` 拆分** | post_sr/post_do/post_grn 提取子函式 | [x] |

### 🔴 高優先（架構違規：Handler 直接 SQL）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R11-19 | **Handler 層 98 處直接 SQL 清除** | 21+ 個檔案遷移至 service/repository | [x] |
| R11-20 | **Repository 層擴展** | 新增 protocol/animal/hr/user_preferences repository | [x] |

### 🟠 中優先（前端超大元件 >600 行）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R11-6 | **`ProtocolContentView.tsx` 拆分** | 870 行→依內容區塊拆為子元件 | [x] |
| R11-7 | **`ProductImportDialog.tsx` 拆分** | 863 行→預覽/映射/錯誤各自獨立 | [x] |
| R11-8 | **`usePermissionManager.ts` 拆分** | 853 行→categories/search/mutation 三個 hook | [x] |
| R11-9 | **`AccountingReportPage.tsx` 拆分** | 838 行→4 個 Tab 子元件 | [x] |
| R11-10 | **`HrLeavePage.tsx` 拆分** | 837 行→表單/表格/餘額子元件 | [x] |
| R11-11 | **`BloodTestTab.tsx` 拆分** | 811 行→套餐/輸入/歷史子元件 | [x] |
| R11-12 | **`DashboardPage.tsx` 拆分** | 805 行→提取 useDashboardData hook | [x] |
| R11-13 | **`DocumentLineEditor.tsx` 拆分** | 723 行 + 10 處 any→子元件 + 具體型別 | [x] |
| R11-14 | **`useDocumentForm.ts` 拆分** | 717→303 行，提取 useDocumentLines + useDocumentSubmit | [x] |

### 🟡 低優先（前端元件 300–600 行 & 細節問題）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R11-15 | **中大型元件逐步拆分** | 10 個元件全部 ≤300 行（平均 -80%） | [x] |
| R11-16 | **重複常數合併** | STORAGE_CONDITIONS 提取至 lib/constants/product.ts | [x] |
| R11-17 | **剩餘 `any` 型別消除** | 3 個檔案 any→AxiosError/具體型別 | [x] |
| R11-18 | **後端中長函數清理** | auth.rs 4 個 50-66 行函數→提取子函式 | [x] |
| R11-21 | **try-catch → TanStack Query** | 25 處改 useMutation，27 處合理保留 | [x] |
| R11-22 | **源碼 TODO 註解清理** | stocktake 類別篩選 + MyProjectDetailPage 動物查詢 | [x] |

---

## 🟢 R12 — 長期演進項目（已評估未排程，2026-03-20）

> 來源：各評估文件與安全審查建議。視業務需求排程。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R12-1 | **Dependabot Phase 2.5 升級** | utoipa 5、axum-extra 0.12、tailwind-merge 3 等升級完成 | [x] |
| R12-2 | **財務模組 Phase 2–5 實作** | 推遲：Phase 1 自動過帳 + 帳齡報表已涵蓋日常需求 | ⏸️ |
| R12-3 | **圖片處理獨立服務** | `image-processor/` Node.js 服務（Sharp）+ Docker 整合 | [x] |
| R12-4 | **剩餘硬編碼色彩清理** | 748→112 處（-85%），剩餘為規範內或 Canvas 色彩 | [x] |
| R12-5 | **RHF + Zod 表單遷移** | 27 檔 useForm、18 個 Zod schema，CRUD 覆蓋率 100% | [x] |
| R12-6 | **子系統色相實際套用** | Sidebar active → `bg-subsystem-*` 動態色彩 | [x] |
| R12-7 | **CSRF Token 客戶端刷新** | 403 偵測 → 刷新 cookie → 自動重試 | [x] |

---

## 🎨 R13 — UI 一致性與設計規範（2026-03-26）

> 來源：DESIGN.md §15 按鈕規範。UI 元素一致性改善。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R13-1 | **PageHeader 按鈕高度統一** | 完成：34 檔 48 個 Button 統一 `size="sm"`（h-9）。DESIGN.md §15 | [x] |

---

## 📄 R14 — AUP 計畫書 PDF 輸出修正（2026-03-26）

> 來源：使用者回饋，PDF 輸出格式需對齊官方紙本範本。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R14-1 | **PDF 封面標題頁格式修正** | header 字間距、small caps、sponsor/facility 加框線、移除 `=` 分隔、版權固定底部 | [x] |
| R14-2 | **PDF 試驗人員表格格式修正** | 訓練欄每行一筆（`<br>` 分行）、半形括號、欄寬 45% 訓練欄、`safe` filter | [x] |

---

## 🔍 R15 — Code Review 發現（Claude + Codex 交叉審查，2026-03-27）

> 來源：Claude Code Review + OpenAI Codex (GPT-5.4) 獨立審查，針對未提交變更（email 測試、PO 重算、庫存展開行、stock service product_id 篩選）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R15-1 | **展開行數量不匹配（未分配庫存遺漏）** | 概覽模式父行用 stock_ledger 含未分配庫存，但 BatchDetailRows 查 storage_location_inventory 不含未分配 → 數量不一致誤導倉管。Codex 發現，P2 | [x] |
| R15-2 | **PO 重算半完成風險** | `recalculate_all_po_receipt_status` 逐筆開 tx，中途失敗前面已 commit 不會 rollback。Claude+Codex 共同發現，P2 | [x] |
| R15-3 | **expandedRows 不隨 filter 重置** | 切換倉庫/關鍵字篩選時展開狀態殘留，可能對應到不同行。Codex 發現，P2 | [x] |
| R15-4 | **展開行不傳遞 batchFilter** | BatchDetailRows API 呼叫只傳 warehouse_id + product_id，忽略使用者輸入的 batchFilter。Codex 發現，P2 | [x] |
| R15-5 | **Email 引號可能破壞含引號名稱** | display name 加雙引號後，若 from_name 含引號（如 `ACME "QA"`）會造成 lettre parse 失敗；中文 UTF-8 需驗證。Codex 發現，P2 | [x] |
| R15-6 | **recalculate 權限太寬鬆** | 使用 `erp.document.approve` 而非 admin 權限，此維護型 endpoint 應限 admin。Codex 發現，P2 | [x] |
| R15-7 | **Stock service DRY 違規** | 抽出 `SliFilterBuilder` 共用 keyword/product/batch filter 建構邏輯。Claude 發現，Low | [x] |
| R15-8 | **send_test_email handler 超 50 行** | email body 移至 `EmailService::send_test_email`，handler 精簡為 ~35 行。Claude 發現，Low | [x] |
| R15-9 | **`let _ = idx` 抑制 unused 警告** | stock.rs 兩處改為最後一個 filter 後統一放置 + 加註解。R15-4 順便修正 | [x] |
| R15-10 | **BatchDetailRows key 使用 array index** | 改用 `storage_location_id + batch_no` 組合。R15-1 順便修正 | [x] |
| R15-11 | **InventoryPage.tsx 超 300 行** | 拆分為 InventoryPage (220 行) + components/InventoryRow.tsx (262 行)。Claude 發現，Low | [x] |

---

## 🔍 R16 — 全專案 Code Review (2026-03-29)

> 5 面向平行審查：Backend 安全、Frontend 安全、Backend 品質、Frontend 品質、CI/測試覆蓋
> CRITICAL 2 / HIGH 23 / MEDIUM 22 / LOW 11

### 第一批 — 安全 + 功能 Bug（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-1 | **Auth 查詢錯誤靜默吞掉** | `handlers/protocol/` 13+ 處 `.unwrap_or((false,))` 改用 `?` 傳播錯誤，避免 DB 故障遮蔽為 403。CRITICAL | [ ] |
| R16-2 | **Content-Disposition header injection** | `handlers/upload.rs:466` + 12 處 export handler 檔名未跳脫，改用 RFC 5987 percent-encode。HIGH | [ ] |
| R16-3 | **稽核日誌 PDF XSS** | `useAuditLogExport.ts:54-62` document.write 未 HTML 跳脫，加入 `escapeHtml()` 函式。HIGH | [ ] |
| R16-4 | **window.open 缺 noopener** | `VetRecommendationDialog.tsx:85` 補 `'noopener,noreferrer'`。HIGH | [ ] |
| R16-5 | **Query key 不匹配快取 bug** | `useLeaveMutations.ts:25` invalidates `'hr-balance-summary'` 但實際 key 是 `'hr-balance-summary-expiring'`，改用 `queryKeys.hr.balanceSummary`。HIGH | [ ] |
| R16-6 | **window.location.reload() 強制刷新** | `useDocumentSubmit.ts:168`, `DocumentDetailPage.tsx:171` 改用 `queryClient.invalidateQueries()`。HIGH | [ ] |

### 第二批 — 架構 + 安全加固（P1-P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-7 | **Handler 層直接寫 SQL** | `handlers/amendment.rs`, `protocol/crud.rs`, `protocol/review.rs`, `hr/` 等 8+ 檔。抽取至 `repositories/`。CRITICAL（系統性） | [ ] |
| R16-8 | **3 套重複 check_protocol_access** | `amendment.rs`, `pdf_export.rs`, `signature/access.rs` 合併至 `services/access.rs`。HIGH | [ ] |
| R16-9 | **Swagger UI 無認證暴露** | `startup/server.rs:76` production 關閉或加 auth gate。HIGH | [ ] |
| R16-10 | **動態 table name 無白名單** | `services/signature/access.rs:98-145` 加 allowed_tables 驗證。HIGH | [ ] |
| R16-11 | **CSRF 可被 env var 關閉** | `DISABLE_CSRF_FOR_TESTS` 加 production guard，拒絕非 dev 環境啟用。MEDIUM | [ ] |
| R16-12 | **缺 HSTS header** | `startup/server.rs` 加 `Strict-Transport-Security`，gate on `cookie_secure`。MEDIUM | [ ] |
| R16-13 | **CI 硬編碼 fallback 密碼** | `ci.yml:411-468` 移除 fallback，secrets 必填。HIGH | [ ] |

### 第三批 — 品質改善（P2-P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-14 | **角色碼魔術字串** | `hr/dashboard.rs`, `scheduler.rs`, `protocol/crud.rs` 角色碼移至 `constants.rs`。HIGH | [ ] |
| R16-15 | **scheduler.rs 函數過長** | `start()` 235 行、`generate_monthly_report` 138 行。拆分 helper + 子函式。HIGH | [ ] |
| R16-16 | **services/stock.rs 超 800 行** | 942 行，拆分 inventory/ledger 模組。HIGH | [ ] |
| R16-17 | **613 處硬編碼 Tailwind 色彩 token** | 88 個元件/頁面，替換為 CSS Variable token。HIGH（規模大） | [ ] |
| R16-18 | **5 個元件超 300 行** | HrAttendancePage(460), ObservationFormDialog(457), SacrificeFormDialog(427), AnimalEditPage(385), RolesPage(362)。抽出 hooks + 子元件。HIGH | [ ] |
| R16-19 | **PageErrorBoundary 僅覆蓋 5/40+ routes** | 在 `MainLayout` 層統一包裹或逐一補齊。HIGH | [ ] |
| R16-20 | **HR query key 未使用 queryKeys factory** | `HrAttendancePage`, `HrLeavePage` 硬編碼 query key string，遷移至 `queryKeys.hr.*`。HIGH | [ ] |
| R16-21 | **Zustand store 直接 mutation** | `client.ts:112` `sessionExpiresAt` 直接賦值改用 `setState()`。MEDIUM | [ ] |
| R16-22 | **format!() 拼接動態 SQL** | `services/stock.rs:471,499,530,608` 改用 `sqlx::QueryBuilder`。MEDIUM | [ ] |
| R16-23 | **Array index 作 React key** | BloodTestFormDialog, VetReviewForm, HrAnnualLeavePage 等 5 處改用穩定 ID。MEDIUM | [ ] |
| R16-24 | **直接 import axios 繞過中央 client** | `useAnimalsMutations.ts`, `useUserManagement.ts` 改用 `@/lib/api`。LOW | [ ] |
| R16-25 | **console.debug 未限 dev-only** | `webVitals.ts:13` 改用 `logger.debug()` 或 gate `import.meta.env.DEV`。LOW | [ ] |

### 第四批 — CI/測試改善（P2-P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-26 | **GitHub Actions 版本標籤不存在** | `actions/checkout@v6` 等改為正確版本 v4 或 SHA pin。HIGH | [ ] |
| R16-27 | **Backend coverage threshold 僅 2%** | `tarpaulin --fail-under 2` 提高至合理值或整合 integration test 覆蓋。HIGH | [ ] |
| R16-28 | **CI 無 ESLint job** | frontend-check 加入 `npx eslint src --max-warnings=0`。MEDIUM | [ ] |
| R16-29 | **E2E 僅測 read path** | 擴充至少 1 個完整 create+submit flow（animal, protocol, user）。MEDIUM | [ ] |
| R16-30 | **unsafe-guard 只 warning 不 block** | `ci.yml:97` `::warning::` 改為 `exit 1` 或要求 `// SAFETY:` 註解。MEDIUM | [ ] |
| R16-31 | **Edge case 測試不足** | 缺少 refresh token replay、暴力破解 rate limit、檔案上傳安全、分頁邊界、SQL injection in search 等 18+ 項。MEDIUM | [ ] |

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
| 🔒 R9 安全與品質修復 | 0 |
| 🔒 R10 程式碼審查 | 0 (3 推遲) |
| 🔧 R11 技術債 + Git 修復 | 0 |
| 🟢 R12 長期演進項目 | 0 (1 暫緩) |
| 🎨 R13 UI 一致性 | 0 |
| 📄 R14 PDF 輸出修正 | 0 |
| 🔍 R15 Code Review 發現 | 0 |
| 🔍 R16 全專案 Code Review | 31 |
| **合計（未完成）** | **31** |

---

## 變更紀錄（封存，不再新增——變更日誌統一記錄於 `docs/PROGRESS.md` §9）

| 2026-03-26 | 🧠 Claude：R13 更新計畫全面完成 — P0 CI 觸發恢復（P0 歸零）；P1 品質強化（49 Vitest 測試、4 元件 Props 合併、4 audit 色彩 token、CSRF 419）；P2 中優先（FormField 12 檔統一、StatsCard 共用元件、請假日期時區修復、UserEditDialog 單一資料源重構）；P3 長期演進（Dependabot 2.5 utoipa5/axum-extra0.12/tw-merge3、QA browser scripts、E2E 8→12 specs +18 tests）；R12-1 完成、R12-2 暫緩。待辦 5→2。 |
| 2026-03-25 | 🧠 Claude：gstack 全面審查 + Simplify 重構 — Code Review（/review）8 auto-fix + 4 user-approved（deleteResource data 遺失、Retry-After NaN、overtime validation、stale closure、hidden tab bypass、canEditProtocol）；安全審計（/cso）92/100 → 4 項修復（AI rate limit 強制、Cargo.lock 追蹤、CI script injection、/metrics auth）；Simplify（DataTable 7 檔、StatusBadge 7 檔、FilterBar 4 檔、檔案拆分 5→19 檔、watch() 優化 4 檔、formatDate 統一 4 檔）；zodResolver 型別修復 7 檔。待辦 5→5。 |
| 2026-03-25 | 🧠 Claude：RHF+Zod 全面遷移完成 + UI 債清零 — **RHF+Zod** 從 1 檔擴展到 17 檔（Auth 3 頁 + Master 5 頁 + Admin UserForm 3 dialog + AnimalEdit + ApAging + ArAging + WarehouseLayout + Partner + HR 2），新增 10 個 Zod schema 到 validation.ts。**PageHeader** 35 頁遷移。**PageTabs** 9 頁遷移（含 AdminAudit hook 重構）。**EmptyState** 24 檔（19 TableEmptyRow + 11 standalone）。**i18n** 28 處修復跨 15 檔。**a11y** 93 處修復跨 43 檔（73 aria-label + 20 input label）。設計合規度 ~92%。 |
| 2026-03-25 | 🧠 Claude：RHF+Zod 延伸遷移 + DataTable 套用 + Protocol Tab URL 同步 — Partner 表單遷移到 RHF+Zod（`partnerFormZodSchema`，欄位級錯誤顯示，移除手寫 regex 驗證）；HR 5 個列表元件遷移到 DataTable（MyLeaves/AllRecords/PendingApprovals/MyOvertime/PendingOT，移除手寫 Table+Skeleton+Empty）；ProtocolDetailPage 9 個 Tab 從 useState 遷移到 PageTabs URL sync（支援瀏覽器前進/後退/分享連結）；刪除 ProtocolTabNav.tsx（已廢棄）。 |
| 2026-03-25 | 🧠 Claude：R12-4~R12-7 全部完成 — 硬編碼色彩從 748→112（-85%，含 auditLogs 58 處、Auth 表單 85 處、constants 12 處、ErpWidgets 17 處）；HR Leave/Overtime 表單遷移至 React Hook Form + Zod（欄位級驗證錯誤顯示）；Sidebar 子系統色相動態套用（NavItem.subsystem → bg-subsystem-* active 色）；CSRF Token 客戶端自動刷新機制（403 偵測 → GET /auth/me 刷新 cookie → 重試）。待辦 9→5。 |
| 2026-03-24 | 🧠 Claude：UI 一致性重構與設計系統合規 — 新增 5 個共用框架元件（PageHeader/FilterBar/PageTabs/DataTable/StatusBadge）；語義化色彩系統（6 組 status token + 5 子系統色相，Light/Dark 雙主題）；硬編碼色彩從 748→262 處（-65%）；HR 模組全面重構（4 頁遷移 PageHeader+PageTabs、Tab URL 同步、移除 window.location.reload）；ERP/動物管理/Admin/報表/文件模組批次清理；Backend 安全掃描 92/100。新增 R12-4~R12-7 待辦。待辦 5→9。 |
| 2026-03-23 | 🧠 Claude：設備維護管理系統擴充 — Migration 018 新增 6 enum + 5 張新資料表；後端完整 CRUD（廠商/校正確效查核/維修保養/報廢/年度計畫）；前端三個新分頁（維修保養/報廢/年度計畫矩陣）；Email 通知模板 + 排程逾期檢查 + 報廢電子簽章。AI 資料查詢接口 — Migration 017、API Key SHA-256 認證、6 個查詢領域（animals/observations/surgeries/weights/protocols/facilities）、查詢日誌。圖片處理獨立服務 `image-processor/` 上線（R12-3 完成）。會計 Repository 層提取（`repositories/accounting.rs`）。多項 Bug 修正：調整單效期欄位驗證、調撥單批號效期顯示、儲位下拉選單。Dependabot 依賴更新（axum 0.8.8、tower-http 0.6.8、rand 0.9.2、zip 7.2.0、i18next 25.10.4 等）。CI 修復（cargo deny、npm audit、Trivy、SQL guard）。待辦 6→5。 |
| 2026-03-23 | 🧠 Claude：R9-C2 CI 密碼改 GitHub Secrets — `ci.yml` 和 `docker-compose.test.yml` 中的 JWT_SECRET、DEV_USER_PASSWORD、ADMIN_INITIAL_PASSWORD 改為 GitHub Secrets 參照（`CI_JWT_SECRET`、`CI_ADMIN_PASSWORD`、`CI_DEV_PASSWORD`）。DB 密碼維持硬編碼（CI 臨時容器，風險極低）。待辦 7→6。 |
| 2026-03-21 | 🧠 Claude：R10 程式碼審查 17/20 完成 — M2 確認無 N+1、M3 MIME 預檢+欄位級大小檢查、M4 unwrap 已清零、M5 CSRF Signed Double Submit Cookie、M6 Zod 驗證、M7 MIME 白名單、M9 Alert 門檻收緊、M10 確認已安全；L1 auth handler 拆分（734→7 檔）、L2 auth service 拆分（1006→6 檔）、L3 signature 拆分（1459→11 檔）、L4 product service 拆分（832→3 檔）、L6 Cookie consent 重寫、L7 密碼 10 字元+黑名單、L8 Watchtower 3600s、L9 login_events 索引、L10 JSONB 驗證。M1/M8/L5 推遲。待辦 27→7。 |
| 2026-03-21 | 🧠 Claude：R11 技術債全部清零 — R11-15 中大型元件拆分（10 個元件全部降至 ≤300 行，平均縮減 -80%）；R11-21 前端 try-catch 重構（25 處改為 useMutation，27 處合理保留）；R11-22 源碼 TODO 清理（stocktake 類別篩選實作、MyProjectDetailPage 動物查詢實作）。待辦統計 30→27。 |
| 2026-03-20 | 🧠 Claude：未追蹤項目納入 TODO — P0-R12-1 CI 自動觸發恢復、P0-R12-2 SQL 字串拼接殘留修復、R11-22 源碼 TODO 註解清理、R12-1/R12-2/R12-3 長期演進項目（Dependabot 2.5 升級/財務模組 Phase 2–5/圖片處理獨立服務）。待辦統計 25→31。 |
| 2026-03-15 | 🧠 Claude：R9 安全與品質修復 — R9-1 IDOR 漏洞修復（`download_attachment`/`list_attachments` 加入 entity_type 權限檢查）、R9-2 上傳 handler 去重（抽取 `handle_upload()` 通用函式，606→420 行）、R9-3 DB 錯誤碼修正（23505→409、23503/23502/23514→400）。R9-4 歡迎信安全改善、R9-5 ERP/HR 整合測試待後續排程。 |
| 2026-03-15 | 🧠 Claude：Git 歷史紀錄深度清理 — 徹底移除被誤傳進 Git 的 `.venv` 目錄（體積過大）與 `old_ipig.dump`（敏感資料）。使用 `git-filter-repo` 重寫倉庫歷史，移除檔案足跡並減小倉庫體積。更新 `.gitignore` 確保未來不再追蹤。 |
| 2026-03-15 | 🧠 Claude：單據頁面標題顯示優化 — 修正「建立新的undefined」問題。當類型未定時顯示「建立新的單據」。優化「新增/編輯」描述文字。 |
| 2026-03-14 | 🧠 Claude：SSE 安全警報 Cloudflare 524 Timeout 修復 — 後端 `sse.rs` 心跳從 `.text("")` 改為 `.comment("heartbeat")` 並間隔從 30s 縮至 15s；前端 `useSecurityAlerts.ts` 加入指數退避重連（5 次，2s→32s），連線成功重置計數器。 |
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
