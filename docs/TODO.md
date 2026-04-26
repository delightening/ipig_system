# 豬博士 iPig 系統 - 待辦功能清單

> **最後更新：** 2026-04-23 (v32)
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
| P3-2 | **Gotenberg HTTP Timeout 設定** | `services/gotenberg.rs`：`reqwest::Client::new()` 無 timeout，Gotenberg 無回應時 async task 永久 hang。需設定 connect_timeout（5s）與 timeout（60s）。來源：2026-04-17 效能評估。 | [ ] |

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
| P1-M4 | **憑證輪換文件** | `docs/security/CREDENTIAL_ROTATION.md` 已存在 | [x] |
| P1-M5 | **Dependabot Phase 2 收尾** | zod 4、zustand 5、date-fns 4 已升級 | [x] |
| P2-M2 | **人員訓練紀錄模組** | migration 020、training_records 表、CRUD API、TrainingRecordsPage | [x] |
| P2-M3 | **設備校準紀錄模組** | migration 021、equipment + equipment_calibrations、EquipmentPage | [x] |
| P2-M4 | **稽核日誌 UI 使用者篩選** | AuditLogsPage 新增「操作者」篩選 | [x] |
| P2-M5 | **SOC2_READINESS.md** | Trust Services Criteria 對照文件 | [x] |

---

## 🟣 R4-100 — 邁向 100% 目標（依據 IMPROVEMENT_PLAN_R4 §7）

> 詳見 [IMPROVEMENT_PLAN_R4.md](archive/improvement-plans/IMPROVEMENT_PLAN_R4.md) §7。兩軌可並行。

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

> 依據 `docs/archive/improvement-plans/IMPROVEMENT_PLAN_R7.md` 全面原始碼審視發現。

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
| R16-1 | **Auth 查詢錯誤靜默吞掉** | `handlers/protocol/` 13+ 處 `.unwrap_or((false,))` 改用 `?` 傳播錯誤，避免 DB 故障遮蔽為 403。CRITICAL | [x] |
| R16-2 | **Content-Disposition header injection** | `handlers/upload.rs:466` + 12 處 export handler 檔名未跳脫，改用 RFC 5987 percent-encode。HIGH | [x] |
| R16-3 | **稽核日誌 PDF XSS** | `useAuditLogExport.ts:54-62` document.write 未 HTML 跳脫，加入 `escapeHtml()` 函式。HIGH | [x] |
| R16-4 | **window.open 缺 noopener** | `VetRecommendationDialog.tsx:85` 補 `'noopener,noreferrer'`。HIGH | [x] |
| R16-5 | **Query key 不匹配快取 bug** | `useLeaveMutations.ts:25` invalidates `'hr-balance-summary'` 但實際 key 是 `'hr-balance-summary-expiring'`，改用 `queryKeys.hr.balanceSummary`。HIGH | [x] |
| R16-6 | **window.location.reload() 強制刷新** | `useDocumentSubmit.ts:168`, `DocumentDetailPage.tsx:171` 改用 `queryClient.invalidateQueries()`。HIGH | [x] |

### 第二批 — 架構 + 安全加固（P1-P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-7 | **Handler 層直接寫 SQL（授權部分）** | 授權查詢已集中至 `services/access.rs`。protocol/crud, review, export, pdf_export, amendment 已改用。CRITICAL（部分完成：授權查詢） | [x] |
| R16-8 | **3 套重複 check_protocol_access** | 合併至 `services/access.rs`（require_protocol_view_access, require_protocol_related_access 等 9 函式）。HIGH | [x] |
| R16-9 | **Swagger UI 無認證暴露** | `startup/server.rs:76` production 關閉或加 auth gate。HIGH | [x] |
| R16-10 | **動態 table name 無白名單** | `services/signature/access.rs:98-145` 加 allowed_tables 驗證。HIGH | [x] |
| R16-11 | **CSRF 可被 env var 關閉** | `DISABLE_CSRF_FOR_TESTS` 加 production guard，拒絕非 dev 環境啟用。MEDIUM | [x] |
| R16-12 | **缺 HSTS header** | `startup/server.rs` 加 `Strict-Transport-Security`，gate on `cookie_secure`。MEDIUM | [x] |
| R16-13 | **CI 硬編碼 fallback 密碼** | `ci.yml:411-468` 移除 fallback，secrets 必填。HIGH | [x] |

### 第三批 — 品質改善（P2-P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-14 | **角色碼魔術字串** | `hr/dashboard.rs`, `scheduler.rs`, `protocol/crud.rs` 角色碼移至 `constants.rs`。HIGH | [x] |
| R16-15 | **scheduler.rs 函數過長** | `start()` 235 行、`generate_monthly_report` 138 行。拆分 helper + 子函式。HIGH | [x] |
| R16-16 | **services/stock.rs 超 800 行** | 942 行，拆分 inventory/ledger 模組。HIGH | [x] |
| R16-17 | **613 處硬編碼 Tailwind 色彩 token** | 88 個元件/頁面，替換為 CSS Variable token。HIGH（規模大） | [x] |
| R16-18 | **5 個元件超 300 行** | HrAttendancePage(460), ObservationFormDialog(457), SacrificeFormDialog(427), AnimalEditPage(385), RolesPage(362)。抽出 hooks + 子元件。HIGH | [x] |
| R16-19 | **PageErrorBoundary 僅覆蓋 5/40+ routes** | 在 `MainLayout` 層統一包裹或逐一補齊。HIGH | [x] |
| R16-20 | **HR query key 未使用 queryKeys factory** | `HrAttendancePage`, `HrLeavePage` 硬編碼 query key string，遷移至 `queryKeys.hr.*`。HIGH | [x] |
| R16-21 | **Zustand store 直接 mutation** | `client.ts:112` `sessionExpiresAt` 直接賦值改用 `setState()`。MEDIUM | [x] |
| R16-22 | **format!() 拼接動態 SQL** | `services/stock.rs:471,499,530,608` 改用 `sqlx::QueryBuilder`。MEDIUM | [x] |
| R16-23 | **Array index 作 React key** | BloodTestFormDialog, VetReviewForm, HrAnnualLeavePage 等 5 處改用穩定 ID。MEDIUM | [x] |
| R16-24 | **直接 import axios 繞過中央 client** | `useAnimalsMutations.ts`, `useUserManagement.ts` 改用 `@/lib/api`。LOW | [x] |
| R16-25 | **console.debug 未限 dev-only** | `webVitals.ts:13` 改用 `logger.debug()` 或 gate `import.meta.env.DEV`。LOW | [x] |

### 第四批 — CI/測試改善（P2-P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R16-26 | **GitHub Actions 版本標籤不存在** | `actions/checkout@v6` 等改為正確版本 v4 或 SHA pin。HIGH | [x] |
| R16-27 | **Backend coverage threshold 僅 2%** | `tarpaulin --fail-under 2` 提高至合理值或整合 integration test 覆蓋。HIGH | [x] |
| R16-28 | **CI 無 ESLint job** | frontend-check 加入 `npx eslint src --max-warnings=0`。MEDIUM | [x] |
| R16-29 | **E2E 僅測 read path** | 擴充至少 1 個完整 create+submit flow（animal, protocol, user）。MEDIUM | [x] |
| R16-30 | **unsafe-guard 只 warning 不 block** | `ci.yml:97` `::warning::` 改為 `exit 1` 或要求 `// SAFETY:` 註解。MEDIUM | [x] |
| R16-31 | **Edge case 測試不足** | 缺少 refresh token replay、暴力破解 rate limit、檔案上傳安全、分頁邊界、SQL injection in search 等 18+ 項。MEDIUM | [x] |

---

## 🔒 R17 — CSO 安全審計發現（2026-03-29）

> 來源：gstack /cso 全面安全審計（14 phase），對比 2026-03-25 基準。0 CRITICAL / 0 HIGH / 5 MEDIUM / 1 LOW。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R17-1 | **CI 日誌檔含測試密碼** | `logs_61488804168/` 目錄含明文測試密碼已提交至 git，需 `git filter-repo` 移除 + 加 `logs_*/` 到 `.gitignore`。MEDIUM | [x] |
| R17-2 | **Web 容器綁定 0.0.0.0** | `docker-compose.yml:159` web service 未限制 `127.0.0.1`，對外暴露 port 8080。prod overlay 已修正但基礎 compose 未改。MEDIUM | [x] |
| R17-3 | **CSP unsafe-inline + unsafe-eval** | `nginx.conf:22` script-src 含 `'unsafe-inline'`（Cloudflare）+ `'unsafe-eval'`（Vite）。已接受風險，DOMPurify 為補償控制。MEDIUM | 已接受 |
| R17-4 | **/metrics 端點預設無認證** | Prometheus metrics 公開暴露，需設定 `METRICS_TOKEN` 或限制內網存取。LOW | [x] |

> R16-9（Swagger UI 暴露）、R16-26（Actions 未 SHA 釘選）已在 R16 追蹤，不重複列入。

### R17 詳細實作計畫

<details>
<summary>R17-1：CI 日誌檔含測試密碼</summary>

**問題**：`logs_61488804168/` 目錄含 16 個檔案（安全/測試日誌），可能含明文測試密碼，已提交至 git history。

**步驟**：
1. 安裝 `git filter-repo`：`pip install git-filter-repo`
2. 備份 repo：`cp -r .git .git.bak`
3. 移除 git history 中的 logs 目錄：
   ```bash
   git filter-repo --path logs_61488804168/ --invert-paths
   ```
4. 加入 `.gitignore`：
   ```
   # CI/CD log artifacts
   logs_*/
   ```
5. 驗證：`git log --all --full-history -- logs_61488804168/` 應無結果
6. Force push（需確認遠端無其他人在用）：`git push --force-with-lease`
7. 通知所有開發者重新 clone

**風險**：force push 會改變 commit hash，需協調其他分支。
**前置條件**：確認此 repo 無其他人正在工作中的分支。
</details>

<details>
<summary>R17-2：Web 容器綁定 0.0.0.0</summary>

**問題**：`docker-compose.yml:159` web service ports 為 `"${WEB_PORT:-8080}:8080"`（綁 0.0.0.0），`docker-compose.prod.yml:47` 已修正為 `"127.0.0.1:${WEB_PORT:-8080}:8080"`，但基礎 compose 未改。

**步驟**：
1. 修改 `docker-compose.yml` web service：
   ```yaml
   ports:
     - "127.0.0.1:${WEB_PORT:-8080}:8080"
   ```
2. 同步檢查其他 service 的 port binding（API backend、PostgreSQL）是否也綁 0.0.0.0
3. 驗證：`docker compose up -d` → `ss -tlnp | grep 8080` 確認只綁 127.0.0.1
4. 確認 CI docker-compose.test.yml 不受影響（CI 可能需要 0.0.0.0）

