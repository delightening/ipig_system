# iPig 豬博士動物科技系統 — Nest-based 模組化規格說明書

> **版本**：8.0 (2026-03-01)
> **狀態**：功能完整，上線準備中
> **核心架構**：Rust (Axum) + React (Vite) + PostgreSQL

---

## 1. 系統總覽 (Project Overview)

iPig 系統是一套專為實驗動物管理設計的**統一入口門戶 (Unified Portal)**。系統採模組化架構，將「計畫書審查 (AUP)」、「進銷存 (ERP)」、「動物醫療紀錄 (Animal Management)」、「設施管理 (Facility)」、「人事行政 (HR)」與「稽核安全 (Audit & Security)」整合於單一平台，並嚴格遵循 GLP (Good Laboratory Practice) 合規要求。

---

## 2. 核心模組架構 (Nest-based Modules)

系統依功能邊界劃分為以下「巢狀模組」，每個模組包含獨立的處理層 (Handlers)、服務層 (Services) 與資料模型 (Models)。

### 2.1 認證與安全模組 (Auth & Security Nest)
- **Identity**: JWT (HttpOnly Cookie) + CSRF Token 雙重認證，支援多端點登入。
- **RBAC**: 精細的權限控制矩陣 (Role-Based Access Control)，含動態角色/權限管理 API。
- **Security**:
  - `Login Tracker`: 偵測異常登入、帳號鎖定與安全警報 (Security Alerts)。
  - `Rate Limiter`: 認證端點 10 次/分、一般 API 120 次/分。
  - `GeoIP`: 結合 MaxMind GeoLite2-City 進行地理位置稽核。
  - `Session Manager`: 即時工作階段監控、強制登出 (Force Logout) 與 SSE 即時安全警報推送。
  - `JWT Blacklist`: 已撤銷 Token 即時失效控制 (SHA-256 雜湊)。
  - `CSRF Middleware`: 針對寫入操作的 CSRF 防護。
  - `Real IP Middleware`: 信任反向代理 X-Forwarded-For 標頭策略。
  - `Impersonate`: 管理員模擬登入（含 impersonated_by JWT 欄位、30 分鐘時限、完整稽核日誌）。
- **Audit Trail**:
  - `Activity Logger`: 自動記錄所有使用者操作（含 Entity 歷史追蹤）。
  - `Audit Dashboard`: 安全事件總覽、登入事件、Session 管理。
  - `Audit Integrity`: 稽核紀錄完整性驗證 (HMAC)。
  - `Partition Maintenance`: 活動日誌分區自動維護。

### 2.2 AUP 計畫書審查模組 (Protocol & Review Nest)
- **AUP Lifecycle**: 從草稿撰寫、自動儲存到多層級審查（預審、獸醫審查表、IACUC 審查）。
- **Amendment**: 計畫核准後的變更申請流程，支援 MAJOR/MINOR 分類、版本控管與審查歷程追蹤。
- **Collaboration**: 共同編輯者 (Co-editors) 機制、草稿意見回覆 (Draft Reply) 與審查意見解決。
- **Electronic Signatures**: GLP 合規電子簽章 (Protocol Review Signing)。
- **Compliance**: 版本控管、歷史快照、PDF 自動匯出。

### 2.3 實驗動物管理模組 (Animal Management Nest)
- **Life Cycle**: 動物進場 (Sources)、分配 (Batch Assign)、實驗追蹤至安樂死、犧牲或轉讓。
- **Medical Records**:
  - `Observations`: 臨床觀察、治療紀錄、版本歷程、獸醫已讀標記。
  - `Surgeries`: 手術過程、生理數值、麻醉紀錄、版本歷程。
  - `Blood Tests`: 血檢紀錄，整合檢驗項目模板 (Templates) 與組合 (Panels) 管理。
  - `Weights`: 體重追蹤紀錄。
  - `Vaccinations`: 疫苗接種紀錄。
  - `Care Records`: 疼痛評估紀錄（Pain Assessment）。
  - `Vet Recommendations`: 獸醫建議（含附件上傳）。