**影響範圍**：僅開發/基礎 compose，prod overlay 已正確。
</details>

<details>
<summary>R17-4：/metrics 端點預設無認證</summary>

**問題**：`handlers/metrics.rs:54-97` 已有 `METRICS_TOKEN` Bearer 認證邏輯，但環境變數未設定時端點公開。`config.rs` 未定義此變數，`.env.example` 也未列出。

**步驟**：
1. 在 `.env.example` 新增：
   ```
   # Prometheus metrics authentication (recommended for production)
   # METRICS_TOKEN=your-secure-random-token-here
   ```
2. 在 `config.rs` Config struct 新增 `metrics_token: Option<String>` 欄位，從 `METRICS_TOKEN` 讀取
3. 修改 `handlers/metrics.rs` 改讀 `state.config.metrics_token` 而非直接 `std::env::var`（統一 config 管理）
4. **可選加固**：如果 `cookie_secure = true`（即 production）且 `METRICS_TOKEN` 未設定，回傳 403 並 log warning
5. 在 `OPERATIONS.md` 補充說明

**影響範圍**：僅 /metrics 端點，不影響其他功能。Prometheus scrape config 需加 Bearer token header。
</details>

---

## 🫀 R18 — Heartbeat 自動化維護（2026-03-29）

> 來源：`docs/heartbeatImprovement.md`。透過 Claude Code `/schedule` 定期排程持續維護程式碼品質、安全性與功能完整性。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R18-1 | **每日分段 Code Review 排程** | 10 天一輪迴，每天 review 一個模組區塊（安全、品質、TODO/FIXME、CLAUDE.md 合規），產出報告至 `docs/heartbeat/` | [x] |
| R18-2 | **每日健康檢查排程** | cargo test + clippy + npm audit + cargo deny + build + E2E，產出 health report | [x] |
| R18-3 | **月度架構審查排程** | 每月 1 日深度掃描：量化指標、重複程式碼、依賴健康度、測試覆蓋率 | [x] |
| R18-4 | **Heartbeat 報告目錄建立** | 建立 `docs/heartbeat/` 目錄 + README.md | [x] |

### R18 詳細實作計畫

<details>
<summary>R18-4：Heartbeat 報告目錄建立（先做）</summary>

**步驟**：
1. 建立目錄結構：
   ```
   docs/heartbeat/
     README.md          # 系統說明、報告命名規則、連結索引
   ```
2. README.md 內容：
   - Heartbeat 系統概述
   - 報告類型說明（daily review / health check / monthly architecture）
   - 檔案命名規則（`YYYY-MM-DD.md` / `health-YYYY-MM-DD.md` / `architecture-YYYY-MM.md`）
   - 嚴重度定義（Critical / High / Medium / Low）
3. 加入 `.gitignore` 排除過舊報告（可選）：`docs/heartbeat/` 保留最近 30 天

**前置條件**：無。
</details>

<details>
<summary>R18-1：每日分段 Code Review 排程</summary>

**使用工具**：Claude Code `/schedule` 建立 remote trigger

**排程設定**：
- **頻率**：週一至週五，每日一次
- **Cron**：`0 8 * * 1-5`（每天早上 8:00）

**Prompt 模板**：
```
在 C:\System Coding\ipig_system 執行 Heartbeat 每日 Code Review。

今天是 {date}，根據以下排程表判斷今天是 D 幾：
  D1: handlers/auth/ + middleware/ + services/auth/（認證安全）
  D2: handlers/protocol/ + services/protocol/（IACUC 審查流程）
  D3: handlers/animal/ + services/animal/（動物管理）
  D4: handlers/hr/ + services/hr/ + services/calendar/（HR 模組）
  D5: ERP 相關 handlers + services（庫存、採購、倉儲）
  D6: services/notification/ + email/ + pdf/ + repositories/（通知、PDF、Repository）
  D7: 剩餘 services + models/（審計、設備、簽章等）
  D8: frontend pages/protocols/ + animals/ + amendments/
  D9: frontend pages/admin/ + hr/ + dashboard/ + auth/
  D10: frontend pages/erp/ + inventory/ + master/ + documents/ + reports/ + 共用 components/

計算方式：(工作天數 % 10) + 1 = 今天的 D 值

檢查項目：
1. 函數長度 ≤ 50 行、圈複雜度 ≤ 10、巢狀 ≤ 3 層
2. SQL injection（字串拼接 SQL）、權限檢查完整性、unwrap() 殘留
3. TODO/FIXME/HACK 列表
4. CLAUDE.md 架構分層合規

產出報告寫入 docs/heartbeat/YYYY-MM-DD.md，格式依 heartbeatImprovement.md。
不要自動修復，僅報告發現。
```

**驗收條件**：每天產出一份 markdown 報告，包含發現問題數和嚴重度分布。
</details>

<details>
<summary>R18-2：每日健康檢查排程</summary>

**使用工具**：Claude Code `/schedule` 建立 remote trigger

**排程設定**：
- **頻率**：每日（含週末）
- **Cron**：`0 7 * * *`（每天早上 7:00，比 code review 早 1 小時）

**Prompt 模板**：
```
在 C:\System Coding\ipig_system 執行 Heartbeat 每日健康檢查。

依序執行以下命令並記錄結果：

1. Backend 編譯：cd backend && cargo build --release 2>&1
2. Backend lint：cargo clippy -- -D warnings 2>&1
3. Backend 測試：cargo test 2>&1
4. Backend 安全掃描：cargo deny check advisories 2>&1
5. Frontend 編譯：cd frontend && npm run build 2>&1
6. Frontend 安全掃描：npm audit 2>&1
7. Migration 檢查：列出 backend/migrations/ 目錄確認命名順序

產出報告寫入 docs/heartbeat/health-YYYY-MM-DD.md，格式依 heartbeatImprovement.md。

注意：
- E2E 測試需要 Docker（PostgreSQL），如環境不支援則跳過並標記
- 記錄每項的 pass/fail + 具體錯誤訊息
- 如果 CVE 為 HIGH/CRITICAL，在報告頂部加 ⚠️ 警告
```

**前置條件**：
- Rust toolchain 已安裝（`cargo`、`clippy`）
- Node.js 已安裝（`npm`）
- `cargo-deny` 已安裝（`cargo install cargo-deny`）
</details>

<details>
<summary>R18-3：月度架構審查排程</summary>

**使用工具**：Claude Code `/schedule` 建立 remote trigger

**排程設定**：
- **頻率**：每月 1 日
- **Cron**：`0 9 1 * *`

**Prompt 模板**：
```
在 C:\System Coding\ipig_system 執行 Heartbeat 月度架構審查。

深度檢查以下面向：

1. CLAUDE.md 規範合規
   - 掃描 handlers/ 中是否有直接寫 SQL 的檔案（應在 repositories/）
   - 掃描 services/ 中是否有建構 HTTP response 的程式碼（應在 handlers/）
   - 掃描 utils/ 中是否 import AppState（禁止）
   - 掃描 models/ 中是否 import 其他層（禁止）

2. 量化指標
   - 列出所有 > 50 行的 Rust 函數
   - 列出所有 > 300 行的 React 元件（.tsx）
   - 列出所有 > 6 個 Props 的 React 元件
   - 列出所有 > 5 個參數的 Rust 函數

3. 重複程式碼
   - 掃描相同 SQL SELECT 出現 ≥ 2 處
   - 掃描相同驗證邏輯 ≥ 2 處

4. 依賴健康度
   - 列出 Cargo.toml 中可升級的依賴
   - 列出 package.json 中可升級的依賴
   - 標記 major version behind 的依賴

5. 測試覆蓋
   - 計算 handler 數量 vs 整合測試數量
   - 標記無測試的關鍵 handler

產出報告寫入 docs/heartbeat/architecture-YYYY-MM.md，格式依 heartbeatImprovement.md。
與上月報告對比（如存在），標記趨勢（改善/惡化/持平）。
```

**驗收條件**：每月產出一份包含量化指標和趨勢的架構審查報告。
</details>

---

## 🎫 R19 — 客戶邀請制入口（2026-03-29）

> 來源：`docs/clientsAccess.md`。讓外部客戶透過一次性邀請連結自助註冊，提交 IACUC 計劃書。

### Phase 1：邀請後台

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R19-1 | **invitations migration** | `invitations` 表 + pending email UNIQUE index | [x] |
| R19-2 | **邀請 Backend** | model + handler + service（建立/列表/撤銷/重新發送），含 Email 已註冊→重設密碼、已邀請→重新發送邏輯 | [x] |
| R19-3 | **邀請 Email 模板** | HTML + plain text，含一次性連結 | [x] |
| R19-4 | **邀請管理前端頁面** | Admin 建立 Dialog（只需 Email + 可選組織）、送出後顯示可複製連結、列表 + 狀態篩選 | [x] |
| R19-5 | **邀請權限設定** | `invitation.create/view/revoke/resend` → IACUC_STAFF, SYSTEM_ADMIN | [x] |

### Phase 2：客戶自助註冊

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R19-6 | **公開 verify/accept endpoints** | verify 回傳 token 狀態 + Email；accept 建帳 + 自動分配 PI 角色 + 回傳 JWT | [x] |
| R19-7 | **客戶註冊頁面** | `/invite/{token}` 頁面：驗證連結→填寫資料→設定密碼→自動登入→導向「我的計劃書」 | [x] |
| R19-8 | **錯誤處理頁面** | 連結過期/已使用/無效 → 友善提示頁面 | [x] |

### Phase 3：客戶介面

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R19-9 | **客戶 Sidebar 簡化** | PI 角色 sidebar 只顯示「我的計劃書」+ 個人設定 | [x] |
| R19-10 | **計劃書狀態 Timeline UI** | 視覺化顯示計劃書審查進度（6 階段） | [x] |
| R19-11 | **審查意見通知** | 審查進度變更時 Email + 站內通知客戶 | [x] |

### Phase 4：測試與上線

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R19-12 | **邀請流程 E2E 測試** | 建立→接受→登入→提交計劃書完整流程 | [x] |
| R19-13 | **權限隔離測試** | 客戶不可存取其他人資料、不可存取 Admin/HR/ERP | [x] |
| R19-14 | **安全測試** | Token 暴力破解、過期處理、重複使用防護 | [x] |

### R19 詳細實作計畫

<details>
<summary>R19-1：invitations migration</summary>

**檔案**：`backend/migrations/0XX_invitations.sql`（接在最後一個 migration 之後）

**SQL**：
```sql
CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    organization VARCHAR(255),
    invitation_token VARCHAR(255) UNIQUE NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'expired', 'revoked')),
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    created_user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 同一 Email 只能有一筆 pending 邀請
CREATE UNIQUE INDEX idx_invitations_email_pending
    ON invitations (email) WHERE status = 'pending';

-- Token 查詢索引
CREATE INDEX idx_invitations_token ON invitations (invitation_token);

-- 過期清理用索引
CREATE INDEX idx_invitations_expires_at ON invitations (expires_at)
    WHERE status = 'pending';
```

**注意**：不需要 `role_ids` 欄位，角色固定 PI。
</details>

<details>
<summary>R19-2：邀請 Backend（model + handler + service）</summary>

**新增檔案**：

1. **`backend/src/models/invitation.rs`**
   ```rust
   // DB entity
   pub struct Invitation { id, email, organization, invitation_token, invited_by, status, expires_at, accepted_at, created_user_id, created_at, updated_at }

   // Request DTOs
   pub struct CreateInvitationRequest { email: String, organization: Option<String> }

   // Response DTOs
   pub struct InvitationResponse { ...全欄位..., invite_link: String, invited_by_name: String }
   pub struct CreateInvitationResponse { invitation: InvitationResponse, invite_link: String }

   // 錯誤類型
   pub enum InvitationError { EmailAlreadyRegistered, AlreadyInvited { invitation_id: Uuid }, TokenInvalid, TokenExpired, TokenUsed }
   ```

2. **`backend/src/services/invitation.rs`**
   ```rust
   pub struct InvitationService;
   impl InvitationService {
       // 建立邀請
       pub async fn create(db, email, organization, invited_by, base_url) -> Result<CreateInvitationResponse, AppError> {
           // 1. 檢查 users 表 email 是否已存在 → AppError::Conflict("EmailAlreadyRegistered")
           // 2. 檢查 invitations 表是否已有 pending → AppError::Conflict("AlreadyInvited")
           // 3. generate_crypto_random_token(64) → base64url 編碼
           // 4. INSERT INTO invitations
           // 5. spawn(send_invitation_email) — 非同步不阻塞
           // 6. 回傳 invitation + invite_link
       }

       // 列出邀請（支援 status 篩選 + 分頁）
       pub async fn list(db, status_filter, page, per_page) -> Result<PaginatedResponse<InvitationResponse>>

       // 撤銷邀請
       pub async fn revoke(db, invitation_id) -> Result<()>
           // UPDATE status = 'revoked' WHERE status = 'pending'

       // 重新發送（更新 token + 重設 expires_at + 重發 Email）
       pub async fn resend(db, invitation_id, base_url) -> Result<CreateInvitationResponse>
           // 1. 檢查 status = 'pending'
           // 2. 產生新 token，更新 expires_at = now + 7d
           // 3. 重發 Email + 回傳新連結

       // 驗證 token
       pub async fn verify(db, token) -> Result<VerifyResponse>
           // 回傳 { valid, email, organization, reason }

       // 接受邀請
       pub async fn accept(db, token, req: AcceptInvitationRequest) -> Result<(User, AuthTokens)>
           // 1. 驗證 token (pending + 未過期)
           // 2. UserService::create(email from invitation, ...)
           // 3. RoleService::assign_role(user_id, PI_ROLE_ID)
           // 4. UPDATE invitation status='accepted', accepted_at, created_user_id
           // 5. generate_auth_tokens(user) → 自動登入
           // 6. audit_log("invitation_accepted", ...)

       // 排程：過期清理（可加入 scheduler.rs）
       pub async fn expire_stale(db) -> Result<u64>
           // UPDATE status='expired' WHERE status='pending' AND expires_at < now()
   }
   ```

3. **`backend/src/handlers/invitation.rs`**
   ```rust
   // 需認證（IACUC_STAFF / SYSTEM_ADMIN）
   pub async fn create_invitation(State, Extension(user), Json(req)) -> Result<Json<CreateInvitationResponse>>
   pub async fn list_invitations(State, Extension(user), Query(params)) -> Result<Json<PaginatedResponse>>
   pub async fn revoke_invitation(State, Extension(user), Path(id)) -> Result<StatusCode>
   pub async fn resend_invitation(State, Extension(user), Path(id)) -> Result<Json<CreateInvitationResponse>>

   // 公開（無需認證）
   pub async fn verify_invitation(State, Path(token)) -> Result<Json<VerifyResponse>>
   pub async fn accept_invitation(State, Json(req)) -> Result<Json<AcceptResponse>>
   ```

4. **`backend/src/routes/invitation.rs`**
   ```rust
   pub fn admin_routes() -> Router {
       Router::new()
           .route("/invitations", post(create).get(list))
           .route("/invitations/:id", delete(revoke))
           .route("/invitations/:id/resend", post(resend))
   }
   pub fn public_routes() -> Router {
       Router::new()
           .route("/invitations/verify/:token", get(verify))
           .route("/invitations/accept", post(accept))
   }
   ```

**修改檔案**：
- `backend/src/routes/mod.rs`：註冊 invitation routes（admin 在 protected_routes、public 在 public_routes）
- `backend/src/models/mod.rs`：加 `pub mod invitation;`
- `backend/src/services/mod.rs`：加 `pub mod invitation;`
- `backend/src/handlers/mod.rs`：加 `pub mod invitation;`
- `backend/src/services/scheduler.rs`：加入每日過期清理 job（`0 4 * * *`）

**accept endpoint 的回應**：
```json
{
    "user": { "id": "...", "email": "...", "display_name": "..." },
    "access_token": "eyJ...",
    "refresh_token": "..."
}
```
前端收到後寫入 cookie/store，直接導向 `/my-projects`。
</details>

<details>
<summary>R19-3：邀請 Email 模板</summary>

**新增檔案**：`backend/src/services/email/invitation.rs`

**函式**：
```rust
pub async fn send_invitation_email(
    smtp_config: &SmtpConfig,
    to_email: &str,
    invite_link: &str,
    expires_at: &str,  // 格式化的日期字串
) -> Result<()>
```

**Email 模板**（`resources/templates/email/invitation.html`）：
- 複用現有 Email 模板風格（inline CSS、公司 logo CID、響應式）
- 內容：
  - 標題：邀請您加入實驗動物管理平台
  - 功能說明（提交 AUP、追蹤進度、線上溝通）
  - CTA 按鈕：「完成註冊」→ `{invite_link}`
  - 過期提示：`⏰ 此連結將於 {expires_at} 到期`
  - 聯絡資訊（037-433789）
  - Plain text fallback 版本

**參考**：複用 `services/email/auth.rs` 的 `send_welcome_email()` 結構。
</details>

<details>
<summary>R19-4：邀請管理前端頁面</summary>

**新增檔案**：

1. **`frontend/src/pages/admin/InvitationsPage.tsx`**（≤300 行）
   - 使用 `PageHeader`（標題 + 「新增邀請」按鈕）
   - 使用 `DataTable` 顯示邀請列表
   - 欄位：Email、組織、狀態 badge、邀請人、建立時間、到期時間、操作
   - 狀態篩選 Tab：全部 / Pending / Accepted / Expired / Revoked
   - 操作按鈕：重新發送（pending）、撤銷（pending）
   - 分頁

2. **`frontend/src/pages/admin/components/InvitationCreateDialog.tsx`**（≤200 行）
   - React Hook Form + Zod 驗證
   - 欄位：Email（必填）、組織（選填）
   - 送出成功後切換為「成功狀態」：
     - 顯示「✅ 邀請已送出至 xxx@xxx.com」
     - 顯示可複製的連結 + 📋 複製按鈕（`navigator.clipboard.writeText`）
     - 過期時間提示
   - 錯誤處理：
     - `EmailAlreadyRegistered` → 提示「此 Email 已有帳號」+ 連結到使用者管理
     - `AlreadyInvited` → 提示「已邀請過」+ 「重新發送」按鈕

3. **`frontend/src/lib/api/invitation.ts`**
   ```typescript
   export const invitationApi = {
     create: (data: { email: string; organization?: string }) => client.post('/invitations', data),
     list: (params: { status?: string; page?: number }) => client.get('/invitations', { params }),
     revoke: (id: string) => client.delete(`/invitations/${id}`),
     resend: (id: string) => client.post(`/invitations/${id}/resend`),
   }
   ```

4. **`frontend/src/types/invitation.ts`**
   ```typescript
   export interface Invitation { id, email, organization, status, invited_by_name, expires_at, accepted_at, created_at }
   export type InvitationStatus = 'pending' | 'accepted' | 'expired' | 'revoked'
   ```

**修改檔案**：
- `frontend/src/App.tsx`：加入 `/admin/invitations` 路由
- Sidebar 導航：在 Admin section 加入「邀請管理」項目
- `frontend/src/lib/api/index.ts`：匯出 `invitationApi`
</details>

<details>
<summary>R19-5：邀請權限設定</summary>

**修改檔案**：`backend/src/startup/permissions.rs`

**新增權限**：
```rust
// 在 permissions 定義區加入
("invitation.create", "建立客戶邀請"),
("invitation.view", "查看邀請列表"),
("invitation.revoke", "撤銷邀請"),
("invitation.resend", "重新發送邀請"),
```

**分配角色**：
- `IACUC_STAFF`：invitation.create + view + revoke + resend
- `SYSTEM_ADMIN`：invitation.create + view + revoke + resend

**其他角色不可見邀請管理頁面**。

**前端**：Sidebar 根據 `invitation.view` 權限顯示/隱藏「邀請管理」。
</details>

<details>
<summary>R19-6：公開 verify/accept endpoints</summary>

**verify endpoint**：`GET /api/invitations/verify/{token}`
- 無需認證，加 rate limiter（10 次/分鐘/IP）
- 回應：
  ```json
  // 有效
  { "valid": true, "email": "wang@hospital.org", "organization": "台大醫院" }
  // 已使用
  { "valid": false, "reason": "already_accepted" }
  // 已過期
  { "valid": false, "reason": "expired" }
  // 不存在
  → 404
  ```

**accept endpoint**：`POST /api/invitations/accept`
- 無需認證，加 rate limiter（5 次/分鐘/IP — 更嚴格防暴力破解）
- Request body：
  ```json
  {
      "invitation_token": "a8f3...x9z",
      "display_name": "王大明",
      "phone": "0912345678",
      "organization": "台大醫院",
      "password": "SecurePass123!",
      "position": "主治醫師",
      "agree_terms": true
  }
  ```
- 驗證：
  - `invitation_token`：查 DB，狀態必須 pending + 未過期
  - `display_name`：1-100 字元
  - `phone`：9-10 位數字
  - `password`：≥ 10 字元（複用現有密碼規則，含大小寫 + 數字）
  - `agree_terms`：必須 true
- 成功後：
  1. 建立 user（`must_change_password = false`）
  2. 分配 PI 角色
  3. 更新 invitation（status=accepted）
  4. 產生 JWT access_token + refresh_token
  5. 回傳：user 資訊 + tokens
  6. 設定 HttpOnly cookie（與現有 login 一致）