- **Sacrifice & Pathology**: 犧牲紀錄與病理報告（含電子簽章）。
- **Sudden Death**: 異常死亡（猝死）登記與流程處理。
- **Euthanasia**: 安樂死審批流程（申請 → 核准 → 上訴 → 執行），含多角色簽核與電子簽章。
- **Transfers**: 動物轉讓工作流（獸醫評估 → 計畫分配 → 核准 → 完成/拒絕），含資料隔離 (Data Boundary)。
- **Import/Export**: 批次匯入模板（基本資料/體重）、單筆/專案醫療資料匯出。
- **Electronic Signatures**: 犧牲、觀察、安樂死、轉讓紀錄的 GLP 合規電子簽章。
- **Annotations**: 紀錄註記功能（支援多種紀錄類型）。

### 2.4 設施管理模組 (Facility Management Nest)
- **Species**: 物種主檔管理（含生理參數）。
- **Buildings**: 建築物管理。
- **Zones**: 區域管理。
- **Pens**: 欄舍管理。
- **Departments**: 部門管理。
- **Facility Hierarchy**: 建築 → 區域 → 欄舍的層級關聯。

### 2.5 進銷存模組 (ERP / Inventory Nest)
- **Products**: 產品主檔管理（含 SKU 自動編碼、分類管理）。
- **Warehouses**: 倉庫管理（含倉庫佈局 Layout）。
- **Storage Locations**: 倉庫儲位管理（含庫存品項、跨儲位調撥）。
- **Partners**: 交易夥伴管理（供應商/客戶，含自動編碼）。
- **Documents**: 進銷存單據管理（建立、提交、核准、取消工作流）。
- **Inventory**: 即時庫存查詢 (On-Hand)、庫存異動帳 (Stock Ledger)、低庫存警報。
- **Alerts**: 低庫存與到期日警報（含管理員手動觸發）。
- **Treatment Drugs**: 藥物選項管理（支援從 ERP 產品匯入）。

### 2.6 人事行政模組 (HR Nest)
- **Attendance**: 打卡系統（支援 IP 白名單 CIDR 驗證 + GPS 定位半徑驗證）、統計報表、異常校正。
- **Overtime**: 加班申請流程（建立 → 提交 → 核准/拒絕）。
- **Leave**: 請假管理（建立 → 提交 → 核准/拒絕/取消，支援附件上傳）。
- **Leave Balances**: 年假額度、補休時數、餘額摘要、到期補償、手動調整。
- **Dashboard Calendar**: HR 行事曆總覽（整合 FullCalendar）。
- **Google Calendar Sync**: 雙向同步至 Google Calendar（設定、連線/斷線、手動觸發、衝突解決）。
- **Staff Management**: 員工清單與代理人機制。

### 2.7 報表模組 (Reports Nest)
- **ERP Reports**: 庫存現況、庫存異動、採購明細、銷貨明細、成本摘要。
- **Animal Reports**: 血檢成本報表、血檢分析報表。
- **Scheduled Reports**: 定期報表排程（CRUD、執行歷程、下載）。
- **Export**: 活動日誌匯出 (Excel)。

### 2.8 通知模組 (Notification Nest)
- **In-App**: 站內通知列表、未讀計數、標為已讀/全部已讀。
- **Email**: SMTP 郵件通知（各類事件自動觸發）。
- **Notification Routing**: 管理員可設定事件類型對應的通知角色與方式。
- **Notification Settings**: 使用者個別通知偏好設定。
- **Notification Cleanup**: 過期通知自動清理（排程或手動觸發）。

---

## 3. 技術堆疊 (Technical Stack)

### 後端 (Backend - Rust Nest)
- **Axum 0.7**: 高效能異步 Web 框架，單一 `api_routes` 函數統一路由嵌套（943 行）。
- **SQLx 0.8**: 具備編譯期 SQL 驗證 (`SQLX_OFFLINE=true`) 的異步 PostgreSQL 驅動。
- **Tower / Tower-HTTP**: Rate Limiting, CORS, Trace, Request ID, Set-Header 等中間件層。
- **Tokio**: 工業級非同步 Runtime。
- **jsonwebtoken**: JWT 驗證與簽發。
- **Argon2**: 密碼雜湊。
- **utoipa + utoipa-swagger-ui**: OpenAPI 3.0 規格書與 Swagger UI。
- **lettre**: SMTP 郵件發送。
- **rust_xlsxwriter / calamine / csv**: Excel 匯出匯入與 CSV 解析。
- **tokio-cron-scheduler**: 排程任務（低庫存檢查、到期日檢查、通知清理、分區維護）。
- **sha2 / hmac**: 稽核完整性驗證與 Token 雜湊。
- **maxminddb**: GeoIP 地理位置查詢。
- **printpdf**: PDF 生成。
- **reqwest**: HTTP Client（Google Calendar API 等外部整合）。

### 前端 (Frontend - React Nest)
- **React 18**: 組件化 UI 框架。
- **TanStack Query v5 + Table v8**: 處理 Server State 與複雜表格。
- **Zustand 4**: 輕量化全域狀態管理。
- **shadcn/ui**: 基於 Radix UI 的高度可自定義元件庫（25+ UI 元件）。
- **React Hook Form + Zod**: 表單管理與驗證。
- **i18next + react-i18next**: 多語系 (中/英) 國際化支援。
- **FullCalendar**: HR 行事曆與 Google Calendar 整合。
- **Recharts**: 圖表視覺化。
- **signature_pad**: 手寫電子簽章。
- **html2canvas + jspdf**: 前端 PDF 生成。
- **xlsx**: Excel 檔案處理。
- **@dnd-kit**: 拖曳排序功能。
- **react-grid-layout**: 可自定義儀表板佈局。
- **Tailwind CSS 3**: 原子化 CSS 框架。
- **Lucide React**: 圖示庫。

### 資料庫 (Database)
- **PostgreSQL 16**: 19 個遷移檔 (001_ ~ 019_)，涵蓋核心、權限、動物管理、AUP、HR、稽核、ERP 倉庫、補充、藥物選項、JWT 黑名單、稽核完整性、Optimistic Locking、TOTP 2FA、系統設定、複合索引、DB 維護、enum cast 修正等。

---

## 4. API 路由嵌套結構 (API Nesting Structure)

API 採用 `/api` 為根路徑，所有路由透過中間件層級控制：