</details>

<details>
<summary>R19-7：客戶註冊頁面</summary>

**新增檔案**：`frontend/src/pages/auth/InvitationAcceptPage.tsx`

**路由**：`/invite/:token`（公開路由，無需認證）

**流程**：
```
頁面載入
  → useEffect: GET /api/invitations/verify/{token}
  → 成功 (valid=true)：顯示註冊表單，Email 預填（readonly）
  → 失敗 (valid=false)：顯示錯誤頁面（R19-8）

使用者填寫表單
  → React Hook Form + Zod 驗證
  → 欄位：
    - Email（readonly，從 verify 回傳）
    - 姓名*（display_name）
    - 電話*（phone）
    - 組織*（organization，從 verify 預填，可修改）
    - 職稱（position，選填）
    - 密碼*（含強度指示器）
    - 確認密碼*
    - □ 同意服務條款（連結到 /terms）
  → 送出：POST /api/invitations/accept

成功回應
  → 將 tokens 寫入 auth store
  → 設定 cookie
  → navigate('/my-projects')
  → toast.success('歡迎加入！')
```

**UI 設計**：
- 使用與 LoginPage 相同的佈局風格（居中卡片）
- 公司 logo + 標題「完成註冊」
- 表單 ≤ 250 行，提取 Zod schema 到 validation.ts
</details>

<details>
<summary>R19-8：錯誤處理頁面</summary>

**在 InvitationAcceptPage.tsx 中處理**（不需獨立檔案）：

```
token 不存在 (404)   → 「此邀請連結無效，請聯繫管理員」
已使用 (accepted)    → 「此邀請已使用，如忘記密碼請」→ 連結到 /forgot-password
已過期 (expired)     → 「此邀請已過期，請聯繫管理員重新發送」
已撤銷 (revoked)     → 「此邀請已被撤銷，請聯繫管理員」
```

每種狀態顯示友善圖示 + 說明 + 操作建議。
</details>

<details>
<summary>R19-9：客戶 Sidebar 簡化</summary>

**修改檔案**：`frontend/src/components/layout/Sidebar.tsx`（或同層導航元件）

**邏輯**：
```typescript
// 判斷是否為「純客戶」（只有 PI 角色，無其他管理角色）
const isClientOnly = user.roles.length === 1 && user.roles[0].code === 'PI';

if (isClientOnly) {
    // 只顯示：
    // - 我的計劃書 (/my-projects)
    // - 個人設定 (/profile)
    // 隱藏所有 Admin、HR、ERP、動物管理、報表 section
}
```

**注意**：如果 PI 同時有其他角色（如 EXPERIMENT_STAFF），則顯示完整 sidebar。這個邏輯確保內部人員不受影響。
</details>

<details>
<summary>R19-10：計劃書狀態 Timeline UI</summary>

**新增元件**：`frontend/src/components/protocol/ProtocolTimeline.tsx`

**顯示在**：`MyProjectDetailPage.tsx` 頂部

**視覺設計**：水平進度條，6 個節點

```
[Draft] ─── [Submitted] ─── [Pre-Review] ─── [Vet Review] ─── [Committee] ─── [Approved]
  ●            ●               ●               ○               ○              ○
  完成          完成            進行中

  ● = 已完成（綠色）
  ◉ = 進行中（藍色脈動）
  ○ = 未到達（灰色）
  ✕ = 退回修改（橙色，顯示在對應階段）
```

**狀態對應**：
- Draft → 節點 1
- Submitted → 節點 2
- Pre_Review / Pre_Review_Revision_Required → 節點 3
- Vet_Review / Vet_Revision_Required → 節點 4
- Under_Review / Revision_Required / Resubmitted → 節點 5
- Approved / Approved_With_Conditions → 節點 6
- Rejected / Suspended / Closed → 特殊狀態標記

**資料來源**：`protocol.status`（已有）+ `protocol_status_history`（顯示各階段時間戳）
</details>

<details>
<summary>R19-11：審查意見通知</summary>

**修改檔案**：`backend/src/services/notification/protocol.rs`

**新增通知事件**：
| 事件 | 通知對象 | 通道 |
|------|---------|------|
| 計劃書狀態變更 | PI（protocol owner） | Email + 站內 |
| 新審查意見 | PI | Email + 站內 |
| 退回修改 | PI | Email + 站內（含修改建議摘要） |
| 核准 | PI | Email + 站內（含 IACUC 編號） |

**Email 模板新增**：`resources/templates/email/protocol_status.html`
- 動態內容：計劃書標題、新狀態、審查意見摘要（如有）
- CTA 按鈕：「查看詳情」→ `{base_url}/my-projects/{protocol_id}`

**現有基礎**：`services/notification/protocol.rs` 已有通知框架，只需新增 PI 對象的觸發邏輯。
</details>

<details>
<summary>R19-12 / R19-13 / R19-14：測試</summary>

**R19-12 E2E 測試**：`frontend/e2e/invitation.spec.ts`
```
test('完整邀請流程', async () => {
    // 1. Admin 登入 → 建立邀請
    // 2. 取得邀請連結
    // 3. 開新 context → 訪問邀請連結
    // 4. 填寫註冊表單 → 提交
    // 5. 驗證自動登入 → 看到「我的計劃書」
    // 6. 驗證可建立新計劃書
})

test('重複邀請處理', async () => { /* 已邀請 Email → 提示 */ })
test('過期連結處理', async () => { /* 過期 token → 友善提示 */ })
```

**R19-13 權限隔離測試**：`backend/tests/api_invitations.rs`
```rust
#[test] async fn pi_cannot_access_admin_pages() { /* GET /api/users → 403 */ }
#[test] async fn pi_cannot_see_other_protocols() { /* GET /api/protocols → 只回傳自己的 */ }
#[test] async fn pi_cannot_access_erp() { /* GET /api/products → 403 */ }
#[test] async fn pi_cannot_access_hr() { /* GET /api/hr/attendance → 403 */ }
```

**R19-14 安全測試**：`backend/tests/api_invitations.rs`
```rust
#[test] async fn brute_force_token_rate_limited() { /* 10+ requests → 429 */ }
#[test] async fn expired_token_rejected() { /* 設定過期 → 400 */ }
#[test] async fn used_token_rejected() { /* 已 accept → 400 */ }
#[test] async fn revoked_token_rejected() { /* 已 revoke → 400 */ }
#[test] async fn invalid_token_404() { /* 不存在 → 404 */ }
```
</details>

---

## 🤖 R20 — AI 預審與執行秘書標註（2026-03-29）

> 來源：`docs/AIReview.md` + `docs/clientsAccess.md` §4。雙角色 AI 審查：客戶端預審 + 執行秘書標註。

### Phase 1：規則式檢查

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R20-1 | **Backend 驗證規則擴展** | `services/protocol/validation.rs` — 字數門檻、日期邏輯、3Rs 完整性、疼痛分類 vs 麻醉一致性 | [x] |
| R20-2 | **驗證 API endpoint** | `POST /api/protocols/{id}/validate` — Level 1 規則檢查 | [x] |
| R20-3 | **前端提交前驗證 UI** | 提交時觸發驗證 + 報告面板（必須修正/建議改善） | [x] |

### Phase 2：Claude API 整合

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R20-4 | **protocol_ai_reviews migration** | 儲存 AI 預審結果，含 `review_type`（client_pre_submit / staff_pre_review） | [x] |
| R20-5 | **AI 預審 service** | 擴展 `services/ai/` — system prompt、計劃書序列化、回應解析、快取、成本控制 | [x] |
| R20-6 | **客戶端 AI 預審** | `POST /api/protocols/{id}/ai-review` + 前端 AI 預審按鈕 + 結果面板 | [x] |
| R20-7 | **執行秘書 AI 標註** | `POST /api/protocols/{id}/staff-review-assist` + Pre-Review 頁面頂部標註面板（🚩⚠️ℹ️ 三類） | [x] |
| R20-8 | **Pre-Review 自動觸發** | Status 變更為 Pre_Review 時自動呼叫 AI 標註 | [x] |

### Phase 3：調校與優化

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R20-9 | **System prompt 調校** | 收集真實審查意見，對比 AI 預審結果，調整 prompt 提高準確率。**2026-04-12 階段一已完成**：基於 45 封真實 IACUC 信件分析，CLIENT/STAFF 兩個 system prompt 已套用 5 類補丁（交叉引用稽核、人道終點量化、對照組處置、3R 教學挑戰、文書 pre-filter）。完整報告見 `docs/R20_real_review_patterns.md`。**剩餘**：Gmail Takeout data pipeline、Evonne 標 50 筆 ground truth、`backend/tests/ai_review_eval.rs` eval harness、Recall ≥ 0.7 / Precision ≥ 0.6 baseline | 🔶 |
| R20-10 | **退回率追蹤** | 追蹤 Pre-Review 退回次數是否下降 | [ ] |

### R20 詳細實作計畫

<details>
<summary>R20-1：Backend 驗證規則擴展（Level 1 規則引擎）</summary>

**新增檔案**：`backend/src/services/protocol/validation.rs`

**規則引擎設計**：

```rust
pub struct ValidationResult {
    pub passed: Vec<ValidationCheck>,
    pub errors: Vec<ValidationIssue>,    // 必須修正
    pub warnings: Vec<ValidationIssue>,  // 建議改善
}

pub struct ValidationIssue {
    pub code: String,           // e.g. "3RS_REDUCTION_MISSING"
    pub category: String,       // e.g. "3Rs", "animals", "design"
    pub section: String,        // e.g. "purpose", "animals", "design"
    pub message: String,        // 人類可讀訊息
    pub suggestion: String,     // 建議修正方式
}

pub fn validate_protocol(working_content: &serde_json::Value) -> ValidationResult
```

**驗證規則清單**（從 `working_content` JSON 解析）：

| 規則 | 類型 | 欄位 | 條件 |
|------|------|------|------|
| 研究目的字數 | error | `purpose.significance` | ≥ 100 字，「略」「同上」視為無效 |
| Replacement 說明 | error | `purpose.replacement` | ≥ 50 字，必須說明為何不能用替代方法 |
| Reduction 說明 | error | `purpose.reduction` | ≥ 50 字，必須提及統計方法或文獻支持 |
| Refinement 說明 | error | `purpose.refinement` | ≥ 50 字，必須提及痛苦最小化措施 |
| 日期邏輯 | error | `basic.start_date`, `end_date` | end > start，期限 ≤ 3 年 |
| 動物數量 | error | `animals.total_count` | > 0 且與分組合計一致 |
| 疼痛分類 vs 麻醉 | warning | `design.pain_category`, `design.anesthesia` | C/D/E 類必須有麻醉方案 |
| 人員訓練證照 | warning | `personnel[].training` | 所有人員應有證照編號 |
| 替代方案搜尋平台 | warning | `purpose.alternative_databases` | ≥ 2 個平台 |
| 人道終點具體性 | warning | `design.humane_endpoint` | 不含「明顯」「嚴重」等模糊詞，應有量化指標 |
| 術後觀察頻率 | warning | `design.post_op_care` | 如有手術，必須提及觀察時間點 |
| 實驗期程合理性 | warning | `basic.start_date`, `end_date` | > 2 年標記提醒 |
| 安樂死方法 | warning | `design.euthanasia_method` | 對照 AVMA 推薦方法清單 |
| 附件完整性 | warning | `attachments[]` | 至少 1 份附件 |