```
/api
├── [PUBLIC] (Rate Limited - Auth)
│   ├── /auth/login
│   ├── /auth/refresh
│   ├── /auth/2fa/verify
│   ├── /auth/forgot-password
│   └── /auth/reset-password
│
└── [PROTECTED] (Auth + CSRF + Rate Limited - API)
    ├── /auth/logout, /auth/stop-impersonate, /auth/heartbeat
    ├── /auth/2fa/setup, /auth/2fa/confirm, /auth/2fa/disable
    ├── /me, /me/password, /me/preferences
    ├── /users, /users/:id, /users/:id/impersonate
    ├── /roles, /permissions
    │
    ├── /warehouses, /storage-locations (含庫存/調撥)
    ├── /products, /categories, /sku
    ├── /partners, /documents (進銷存單據工作流)
    ├── /inventory/on-hand, /inventory/ledger, /inventory/low-stock
    │
    ├── /protocols (AUP 審查生命週期)
    ├── /reviews (審查分配、意見、草稿回覆、獸醫表單)
    ├── /amendments (變更申請工作流)
    ├── /my-projects
    │
    ├── /animal-sources
    ├── /animals (CRUD、批次分配、匯入匯出)
    ├── /animals/:id/observations, /surgeries, /weights, /vaccinations
    ├── /animals/:id/blood-tests, /blood-test-templates, /blood-test-panels
    ├── /animals/:id/sacrifice, /pathology, /sudden-death, /care-records
    ├── /animals/:id/transfers, /transfers/:id (轉讓工作流)
    ├── /euthanasia/orders (安樂死審批流程)
    │
    ├── /signatures (電子簽章: sacrifice/observation/euthanasia/transfer/protocol)
    ├── /annotations (紀錄註記)
    │
    ├── /facilities (species/buildings/zones/pens/departments)
    │
    ├── /hr/attendance (打卡: IP + GPS 驗證)
    ├── /hr/overtime, /hr/leaves (加班/請假工作流)
    ├── /hr/balances (年假/補休/摘要/調整)
    ├── /hr/calendar (Google Calendar 同步)
    ├── /hr/dashboard, /hr/staff
    │
    ├── /notifications, /alerts
    ├── /reports, /scheduled-reports, /report-history
    ├── /attachments (檔案上傳/下載)
    │
    ├── /admin/config-warnings
    ├── /admin/audit (活動日誌/登入/Session/安全警報/SSE)
    ├── /admin/notification-routing
    └── /admin/treatment-drugs
```

---

## 5. 最近進度 (Recent Progress)

- **2026-03-01**: 修復複製後編輯觀察紀錄 500 錯誤（migration 019 修正 `version_record_type` enum cast 遞迴問題）。
- **2026-02-28**: 第四輪改善計畫 R4 完成（IDOR 修補、CSP、Error Boundary、備份驗證等 20 項）。
- **2026-02-28**: 手寫簽名 Canvas 寬度無限擴張修復、ProtocolEditPage Section 導航改用 URL Search Params。
- **2026-02-28**: 系統改善 14 項（Docker 網路隔離、N+1 修復、Optimistic Locking、2FA、WAF 等）。
- **2026-02-28**: 全部待辦項目完成（P0~P5 零缺口），Storybook、TOTP 2FA、WAF overlay 交付。
- **2026-02-28**: Prometheus + Grafana 部署、後端 API 整合測試 25+ cases、效能基準報告。
- **2026-02-27**: E2E 34/34 連續通過（storageState 免重複登入、rate limit 600/min）。
- **2026-02-25**: SEC-33 敏感操作二級認證、電子簽章合規審查、資料保留政策、OpenAPI ≥90%。

---

## 6. 已完成安全強化 (Completed Security Hardening)

| 編號 | 等級 | 項目 | 狀態 |
|------|------|------|------|
| SEC-01 | P0 | Refresh Token SHA-256 雜湊 | ✅ |
| SEC-02 | P2 | Token HttpOnly Cookie 遷移 | ✅ |
| SEC-03 | P0 | 移除硬編碼密碼（改環境變數） | ✅ |
| SEC-04 | P1 | API Rate Limiting | ✅ |
| SEC-06 | P0 | 開發帳號強制改密 | ✅ |
| SEC-07 | P1 | Nginx 安全標頭 (5 項) | ✅ |
| SEC-08 | P1 | Docker 非 root 運行 | ✅ |
| SEC-09 | P2 | JWT 預設 6 小時（可配置） | ✅ |
| SEC-10 | P1 | 密碼強度驗證 | ✅ |
| SEC-11 | P2 | Impersonate 安全增強 | ✅ |
| SEC-16 | P2 | 隱藏 Nginx 版本號 | ✅ |
| SEC-29 | — | CSRF Token 中間件 | ✅ |
| SEC-30 | — | IP Header 信任策略 | ✅ |
| SEC-31 | — | CORS 允許 Origin 設定 | ✅ |
| SEC-32 | — | JWT 分鐘級過期設定 | ✅ |
| SEC-33 | P3 | 敏感操作二級認證 (reauth token) | ✅ |
| SEC-39 | P5 | TOTP 2FA 全端實作 | ✅ |
| SEC-40 | P5 | WAF (ModSecurity + OWASP CRS) overlay | ✅ |