**實作要點**：
- 從 `working_content` JSONB 解析各欄位，容忍欄位缺失（Option）
- 每條規則獨立函式，方便擴展
- 回傳結構化結果，前端可直接對應到表單 section
</details>

<details>
<summary>R20-2：驗證 API endpoint</summary>

**新增 handler**：`backend/src/handlers/protocol/validation.rs`

```rust
/// POST /api/protocols/{id}/validate
/// 權限：protocol owner (PI/Co-editor) 或 IACUC_STAFF
pub async fn validate_protocol(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<ValidationResult>, AppError> {
    // 1. 權限檢查：require_protocol_view_access
    // 2. 讀取 protocol.working_content
    // 3. 呼叫 validation::validate_protocol(working_content)
    // 4. 回傳 ValidationResult
}
```

**路由**：在 `routes/protocol.rs` 加入：
```rust
.route("/protocols/:id/validate", post(validate_protocol))
```

**回應格式**：
```json
{
    "errors": [
        { "code": "3RS_REDUCTION_MISSING", "category": "3Rs", "section": "purpose", "message": "...", "suggestion": "..." }
    ],
    "warnings": [...],
    "passed": ["research_purpose", "personnel_qualifications", ...]
}
```
</details>

<details>
<summary>R20-3：前端提交前驗證 UI</summary>

**新增元件**：`frontend/src/components/protocol/ValidationPanel.tsx`

**觸發時機**：
1. 使用者點擊「提交」按鈕時，先呼叫 `POST /api/protocols/{id}/validate`
2. 如有 errors → 阻擋提交，顯示 ValidationPanel
3. 如只有 warnings → 顯示 ValidationPanel，使用者可選擇「修正」或「忽略並提交」
4. 全部通過 → 直接提交

**元件結構**：
```tsx
<ValidationPanel result={validationResult}>
  {/* errors 區塊 — 紅色，必須修正 */}
  <ValidationSection severity="error" issues={result.errors} />

  {/* warnings 區塊 — 黃色，建議改善 */}
  <ValidationSection severity="warning" issues={result.warnings} />

  {/* passed 區塊 — 綠色，可摺疊 */}
  <ValidationSection severity="passed" items={result.passed} />

  {/* 操作按鈕 */}
  <Button onClick={fix}>修正</Button>
  {onlyWarnings && <Button onClick={submitAnyway}>忽略建議，直接提交</Button>}
</ValidationPanel>
```

**每個 issue 可點擊**：跳轉到對應的表單 section（利用既有的 section tab 導航）。

**修改檔案**：
- `frontend/src/pages/protocols/ProtocolEditPage.tsx`：提交流程插入 validate 步驟
- `frontend/src/lib/api/protocol.ts`：新增 `validate(protocolId)` API 函式
</details>

<details>
<summary>R20-4：protocol_ai_reviews migration</summary>

**檔案**：`backend/migrations/0XX_protocol_ai_reviews.sql`

```sql
CREATE TABLE protocol_ai_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    protocol_version_id UUID REFERENCES protocol_versions(id),
    review_type VARCHAR(30) NOT NULL
        CHECK (review_type IN ('client_pre_submit', 'staff_pre_review')),
    -- Level 1 結果
    rule_result JSONB,
    -- Level 2 AI 結果
    ai_result JSONB,
    ai_model VARCHAR(50),          -- 'claude-haiku-4-5' | 'claude-sonnet-4-6'
    ai_input_tokens INTEGER,
    ai_output_tokens INTEGER,
    -- 合併結果
    total_errors INTEGER NOT NULL DEFAULT 0,
    total_warnings INTEGER NOT NULL DEFAULT 0,
    score INTEGER,                  -- 0-100 整體評分
    -- 元資訊
    triggered_by UUID REFERENCES users(id),  -- NULL = 自動觸發
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 查詢最新一筆 AI review
CREATE INDEX idx_ai_reviews_protocol_latest
    ON protocol_ai_reviews (protocol_id, created_at DESC);

-- 避免同一 version 重複呼叫
CREATE UNIQUE INDEX idx_ai_reviews_version_type
    ON protocol_ai_reviews (protocol_version_id, review_type)
    WHERE protocol_version_id IS NOT NULL;
```

**設計考量**：
- `rule_result` 和 `ai_result` 分開存，Level 1 不花錢可頻繁呼叫
- `ai_input_tokens` / `ai_output_tokens` 追蹤成本
- `protocol_version_id` + `review_type` UNIQUE index 防重複呼叫
- 不分區（量少，每筆計劃書最多幾十筆 review）
</details>

<details>
<summary>R20-5：AI 預審 service（Claude API 整合）</summary>

**新增檔案**：`backend/src/services/protocol/ai_review.rs`

**Config 擴展**（`config.rs`）：
```rust
pub struct Config {
    // ... 現有欄位 ...
    pub anthropic_api_key: Option<String>,       // ANTHROPIC_API_KEY
    pub ai_review_model: String,                 // AI_REVIEW_MODEL, 預設 "claude-haiku-4-5"
    pub ai_review_enabled: bool,                 // AI_REVIEW_ENABLED, 預設 true
    pub ai_review_timeout_secs: u64,             // AI_REVIEW_TIMEOUT_SECS, 預設 30
}
```

**Service 結構**：
```rust
pub struct AiReviewService;

impl AiReviewService {
    /// 完整預審（Level 1 + Level 2）
    pub async fn review_protocol(
        db: &PgPool,
        config: &Config,
        protocol_id: Uuid,
        review_type: &str,        // "client_pre_submit" | "staff_pre_review"
        triggered_by: Option<Uuid>,
    ) -> Result<AiReviewResult, AppError> {
        let start = Instant::now();

        // 1. 讀取 protocol.working_content
        let protocol = find_protocol_by_id(db, protocol_id).await?;
        let content = &protocol.working_content;

        // 2. Level 1：規則引擎
        let rule_result = validation::validate_protocol(content);

        // 3. 快取檢查：同一 version + type 已有結果 → 直接回傳
        if let Some(cached) = find_cached_review(db, protocol.current_version_id, review_type).await? {
            return Ok(cached);
        }

        // 4. Level 2：Claude API（僅在 Level 1 基本通過 + API key 存在時呼叫）
        let ai_result = if config.anthropic_api_key.is_some() && config.ai_review_enabled {
            Some(call_claude_api(config, content, review_type).await?)
        } else {
            None
        };

        // 5. 合併結果
        let combined = merge_results(rule_result, ai_result);

        // 6. 儲存至 DB
        insert_ai_review(db, protocol_id, protocol.current_version_id, review_type, &combined, triggered_by, start.elapsed()).await?;

        Ok(combined)
    }

    /// 呼叫 Claude API
    async fn call_claude_api(
        config: &Config,
        content: &serde_json::Value,
        review_type: &str,
    ) -> Result<AiResult, AppError> {
        let client = reqwest::Client::new();  // 複用現有 reqwest 依賴

        // 序列化計劃書內容為結構化文本
        let protocol_text = serialize_protocol_for_ai(content);

        // 選擇 system prompt
        let system_prompt = match review_type {
            "client_pre_submit" => CLIENT_REVIEW_PROMPT,
            "staff_pre_review" => STAFF_REVIEW_PROMPT,
            _ => return Err(AppError::BadRequest("Invalid review type")),
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", config.anthropic_api_key.as_ref().unwrap())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .timeout(Duration::from_secs(config.ai_review_timeout_secs))
            .json(&serde_json::json!({
                "model": config.ai_review_model,
                "max_tokens": 2048,
                "system": system_prompt,
                "messages": [{ "role": "user", "content": protocol_text }]
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Claude API: {}", e)))?;

        // 解析回應
        let body: serde_json::Value = response.json().await?;
        let text = body["content"][0]["text"].as_str().unwrap_or("");

        // 解析 JSON（Claude 回傳結構化 JSON）
        parse_ai_response(text)
    }
}
```

**System Prompt 常數**：
```rust
const CLIENT_REVIEW_PROMPT: &str = r#"
你是一位資深的 IACUC 審查委員，擁有實驗動物科學與獸醫學背景。
你的任務是預審動物實驗計劃書（AUP），幫助計畫主持人在提交前改善內容。
...（完整 prompt 見 docs/AIReview.md）
回覆格式為 JSON: { "summary": "...", "score": 72, "issues": [...], "passed": [...] }
"#;

const STAFF_REVIEW_PROMPT: &str = r#"
你是一位資深的 IACUC 審查輔助系統，協助執行秘書進行 Pre-Review。
你的任務是標註計劃書中值得注意的地方，幫助審查人員聚焦重點。
產出三類標註：
- 🚩 needs_attention（格式/完整性問題）
- ⚠️ concern（內容疑慮）
- ℹ️ suggestion（審查建議）
回覆格式為 JSON: { "summary": "...", "flags": [...] }
"#;
```

**成本控制**：
- 快取：同一 `protocol_version_id` + `review_type` 不重複呼叫
- 模型選擇：預設 Haiku（快速便宜），`ai_review_model` 可設為 Sonnet
- Token 限制：`serialize_protocol_for_ai` 截斷至 ≤ 8K tokens
- Rate limit：每用戶每日 10 次（在 handler 層檢查 `protocol_ai_reviews` 表 count）
</details>

<details>
<summary>R20-6：客戶端 AI 預審</summary>

**Backend handler**：`backend/src/handlers/protocol/ai_review.rs`

```rust
/// POST /api/protocols/{id}/ai-review
/// 權限：protocol owner (PI/Co-editor)
/// Rate limit：10 次/天/用戶
pub async fn ai_review_protocol(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<AiReviewResult>, AppError> {
    // 1. 權限：require_protocol_edit_access
    // 2. Rate limit 檢查：今天已用次數
    // 3. AiReviewService::review_protocol(db, config, id, "client_pre_submit", Some(user.id))
    // 4. 回傳結果
}

/// GET /api/protocols/{id}/ai-review/latest
/// 取得最新一筆 AI review 結果（快取用）
pub async fn get_latest_ai_review(...)
```

**Frontend**：

1. **`frontend/src/components/protocol/AIReviewButton.tsx`**
   ```tsx
   // 放在 ProtocolEditPage 工具列
   <Button onClick={triggerAiReview} disabled={isLoading}>
     {isLoading ? <Spinner /> : '🔍 AI 預審'}
   </Button>
   // 剩餘次數顯示：「今日剩餘 8/10 次」
   ```

2. **`frontend/src/components/protocol/AIReviewPanel.tsx`**
   ```tsx
   // 顯示 AI 預審結果
   <Card>
     <CardHeader>AI 預審報告 — 評分 {score}/100</CardHeader>
     <CardContent>
       {errors.map(issue => <IssueItem severity="error" issue={issue} />)}
       {warnings.map(issue => <IssueItem severity="warning" issue={issue} />)}
       <Collapsible><PassedItems items={passed} /></Collapsible>
     </CardContent>
     <CardFooter>
       <Button onClick={rerun}>重新檢查</Button>
       {onlyWarnings && <Button onClick={submitAnyway}>忽略建議，直接提交</Button>}
     </CardFooter>
   </Card>
   ```

3. **`frontend/src/lib/api/aiReview.ts`**
   ```typescript
   export const aiReviewApi = {
     trigger: (protocolId: string) => client.post(`/protocols/${protocolId}/ai-review`),
     getLatest: (protocolId: string) => client.get(`/protocols/${protocolId}/ai-review/latest`),
   }
   ```

**修改檔案**：
- `ProtocolEditPage.tsx`：加入 AIReviewButton + AIReviewPanel
- `routes/protocol.rs`：加入新路由
</details>

<details>
<summary>R20-7：執行秘書 AI 標註</summary>

**Backend handler**：

```rust
/// POST /api/protocols/{id}/staff-review-assist
/// 權限：IACUC_STAFF, IACUC_CHAIR
pub async fn staff_review_assist(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<StaffReviewResult>, AppError> {
    // 1. 權限：require permission "aup.review.comment" 或 IACUC_STAFF
    // 2. AiReviewService::review_protocol(db, config, id, "staff_pre_review", Some(user.id))
    // 3. 回傳結果
}

/// GET /api/protocols/{id}/staff-review-assist/latest
pub async fn get_latest_staff_review(...)
```

**Frontend 元件**：`frontend/src/components/protocol/StaffReviewAssistPanel.tsx`

```tsx
// 顯示在 ProtocolDetailPage 的 Pre-Review 階段頂部
// 只有 IACUC_STAFF / IACUC_CHAIR 可見

<Alert variant="info">
  <AlertTitle>📋 Pre-Review 審查輔助</AlertTitle>

  {/* 🚩 需要注意 */}
  <Section title="🚩 需要注意" items={flags.filter(f => f.type === 'needs_attention')} color="red" />

  {/* ⚠️ 留意事項 */}
  <Section title="⚠️ 留意事項" items={flags.filter(f => f.type === 'concern')} color="yellow" />

  {/* ℹ️ 審查建議 */}
  <Section title="ℹ️ 審查建議" items={flags.filter(f => f.type === 'suggestion')} color="blue" />

  <footer>
    AI 標註僅供參考，請依專業判斷審查
    <Button onClick={reanalyze}>重新分析</Button>
  </footer>
</Alert>
```

**修改檔案**：
- `ProtocolDetailPage.tsx`：在 Pre-Review 狀態時顯示 StaffReviewAssistPanel
- `CommentsTab.tsx`：可選 — 在審查意見區旁邊顯示 AI 建議
</details>

<details>
<summary>R20-8：Pre-Review 自動觸發</summary>

**修改檔案**：`backend/src/services/protocol/status.rs`

在 `change_status()` 函式中，當狀態變更為 `Pre_Review` 時：

```rust
ProtocolStatus::PreReview => {
    // ... 現有邏輯（assign co-editor 等）...

    // 自動觸發 AI 標註（非同步，不阻塞狀態變更）
    if state.config.ai_review_enabled && state.config.anthropic_api_key.is_some() {
        let db = state.db.clone();
        let config = state.config.clone();
        let pid = protocol_id;
        tokio::spawn(async move {
            if let Err(e) = AiReviewService::review_protocol(
                &db, &config, pid, "staff_pre_review", None  // None = 自動觸發
            ).await {
                tracing::warn!("Auto AI review failed for protocol {}: {}", pid, e);
            }
        });
    }
}
```

**設計要點**：
- `tokio::spawn` 非同步執行，狀態變更不等 AI 結果
- 失敗只 log warning，不影響正常流程
- 執行秘書打開頁面時，如果 AI 結果已就緒則直接顯示，否則顯示「分析中...」
- 可手動點「重新分析」強制重跑
</details>

<details>
<summary>R20-9：System prompt 調校</summary>

**持續性工作，非一次性開發**。

**方法**：
1. 上線後收集前 20 筆真實計劃書的 AI 預審結果
2. 與實際 Pre-Review / Committee 審查意見對比
3. 分析 False Positive（AI 標記但人工未標記）和 False Negative（人工標記但 AI 遺漏）
4. 調整 system prompt：
   - 如 FP 過多 → 提高判斷門檻，減少 warning
   - 如 FN 過多 → 增加特定領域的檢查指引
5. 記錄每次 prompt 版本和對應的準確率
6. 保存在 `docs/ai-review-prompt-history.md`

**目標**：AI 標記問題 vs 人工審查問題的重疊率 ≥ 80%。
</details>

<details>
<summary>R20-10：退回率追蹤</summary>

**新增查詢**：在 QAU Dashboard 或新增報表頁面

**SQL**：
```sql
-- 月度退回率
SELECT
    DATE_TRUNC('month', h.created_at) AS month,
    COUNT(*) FILTER (WHERE h.to_status IN ('Pre_Review_Revision_Required', 'Vet_Revision_Required', 'Revision_Required')) AS revision_count,
    COUNT(*) FILTER (WHERE h.to_status IN ('Submitted', 'Resubmitted')) AS submission_count,
    ROUND(
        COUNT(*) FILTER (WHERE h.to_status LIKE '%Revision%')::NUMERIC /
        NULLIF(COUNT(*) FILTER (WHERE h.to_status IN ('Submitted', 'Resubmitted')), 0) * 100, 1
    ) AS revision_rate_pct
FROM protocol_status_history h
GROUP BY 1
ORDER BY 1 DESC;
```

**前端**：在 QAU Dashboard 新增「退回率趨勢」圖表（Recharts line chart），月度追蹤。

**衡量基準**：上線 AI 預審前的退回率 vs 上線後，目標降低 50%。
</details>

---

## 🌡️ R21 — 環境監控子系統（MES-Lite）（2026-04）

> 冰箱溫度 + 動物房溫濕度感測器資料收集、即時監控、超限告警。整合進現有 ERP 作為新子系統，不建立獨立 MES。
> 技術棧：TimescaleDB（PostgreSQL extension）+ HTTP/MQTT + Recharts。

### 21-A 後端基礎建設（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R21-1 | **DB Migration：感測器設備表** | `sensor_devices`（id, name, location, type enum: temperature/humidity/combo, calibration_due_at）+ `sensor_readings`（device_id, metric_type, value, unit, recorded_at）；TimescaleDB hypertable on `sensor_readings` | [ ] |
| R21-2 | **DB Migration：告警規則表** | `alert_rules`（device_id, metric_type, min_value, max_value, notify_emails, is_active）+ `alert_events`（rule_id, value, triggered_at, resolved_at, acknowledged_by）| [ ] |
| R21-3 | **感測器資料接收 API** | `POST /api/v1/sensors/readings`（API key 認證，Bearer token，複用 `config.rs` 模式）；handler → service → repository 分層 | [ ] |
| R21-4 | **歷史查詢 API** | `GET /api/v1/sensors/readings?device_id&from&to&interval=5m`（TimescaleDB `time_bucket` 降採樣）；`GET /api/v1/sensors/devices`（設備列表）| [ ] |
| R21-5 | **告警規則 CRUD API** | `GET/POST/PUT/DELETE /api/v1/sensors/alert-rules`（需 `sensor.config` 權限）| [ ] |
| R21-6 | **告警觸發邏輯** | `services/sensor/alert.rs`：每次寫入後檢查規則；超限時呼叫 `services/notification/` + email；自動 resolve（恢復正常後更新 `resolved_at`）| [ ] |

### 21-B 前端（P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R21-7 | **Dashboard 即時面板** | `DashboardPage` 新增「環境監控」區塊；各設備目前溫濕度數值卡片；超限高亮紅色 CSS variable token | [ ] |
| R21-8 | **歷史趨勢圖頁面** | `pages/sensors/SensorHistoryPage.tsx`；Recharts LineChart；時間範圍選擇（1h/6h/24h/7d）；設備切換 | [ ] |
| R21-9 | **告警管理頁面** | `pages/sensors/AlertRulesPage.tsx`；規則 CRUD（RHF + Zod）；告警事件列表（已觸發/已解除/待確認）| [ ] |
| R21-10 | **Subsystem 導覽整合** | Sidebar 新增「環境監控」子系統入口；色相：`--subsystem-sensor: cyan`（DESIGN.md 登記）| [ ] |

### 21-C 硬體整合文件（P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R21-11 | **感測器端設定文件** | `docs/sensor-setup/SETUP.md`：ESP32/Raspberry Pi 範例程式碼（Python/Arduino）；HTTP POST payload 格式；API key 申請流程 | [ ] |
| R21-12 | **MQTT Broker 評估** | 評估 Mosquitto 整合（替代 HTTP polling）；適合感測器數量 > 20 個時啟用；短期不需要 | ⏸️ |

---

## 🛡️ R22 — 攻擊偵測與主動告警（2026-04）

> 建立完整的入侵偵測管線：被動記錄 → 智慧告警 → 主動推送 → 長期可觀測性。
> 依據 Security Audit Report (2026-04-14) 偵測盲區分析，補齊 rate limit、403 權限拒絕、主動通知三大缺口。
> 參照：`dev/SECURITY_AUDIT_REPORT.md`、`docs/COMPLIANCE_DELIVERY_SUMMARY.md` §10

### 22-A 層 1：被動記錄 — 讓事後鑑識有資料（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R22-1 | **Rate limit 事件寫入 DB** | `middleware/rate_limiter.rs` 觸發時呼叫 `AuditService::log_security_event()`，4 tier 全覆蓋 | [x] |
| R22-2 | **AI key rate limit 事件記錄** | `ai_auth.rs` deactivated/expired/rate_limited 三事件寫入 DB | [x] |
| R22-3 | **403 Permission denied 記錄** | `response_logger.rs` middleware 攔截 403 回應寫入 DB | [x] |
| R22-4 | **Account lockout 事件記錄** | `login.rs` lockout 觸發時寫入 DB | [x] |