---

## 7. 正式上線準備 Checklist (Production Readiness)

本章節列出系統從「開發完成」邁向「正式上線 (General Availability, GA)」所需滿足的品質與合規標準。

### 7.1 測試覆蓋率 (Testing Coverage)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| Rust 單元測試 | 142 個通過 | 核心業務邏輯覆蓋率 ≥ 80% | ✅ |
| 後端 API 整合測試 | 25+ cases (6 檔案) | 關鍵流程覆蓋 | ✅ |
| 前端 E2E 測試 | Playwright 7 spec / 34 tests | 登入/AUP/動物/Admin 等關鍵流程 | ✅ |
| E2E CI 自動化 | `docker-compose.test.yml` + GitHub Actions | CI 整合 | ✅ |
| Python 整合測試 | 8 模組 (137 檔) | 輔助驗證 | ✅ |

### 7.2 可觀測性 (Observability)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| 健康檢查端點 `/health` | ✅ 已實作 | DB 連通性 + 延遲量測 | ✅ |
| 結構化日誌 (JSON) | ✅ 條件式 JSON | JSON 格式 + Request ID 全鏈路追蹤 | ✅ |
| Metrics 端點 `/metrics` | ✅ Prometheus 已實作 | HTTP 指標 + DB Pool 狀態 | ✅ |
| 錯誤監控 (Sentry 等) | 無（不規劃導入） | 前後端錯誤即時通知 + 堆疊追蹤 | 🔴 |
| 啟動配置檢查 | ✅ 已實作 | — | ✅ |

### 7.3 備份與災難復原 (Backup & DR)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| 資料庫自動備份 | ✅ pg_dump + cron + rsync | — | ✅ |
| 備份加密 | ✅ GPG 加密 | 透過 BACKUP_GPG_RECIPIENT 啟用 | ✅ |
| 復原演練 | ✅ DR_RUNBOOK.md | RPO < 1h、RTO < 4h、演練記錄表 | ✅ |
| 上傳檔案備份 | ✅ rsync 已整合 | db-backup 容器自動同步 /uploads | ✅ |
| GeoIP 資料更新 | ✅ 更新腳本 | scripts/update_geoip.sh + SHA256 驗證 | ✅ |

### 7.4 安全性補強 (Security Hardening)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| Rust 依賴掃描 (`cargo audit`) | ✅ CI 已整合 | — | ✅ |
| npm 依賴掃描 (`npm audit`) | ✅ CI 已整合 | — | ✅ |
| 容器安全掃描 (Trivy) | ✅ CI 已整合 | CI 中掃描 Docker image | ✅ |
| Session 逾時預警 | 前端 60s 倒數 Dialog + 續期 | 使用者友善 | ✅ |
| TOTP 2FA | 全端實作（setup/confirm/verify/disable） | 管理員可選 | ✅ |
| WAF overlay | ModSecurity + OWASP CRS | 選用部署 | ✅ |
| Named Tunnel 腳本 | 已提供 | Cloudflare Named Tunnel | ✅ |

### 7.5 GLP 合規文件 (Regulatory Compliance)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| 電子簽章合規 (21 CFR Part 11) | 審查文件已產出 | 法規合規審查 | ✅ |
| 稽核紀錄不可刪改 | ✅ HMAC 驗證 + 無 delete API | — | ✅ |
| 資料保留政策 | ✅ DATA_RETENTION_POLICY.md | 各類紀錄法定保留年限 | ✅ |
| GLP 驗證文件 | IQ/OQ/PQ 框架已建立 | 驗證報告 | 🔶 |