### 22-B 層 2：智慧告警 — 自動產生 security_alerts（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R22-5 | **Auth rate limit 升級告警** | 同一 IP 超過閾值 → critical alert + 去重 + 主動通知 | [x] |
| R22-6 | **IDOR 探測偵測** | 同一 user 超過閾值 403 → critical alert + 去重 + 主動通知 | [x] |
| R22-7 | **Brute force alert 去重** | `check_brute_force()` 加 30 分鐘去重（同 `global_mass_login` 模式） | [x] |
| R22-8 | **告警閾值設定化** | `security_alert_config` 表 + `AlertThresholdService` 60s cache | [x] |

### 22-C 層 3：主動推送 — 即時通知管理者（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R22-9 | **通知管道抽象層** | `SecurityNotifier::dispatch()` 從 `security_notification_channels` 讀取啟用管道 | [x] |
| R22-10 | **Email 通知實作** | 複用現有 SMTP + HTML 模板，收件人從 channel config_json 讀取 | [x] |
| R22-11 | **LINE Notify 整合** | POST notify-api.line.me + `LINE_NOTIFY_TOKEN` env var | [x] |
| R22-12 | **Webhook 通用管道** | POST JSON payload 到 config_json.url，10s timeout | [x] |
| R22-13 | **排程掃描未處理告警** | `scheduler.rs` 每 6 小時掃描 open + >24h alert 重送通知 | [x] |

### 22-D 額外考量：可觀測性與蜜罐（P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R22-14 | **集中式 Log 收集評估** | `docs/r22-log-aggregation.md` — 推薦 Loki（Grafana 同 stack） | [x] |
| R22-15 | **Grafana 安全 Dashboard** | 待 Loki 部署後建立 LogQL dashboard（依賴 R22-14 Phase 2） | ⏸️ |
| R22-16 | **蜜罐端點（Honeypot）** | 6 個假端點（/.env, /wp-login.php 等），觸發 critical alert + 通知，回傳 404 | [x] |
| R22-17 | **Admin Audit 頁面 — 安全事件 Tab** | 前後端完成，11 種 event_type 篩選，SecurityEventsTab 元件 | [x] |
| R22-18 | **Docker log driver 設定** | `docker-compose.prod.yml` api log rotation 50m/5 + tag | [x] |

---

## 🎨 R23 — 全站 Table UI 一致性升級（2026-04）

> 以 ProductTable 為黃金標準，將全站 ~100 個 Table 元件統一至相同容器樣式、header 色彩、row 狀態、
> loading/empty 元件，以及 Tier A 頁面的 mobile card fallback。
> 所有顏色均使用 CSS Variable token，禁止硬編碼色彩。

### Batch 0 — DataTable 共用元件修正（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-0 | **DataTable 基礎樣式升級** | `data-table.tsx` container→`rounded-lg bg-card overflow-hidden [&>div]:overflow-x-hidden`；header→`bg-muted/50 hover:bg-muted/50` | [x] |

### Batch 1 — Master & Inventory Tables（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-1a | **PartnerTable 樣式升級** | container / header / row-states / mobile card | [x] |
| R23-1b | **DocumentTable 樣式升級** | container / table-fixed / header / 取消 doc→`bg-destructive/5` / mobile card | [x] |
| R23-1c | **BloodTestTemplateTable 升級** | container / 替換自製 SortIndicator→`SortableTableHead` / skeleton | [x] |
| R23-1d | **StockLedgerPage 升級** | container / header / skeleton | [x] |

### Batch 2 — Animals & Admin Core Tables（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-2a | **AnimalListTable 全面升級** | 移除 Card wrapper；containerRef / computeWidths（11 欄）；SortableTableHead；mobile AnimalCard | [x] |
| R23-2b | **UserTable 升級** | container / 替換 getSortIcon+button→SortableTableHead / 移除 bg-white | [x] |
| R23-2c | **AuditLogTable 升級** | container / header / skeleton | [x] |

### Batch 3 — Master Pages + Protocol Tabs（P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-3a | **BloodTest Pages 升級** | BloodTestPanelsPage / BloodTestPresetsPage | [x] |
| R23-3b | **WarehousesPage / ProtocolsPage / AnimalSourcesPage 升級** | Tier A full treatment | [x] |
| R23-3c | **Protocol Tabs 升級（5 files）** | AmendmentsTab / AttachmentsTab / CoEditorsTab / ReviewersTab / VersionsTab | [x] |

### Batch 4 — Admin Pages & Config Tabs（P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-4a | **Admin Tier A 頁面升級（4 files）** | InvitationsPage / ManagementReviewPage / ChangeControlPage / RiskRegisterPage | [x] |
| R23-4b | **Admin Config Tabs 升級（~8 files）** | DepartmentTab + AuditActivitiesTab + AuditAlertsTab + AuditSessionsTab + RoutingTable + QA 相關頁 | [x] |

### Batch 5 — Reports + HR Tables（P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-5a | **Reports 頁面 Table 升級（9 files）** | container / header / token-only row colors | [x] |
| R23-5b | **Reports Tab 元件升級（5 files）** | JournalEntries / TrialBalance / ProfitLoss / ApAging / ArAging | [x] |
| R23-5c | **HR Tables 升級（non-DataTable files）** | ConflictsTab / AttendanceHistoryTab 等 | [x] |

### Batch 6 — Animal Detail Tabs + 剩餘（P3）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R23-6a | **MyProjects / MyAmendments 升級** | Tier A full treatment | [x] |
| R23-6b | **Animal Detail Tabs 升級（8 tabs）** | BloodTestTab / ObservationsTab / WeightsTab / VaccinationsTab / SurgeriesTab / PathologyTab / PainAssessmentTab / VetRecommendationsTab | [x] |
| R23-6c | **Protocol content-section Tables** | PersonnelSection / CommentsTableView | [x] |

---

## 🛡️ R24 — Observability 補強與 IP-level Safety Gate（2026-04）

> 延伸 R22 攻擊偵測管線：補齊 4 項 gap（IP 自動封鎖 / 生產 Loki / Alertmanager infra 通知 / Grafana 安全 dashboard）。
> 盤點後確認 ipig_system 已有 80% observability 基礎（R22 完整攻擊偵測 + 4 種通知管道），本輪僅補剩餘缺口。
> 詳細計畫與決策紀錄：`docs/OBSERVABILITY_PLAN.md`
> 已作廢方案：獨立 dash 服務（`C:\System Coding\ipig-dashboard\DASH_SPEC.md`，保留作決策紀錄）

### 24-A IP-level Safety Gate（P0）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R24-1 | **IP blocklist + 自動封鎖 middleware** | `migrations/031_ip_blocklist.sql`（UUID+INET+partial unique index）；`middleware/ip_blocklist.rs` 掛於 `api_middleware_stack` 最外層（涵蓋 /api/v1 所有子路由；`/metrics`、`/api/health`、honeypot 於 /api/v1 外層 bypass）；來源 IP 復用 `middleware/real_ip.rs::extract_real_ip_with_trust`；整合 R22-6 IDOR probe（`response_logger.rs`）/ R22-5 auth ratelimit 升級（`rate_limiter.rs`）/ R22-16 honeypot → 自動封 IP；`/admin/audit/ip-blocklist` 路由 + AdminAuditPage 「IP 黑名單」Tab | [x] |

### 24-B 生產環境可觀測性（P1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R24-2 | **Loki + Promtail 生產部署** | `docker-compose.prod.yml` 新增 Loki + Promtail services（複用 `monitoring/promtail/config.yml`，加 relabel 只收 `ipig-(api\|web)`、加 `environment=prod` 靜態 label）；Loki 30d 保留（`storage.tsdb.retention.time` 需於 Loki config 設定，此輪用預設）；解鎖 R22-15 | [x] |

### 24-C 告警與儀表板（P2）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R24-3 | **Alertmanager infra 通知啟用** | `alertmanager.yml` default/critical receiver 改為 webhook → `http://api:3000/api/webhooks/alertmanager`（Bearer token 防護）；新增平檔 `handlers/alertmanager_webhook.rs`，轉 payload 為 `SecurityNotification` 呼叫 R22 `SecurityNotifier::dispatch()`；`Config::alertmanager_webhook_token` 從 env 讀取 | [x] |
| R24-4 | **Grafana security dashboard** | 新增 `deploy/grafana_security_dashboard.json`（6 panel：Alerts 時間線 / Active Blocklist / Top IPs / Login Anomaly / Honeypot Hits / 403 Rate via Loki）；`provisioning/datasources/loki.yml` + `postgres.yml` 新增；`migrations/032_grafana_readonly.sql` 建 `grafana_readonly` role + GRANT SELECT；`docker-compose.yml` Grafana 掛載新 dashboard JSON | [x] |

---

## 🔒 R25 — 安全基礎設施補強（2026-04-20）

> 延伸 E-系列安全審計：補齊 5 項 CI / infra / 監控層面的 gap。
> 來源：安全審計後續建議（N-1 ~ N-5）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R25-1 | **Trivy 容器掃描加入 CI** | 每次 build 掃 image CVE；現在只有 `cargo audit` 管 Rust 依賴，缺少 OS/base image 層級掃描；加入 `.github/workflows/` trivy-scan job，critical/high 自動 fail | [x] 已存在於 ci.yml:382-432 |
| R25-2 | **security.txt（RFC 9116）** | `/.well-known/security.txt` 提供漏洞回報聯絡管道；AI agent 與安全研究員標準查找點；可由 nginx 靜態服務或後端路由回傳 | [x] |
| R25-3 | **CSP report-uri 端點** | 目前 CSP 只攔截不回報；新增 `POST /api/v1/csp-report` 端點收集真實 XSS 嘗試，寫入 `security_alerts` 或獨立 log table | [x] |
| R25-4 | **Secret scanning in CI** | 加入 `git-secrets` 或 `truffleHog` 掃 commit，防止 API key / token 意外進版控；整合至 GitHub Actions pre-push 或 PR check | [x] gitleaks-action |
| R25-5 | **DB 查詢 statement timeout** | sqlx pool 有 `acquire_timeout`，但個別 query 沒有 statement timeout；長查詢可能打滿 pool；於 `DATABASE_URL` 加 `options=--statement_timeout%3D30000` 或在 pool 建立後執行 `SET statement_timeout` | [x] after_connect hook |

---

## 🔄 R26 — Service-driven Audit 重構延伸待辦（2026-04-21 審查報告產出）