### 7.6 效能基準 (Performance Baseline)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| API 回應時間 | k6 基準 P95: 1.76~2.31ms | P95 < 500ms（一般） | ✅ |
| 壓力測試 | k6 50 VU、正式基準報告 | 效能基準文件化 | ✅ |
| Nginx Brotli 壓縮 | ✅ 壓縮等級 6 + Vary | font/svg 類型 | ✅ |
| 前端 Bundle 大小 | 主 chunk 優化、jsPDF 動態導入 | — | ✅ |

### 7.7 使用者文件與教育訓練 (Documentation & Training)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| API 文件 (OpenAPI/Swagger) | ≥ 90% 端點文件化 | 認證/AUP/動物/電子簽章等 | ✅ |
| 使用者操作手冊 | USER_GUIDE.md v2.0（9 章節） | 各模組操作指南 | ✅ |
| 管理員部署/維運手冊 | DEPLOYMENT.md | 完整部署、備份、復原、監控 | ✅ |

### 7.8 UX / 相容性 (User Experience)

| 項目 | 現況 | 上線目標 | 狀態 |
|------|------|----------|------|
| 響應式設計 | ✅ 基本行動端適配 | — | ✅ |
| 錯誤處理 UX | 統一 API 錯誤處理、toast 提示 | 友善化 + 操作指引 | ✅ |
| Loading 狀態 | Skeleton、TableSkeleton、LoadingOverlay | 非同步操作回饋 | ✅ |
| 瀏覽器相容性 | Playwright Chromium 驗證 | 跨瀏覽器 opt-in | ✅ |

---

## 8. 上線策略 (Go-Live Strategy)

| 階段 | 目標 | 需完成項目 | 預估時程 |
|------|------|-----------|----------|
| **Alpha 內測** | 核心團隊 3-5 人 | 健康檢查端點、備份復原演練、關鍵 Bug 修復 | 1-2 週 |
| **Beta 試營運** | 單一部門 10-20 人 | 測試覆蓋率 ≥ 80%、可觀測性、安全掃描、操作手冊 | 4-6 週 |
| **正式上線 (GA)** | 全機構上線 | 全部 Checklist 完成、GLP 合規文件、教育訓練 | 2-4 週 |

---

## 9. 長期演進路線圖 (Long-term Roadmap)

### 階段一：效能優化與高可用 (Performance & HA)

1.  **多層級快取 (Redis)**：JWT 黑名單/Session 移至 Redis、ERP 主檔 TTL 快取、活動日誌異步寫入。
2.  **資料庫讀寫分離**：Read-only 副本連線池、`EXPLAIN ANALYZE` 索引微調、活動日誌月度分區。
3.  **Nginx Brotli 壓縮**：前端靜態資源長期緩存策略。

### 階段二：智能化功能與生態擴展 (Intelligent Features)

1.  **多物種動態表單**：JSON Schema 驅動、`species` 表擴展生理參數閾值。
2.  **AI 健康監測**：體重趨勢/臨床觀察異常檢測，自動觸發獸醫警報。
3.  **外部系統對接 (Webhook/API)**：SAP/Oracle 推送、LIMS 血檢上傳 API Key。
4.  **PWA 離線支援**：離線快取打卡/觀察草稿、掃碼入庫相機優化。

---

## 8. 容量與資源需求評估 (Capacity & Resource Assessment)

針對「**100 隻豬、每日 1 張照片 + 1 筆文字紀錄**」的營運場景，評估一年的資源成長需求：

### 8.1 儲存空間需求 (Storage Capacity)
*   **照片紀錄 (Blob Storage)**:
    *   根據 `FileService` 配置，動物照片上限為 10MB。實務平均每張照片採 2MB 計。
    *   **日增量**: 100 隻 × 2MB = **200 MB/日**。
    *   **年需求**: 365 天 × 200MB ≈ **73 GB/年**。 (若照片平均 5MB，則需約 **182 GB/年**)。