> 對應 `docs/reviews/2026-04-21-rust-backend-review.md` 與 `plan-for-the-critical-validated-pebble.md`
> PR #1 INFRA 完成後發現的延伸優化項；主功能未壞，這些是「更穩健」升級。

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R26-1 | **長 Scheduler job 升級為 `tokio::select!` 中斷式** | PR #177 完成：`monthly_report` / `db_analyze` / `calendar_sync` 等長 job 升級為 `tokio::select!` 中斷式；併同 `main.rs` shutdown grace period 與安全中斷點 | [x] |
| R26-2 | **HMAC chain 每日驗證 cron** | 完成：`services/audit_chain_verify.rs` + `scheduler.rs::register_audit_chain_verify_job`（每日 02:00 UTC）+ `SecurityNotifier::dispatch` 斷鏈告警；payload 大小限制（top 20 IDs）；`AUDIT_CHAIN_VERIFY_ACTIVE` env 旗標；3 單元測試 | [x] |
| R26-3 | **現有 handler 遷移至 `log_activity_tx`**（97 call sites / 27 handler 檔） | 完成：animals 49（PR #4a-4g）+ user 8 + product 7 + sku 5 + partner/warehouse/equipment 12 + role/ai/auth/hr 12 + 其他 ≈ 全部遷移；跨越 PR #156/162-184/188/191 共計 20+ 個 PR | [x] |
| R26-4 | **舊 `log_activity(&pool, ...)` 最終移除** | 完成：`AuditService::log_activity` 舊版已刪除；`compute_and_store_hmac` 舊版已合併；零 deprecated 警告（`cargo clippy --all-targets -- -D warnings` 綠燈） | [x] |
| R26-5 | **(已完成) migration 036 changed_fields 聯集修正** | 對應 PR #154：stored proc fallback 由 JSONB EXCEPT 改為 UNION + `IS DISTINCT FROM`，正確偵測「被刪除的 key」。 | [x] |
| R26-6 | **HMAC chain 版本化 + 儲存後雜湊** | PR #170 完成：新增 `user_activity_logs.hmac_version SMALLINT`（`1`=legacy string-concat、`2`=length-prefix canonical）；verifier 依 version 分流；DataDiff 的 changed_fields 避免 stored proc fallback 路徑 | [x] |
| R26-7 | **Dead code 11 處逐一 review & 清理** | 完成：PR #173 刪除 8 處真死碼；本次清理剩餘 3 處（`IdxfMeta.format_version` + `ManifestTable.columns` 改為 `_`-prefix serde rename；`QUARTERLY_OVERTIME_LIMIT` 移除未用法規常數）；services 模組樹零 `#[allow(dead_code)]` | [x] |
| R26-8 | **完整 `ProtocolService::change_status` Service-driven 重構** | PR #188 完成：`change_status_tx` 將 10+ DB 操作、numbering、4 helper fn（assign_primary_reviewer/assign_vet_reviewer/record_activity/PartnerService::create_tx）納入單一 tx；跨服務原子性已建立 | [x] |
| R26-9 | **Audit redact allowlist for medical entities** | PR #175 完成：`CareRecord` / `VetAdviceRecord` / `AnimalObservation` 等醫療自由文字 entity 明確標記 `AuditRedact` impl（空 impl 需文檔證明無敏感欄位） | [x] |
| R26-10 | **Vet advice upsert 並發安全 + SDD audit** | PR #174 完成：`delete_vet_advice_record` 加 FOR UPDATE 鎖定；upsert pattern 補 SELECT FOR UPDATE；完整 SDD audit | [x] |
| R26-11 | **IDOR service-layer authz** | PR #176 完成：handler 直接 SQL 檢查身份下沉到 service 層；`services/access.rs` 集中授權 helper | [x] |
| R26-12 | _（保留編號）_ | 規劃階段曾預留為「edge case 修補」，後續實際工作均歸入 R26-13/14；保留編號維持歷史軌跡 | [x] |
| R26-13 | **storage_location 庫存 upsert 原子性 + audit** | PR #197 完成：原 `INSERT ... ON CONFLICT DO UPDATE` 無 before snapshot；改為 SELECT FOR UPDATE + 顯式 INSERT/UPDATE 分支 + `log_activity_tx` 在同一 tx 寫 audit | [x] |
| R26-14 | **Audit redaction 對照文檔 + CI guard** | PR #198 完成：`docs/security/AUDIT_REDACTION.md` 對照表（明確 redact / default empty / 不進 diff / 不存在 entity 分類）+ `.github/workflows/ci.yml::audit-redaction-guard`（find + awk 掃 FromRow struct 含敏感欄位） | [x] |

---

## 🔧 R27 — E2E 修復後的代碼品質改善（2026-04-24 全面代碼審查）

> 對應 PR #200、#201 E2E 測試修復後的全面代碼審查結果
> 優先級均為 LOW，無阻擋項，可後續漸進式改進

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| R27-1 | **Dockerfile CMD 可讀性改善** | `frontend/Dockerfile:72` 超長 CMD（240+ 字元）可考慮提取為獨立 shell 腳本；當前功能正確但後續維護時更易理解。LOW | [ ] |
| R27-2 | **生環境 API_BACKEND_URL 驗證** | `frontend/Dockerfile:72` envsubst 應驗證 `${API_BACKEND_URL}` 非空，避免生成無效 nginx 配置；CI 環境由 docker-compose.test.yml 保證，生環境應加額外檢查。LOW | [ ] |
| R27-3 | **auth_middleware 函式拆分** | `backend/src/middleware/auth.rs` `auth_middleware` ~90+ 行（含註解），超過 ≤60 寬鬆上限。建議拆 `validate_jwt(state, token) -> Claims`（token 提取 + ES256 decode + audience/issuer + jti 黑名單）+ `load_permissions(state, claims) -> Vec<String>`（admin 旁路 + try_get_with single-flight + 錯誤映射）。來源：CodeRabbit PR #210 outside-diff Major。LOW | [ ] |
| R27-4 | **middleware SQL 下放至 repository** | `backend/src/middleware/auth.rs` 內含 4-table JOIN（permissions JOIN role_permissions JOIN user_roles JOIN roles）+ `check_user_active_status` 的 SELECT，違反 CLAUDE.md 「Middleware 禁業務邏輯」「Repository 封裝 SQL」分層。建議移至 `repositories/user.rs`：`list_permission_codes_by_user` + `find_user_active_status_by_id`。同 SELECT 也能被 `services/access.rs` 復用。來源：CodeRabbit PR #210。LOW | [ ] |
| R27-5 | **permission_cache 觀測指標** | `backend/src/middleware/auth.rs` 的 moka cache 沒有 hit/miss/eviction 計數，無法判斷 capacity 10,000 是否足夠、TTL 5min 是否合適。建議在 `try_get_with` 包 wrapper 取 `entry_count()` / `weighted_size()` 並推到既有 `metrics_handle` (Prometheus)。來源：CodeRabbit PR #210。LOW | [ ] |
| R27-6 | **admin 路徑帳號狀態 cache** | `backend/src/middleware/auth.rs::check_user_active_status` 對 admin 每請求查 DB（admin 不走 perm cache）。雖 admin 數量小，但屬均勻優化機會：把 admin 也納入 `try_get_with`（cache 空 Vec），或單獨 `Cache<Uuid, ()>` 快取狀態檢查結果。來源：Gemini PR #210 Medium。LOW | [ ] |
| R27-7 | **amendment::classify 函式拆分** | `backend/src/services/amendment/workflow.rs::classify` ~111 行（>60 寬鬆上限）。Major 與 Minor 分支可拆 `classify_minor_with_signature_tx` + `classify_major_with_reviewers_tx`，主函式僅做驗證 + 分流。來源：CodeRabbit PR #205 outside-diff Major。LOW | [ ] |
| R27-8 | **C2 R7 已獨立修補** | record_decision 終態守衛已由 PR #213 (`glp/c2-extra-decision-terminal-guard`) 處理，本項僅作紀錄追蹤；PR #213 合併後可關閉。LOW | [x] |
| R27-9 | **amendment record_decision 重複查 status** | `backend/src/services/amendment/workflow.rs::record_decision` 終態守衛 SELECT FOR UPDATE 已取得 `current_status`；隨後呼叫的 `check_all_decisions_tx` 內部又重新查一次（`get_by_id_raw` 等）。同 tx 內可省一次往返，把 `current_status` 作為參數傳進 `check_all_decisions_tx`。來源：Gemini PR #216 Medium。LOW | [ ] |
| R27-10 | **animal observation create handler 重複 get_by_id** | `backend/src/handlers/animal/observation.rs::create_animal_observation`（L109 + L139）對同一 animal 重複呼叫 `AnimalService::get_by_id`。可單次查詢後傳遞。來源：Gemini PR #216 Medium。LOW | [ ] |

---

## 📊 待辦統計

| 優先級 | 數量 (未完成) |
|--------|------|
| 🚨 P0 上線前必要 | 0 |
| 🟡 P1 上線前建議 | 0 |
| 🔴 P2 中優先 | 0 |
| 🔵 P3 低優先 | 1 |
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
| 🔍 R16 全專案 Code Review | 0 |
| 🔒 R17 CSO 安全審計 | 0 (1 已接受, 3 完成) |
| 🫀 R18 Heartbeat 自動化維護 | 0 (4 完成) |
| 🎫 R19 客戶邀請制入口 | 0 (14 完成) |
| 🤖 R20 AI 預審與執行秘書標註 | 2 (8 完成, R20-9/10 持續性) |
| 🌡️ R21 環境監控子系統（MES-Lite） | 11 (1 暫緩) |
| 🛡️ R22 攻擊偵測與主動告警 | 0 (17 完成, 1 暫緩) |
| 🎨 R23 全站 Table UI 升級 | 0 (20 完成) |
| 🛡️ R24 Observability 補強 | 0 (4 完成) |
| 🔒 R25 安全基礎設施補強 | 0 (5 完成) |
| 🔄 R26 Service-driven Audit 重構延伸 | 0 (14 完成；含 R26-12 保留編號) |
| 🔧 R27 E2E + bot review 後續清理 | 9 |
| **合計（未完成）** | **22** |

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
| 2026-03-01 | 🧠 Claude：P0–P2 改進計劃全部完成 — P1-M0 稽核匯出 API、P1-M1 API 版本、P1-M2 GDPR、P1-M3 OPERATIONS.md、P1-M4 憑證輪換、P1-M5 Dependabot；P2-M2 人員訓練紀錄、P2-M3 設備校準、P2-M4 稽核 UI 使用者篩選、P2-M5 security/SOC2_READINESS.md。詳見 `docs/development/IMPROVEMENT_PLAN_MARKET_REVIEW.md` |
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
| 2026-02-25 | 🧠 Claude：完成 P1-7 電子簽章合規審查（21 CFR Part 11），新增 `docs/security/ELECTRONIC_SIGNATURE_COMPLIANCE.md`。 |
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