*   **文字記錄 (Database Text)**:
    *   每筆觀察/手術紀錄平均 2KB。
    *   **日增量**: 100 筆 × 2KB = **200 KB/日**。
    *   **年需求**: 365 天 × 200KB ≈ **73 MB/年** (對於儲存空間佔比極低)。
*   **建議配置**: 初期硬體建議預留 **250GB - 500GB** 磁碟，或優先採用 S3 相容物件儲存以利無限擴展。

### 8.2 資料庫效能評估 (Database & Indexing)
*   **紀錄總數**: 一年累計新增 **36,500 筆** 醫療紀錄。
*   **檢索資源**:
    *   PostgreSQL 對於 10 萬筆以下的資料量，使用 B-Tree 索引對 `animal_id` 與 `event_date` 進行查詢，延遲可維持在 **1ms - 10ms**。
    *   **索引大小**: 年增量約 **20-50 MB**，記憶體快取足以應付。
*   **併發負載**: 每日 100 次上傳，即使集中於尖峰 1 小時內，平均 36 秒 1 筆請求，現有 Axum/Tokio 架構可輕鬆應對。

### 8.3 流量需求 (Network Throughput)
*   **上傳頻寬**: 集中尖峰時段每秒約 **0.5 - 2 Mbps**。
*   **檢閱流量**: 當獸醫進行批次紀錄查閱時，讀取 100 張照片約需下載 200MB 資料，建議配置 **100Mbps** 以上的內部頻寬或啟用前端縮圖預覽機制以優化體驗。

---

## 10. 部署規範 (Deployment)

- **Docker Compose**: 5 服務架構：
  | 服務 | 容器名稱 | 說明 |
  |------|----------|------|
  | `db` | ipig-db | PostgreSQL 16-Alpine，含 Healthcheck |
  | `api` | ipig-api | Rust Axum API（非 root 運行） |
  | `web` | ipig-web | React 生產環境 (Nginx 靜態服務) |
  | `web-dev` | ipig-web-dev | Vite 開發伺服器 (Node 22-Alpine) |
  | `db-backup` | ipig-db-backup | 排程資料庫備份（cron + pg_dump，含 rsync 遠端備份） |

- **前端靜態邊界**：ipig-web 僅提供靜態資源，不解析使用者上傳之圖片；若有圖片處理需求，由獨立可升級之服務負責（方案 D，見 [docs/security-compliance/security.md](../security-compliance/security.md)）。
- **Nginx**: 作為反向代理處理 SSL 終止、緩衝優化與安全標頭。
- **Volumes**: `postgres_data`、`db_backups`、`/uploads`（檔案上傳）、`/geoip`（GeoIP 資料庫）。
- **Secrets**: Google Calendar Service Account JSON。
- **環境配置**: 支援 `.env` 統一管理（含 CORS、Cookie 安全、GPS 定位、IP 白名單等 30+ 參數）。

---

## 11. 代碼量概覽 (Codebase Metrics)

| 層級 | 項目 | 數量 |
|------|------|------|
| 後端 Handlers | 模組數 | 28 (含 3 子模組目錄) |
| 後端 Services | 模組數 | 33 (含 8 子模組目錄) |
| 後端 Models | 模組數 | 21 (含 1 子模組目錄) |
| 後端 Middleware | 模組數 | 7 |
| 後端 Bin 工具 | 工具數 | 11 |
| 後端 Routes | 總行數 | 943 |
| 後端 Migrations | 檔案數 | 19 (001_ ~ 019_) |
| 前端 Pages | 頁面目錄 | 14 |
| 前端 Components | 元件數 | 67+ (含 9 子目錄) |
| 前端 Types | 型別檔 | 14 |
| 整合測試 | Python + Rust API | tests/ + backend/tests/ |
| 單元測試 | Rust 測試 | 119+ passed |
| E2E 測試 | Playwright | 7 spec / 34 tests |
| OpenAPI 文件化 | Swagger 端點 | ≥ 90% |

---
*文件更新於 2026-03-01*
