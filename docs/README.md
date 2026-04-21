# 豬博士 iPig 系統 — 文件索引

> **最後更新：** 2026-04-21  
> 本目錄為 `docs/` 總索引。依角色選擇入口：**開發** → dev/；**維運** → ops/ · deploy/；**規格** → spec/；**安全合規** → security/；**使用者** → user/

---

## 📁 目錄結構

```
docs/
├── README.md              ← 本索引
├── TODO.md                ← 待辦任務（CLAUDE.md 定義，勿刪）
├── PROGRESS.md            ← 進度 + §9 變更日誌（CLAUDE.md 定義，勿刪）
│
├── spec/                  # 技術規格 v7.x
│   ├── architecture/      # 架構、DB Schema、API、RBAC、ERD
│   ├── modules/           # 各子系統模組規格
│   └── guides/            # 命名、稽核、儲存、UI/UX
│
├── dev/                   # 開發者日常（整合測試、CI、E2E）
├── ops/                   # 維運（操作手冊、Compose、環境、SKU）
├── runbooks/              # 事故應對（DR、產品回滾）
├── db/                    # 資料庫（Migration、Rollback、匯入）
├── security/              # 安全合規（威脅模型、GLP、SOC2、Rotation）
├── deploy/                # 部署（DEPLOYMENT、NAS 遷移）
├── user/                  # 終端使用者（QUICK_START、USER_GUIDE）
│
└── archive/               # 歷史快照（寫入後只讀）
    ├── code-reviews/
    ├── improvement-plans/
    ├── assessments/
    ├── compliance-reports/
    ├── heartbeat/
    ├── feature-designs/
    └── legacy/
```

---

## 🚀 使用者 (user/)

| 文件 | 說明 |
|------|------|
| [user/QUICK_START.md](./user/QUICK_START.md) | 快速啟動（Docker / 本地開發 / E2E 測試） |
| [user/USER_GUIDE.md](./user/USER_GUIDE.md) | 使用者操作手冊（登入、AUP、動物管理、ERP） |

---

## 🏗️ 技術規格 (spec/)

| 文件 | 說明 |
|------|------|
| [spec/README.md](./spec/README.md) | Profiling Spec v7.x 索引 |
| [spec/architecture/ARCHITECTURE.md](./spec/architecture/ARCHITECTURE.md) | 架構圖與資料流 |
| [spec/architecture/SYSTEM_RELATIONSHIPS.md](./spec/architecture/SYSTEM_RELATIONSHIPS.md) | 模組關聯與法規遵循總覽 |
| [spec/architecture/01_ARCHITECTURE_OVERVIEW.md](./spec/architecture/01_ARCHITECTURE_OVERVIEW.md) | 系統目的、技術堆疊、分層架構 |
| [spec/architecture/02_CORE_DOMAIN_MODEL.md](./spec/architecture/02_CORE_DOMAIN_MODEL.md) | 實體定義、關係、商業規則 |
| [spec/architecture/03_MODULES_AND_BOUNDARIES.md](./spec/architecture/03_MODULES_AND_BOUNDARIES.md) | 模組拆分、API 前綴、檔案結構 |
| [spec/architecture/04_DATABASE_SCHEMA.md](./spec/architecture/04_DATABASE_SCHEMA.md) | 資料表定義、ENUM、遷移統計 |
| [spec/architecture/05_API_SPECIFICATION.md](./spec/architecture/05_API_SPECIFICATION.md) | 完整端點列表、認證、回應格式 |
| [spec/architecture/06_PERMISSIONS_RBAC.md](./spec/architecture/06_PERMISSIONS_RBAC.md) | 角色、權限、存取控制矩陣 |
| [spec/architecture/07_SECURITY_AUDIT.md](./spec/architecture/07_SECURITY_AUDIT.md) | 安全中間件、2FA、WAF、GLP 合規 |
| [spec/architecture/09_EXTENSIBILITY.md](./spec/architecture/09_EXTENSIBILITY.md) | 已完成功能、未來規劃、擴展指南 |
| [spec/architecture/database_erd.md](./spec/architecture/database_erd.md) | 完整 ER 圖（Mermaid）|
| **modules/** | |
| [spec/modules/AUP_SYSTEM.md](./spec/modules/AUP_SYSTEM.md) | AUP 提交與審查系統 |
| [spec/modules/AUP.md](./spec/modules/AUP.md) | AUP 表單欄位規格（詳細） |
| [spec/modules/ERP_SYSTEM.md](./spec/modules/ERP_SYSTEM.md) | ERP 進銷存模組 |
| [spec/modules/ANIMAL_MANAGEMENT.md](./spec/modules/ANIMAL_MANAGEMENT.md) | 動物管理模組 |
| [spec/modules/HR_SYSTEM.md](./spec/modules/HR_SYSTEM.md) | 人事管理模組 |
| [spec/modules/08_ATTENDANCE_MODULE.md](./spec/modules/08_ATTENDANCE_MODULE.md) | 出勤模組（打卡、請假、加班） |
| [spec/modules/NOTIFICATION_SYSTEM.md](./spec/modules/NOTIFICATION_SYSTEM.md) | 通知系統 |
| [spec/modules/MCP_REVIEW_SERVER.md](./spec/modules/MCP_REVIEW_SERVER.md) | MCP AI 接入審查流程規格 |
| [spec/modules/AI_API_GUIDE.md](./spec/modules/AI_API_GUIDE.md) | 外部 AI 資料查詢接口使用指南 |
| [spec/modules/EMS_DEPLOYMENT.md](./spec/modules/EMS_DEPLOYMENT.md) | 環境監控子系統硬體部署（R21）|
| **guides/** | |
| [spec/guides/AUDIT_LOGGING.md](./spec/guides/AUDIT_LOGGING.md) | 稽核日誌規範 |
| [spec/guides/NAMING_CONVENTIONS.md](./spec/guides/NAMING_CONVENTIONS.md) | 命名慣例 |
| [spec/guides/STORAGE_SETUP.md](./spec/guides/STORAGE_SETUP.md) | 儲存設定 |
| [spec/guides/UI_UX_GUIDELINES.md](./spec/guides/UI_UX_GUIDELINES.md) | UI/UX 設計準則 |

---

## 🔧 開發 (dev/)

| 文件 | 說明 |
|------|------|
| [dev/INTEGRATION_TESTS.md](./dev/INTEGRATION_TESTS.md) | 後端整合測試執行說明 |
| [dev/ci-local.md](./dev/ci-local.md) | 本地 CI 操作 |
| [dev/e2e/README.md](./dev/e2e/README.md) | E2E 測試完整指南 |
| [dev/e2e/FLOW.md](./dev/e2e/FLOW.md) | E2E CI 與本機流程 |

---

## ⚙️ 維運 (ops/)

| 文件 | 說明 |
|------|------|
| [ops/OPERATIONS.md](./ops/OPERATIONS.md) | 維運手冊（服務擁有者、on-call、故障排除） |
| [ops/COMPOSE.md](./ops/COMPOSE.md) | Docker Compose 總覽 |
| [ops/ENV_AND_DB.md](./ops/ENV_AND_DB.md) | 環境變數與資料庫 |
| [ops/TUNNEL.md](./ops/TUNNEL.md) | 隧道設定（Cloudflare） |
| [ops/SSL_SETUP.md](./ops/SSL_SETUP.md) | SSL 設定 |
| [ops/WINDOWS_BUILD.md](./ops/WINDOWS_BUILD.md) | Windows 本地建置說明 |
| [ops/infrastructure.md](./ops/infrastructure.md) | 基礎設施總覽 |
| [ops/IMAGE_DIGESTS.md](./ops/IMAGE_DIGESTS.md) | Docker image digest 釘選表 |
| [ops/SMTP_CREDENTIALS.md](./ops/SMTP_CREDENTIALS.md) | SMTP 憑證管理（Plan B） |
| [ops/VPS_CHEATSHEET.md](./ops/VPS_CHEATSHEET.md) | VPS 維運指令速查 |
| [ops/OBSERVABILITY_PLAN.md](./ops/OBSERVABILITY_PLAN.md) | 可觀測性計畫（Prometheus、Loki、告警） |
| **ERP 進銷存操作** | |
| [ops/PRODUCT_IMPORT_ROLLBACK.md](./ops/PRODUCT_IMPORT_ROLLBACK.md) | 產品匯入後還原 |
| [ops/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md](./ops/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md) | LLM 產生 SKU 準則 |
| [ops/STOCKLIST_TO_PRODUCT_IMPORT.md](./ops/STOCKLIST_TO_PRODUCT_IMPORT.md) | 庫存清表 → 產品匯入範本 |
| [ops/SKU_CATEGORY_MANAGEMENT.md](./ops/SKU_CATEGORY_MANAGEMENT.md) | 品類管理說明 |
| [ops/CSV_TO_MD_PRODUCT_LIST_RULES.md](./ops/CSV_TO_MD_PRODUCT_LIST_RULES.md) | CSV 轉 .md 產品清單規則 |

---

## 📖 Runbooks (runbooks/)

| 文件 | 說明 |
|------|------|
| [runbooks/DR_RUNBOOK.md](./runbooks/DR_RUNBOOK.md) | 災難復原 Runbook |
| [runbooks/DR_DRILL_CHECKLIST.md](./runbooks/DR_DRILL_CHECKLIST.md) | 災難復原演練檢查表 |
| [runbooks/PRODUCTS_SOFT_ROLLBACK_BY_TIME.md](./runbooks/PRODUCTS_SOFT_ROLLBACK_BY_TIME.md) | 產品資料按時間點軟回滾 |

---

## 🗄️ 資料庫 (db/)

| 文件 | 說明 |
|------|------|
| [db/DB_ROLLBACK.md](./db/DB_ROLLBACK.md) | Migration 回滾步驟與 SQL |
| [db/ZERO_DOWNTIME_MIGRATIONS.md](./db/ZERO_DOWNTIME_MIGRATIONS.md) | 零停機遷移策略 |
| [db/RESTORE_OLD_DUMP.md](./db/RESTORE_OLD_DUMP.md) | 舊 dump 還原 |
| [db/FULL_DB_EXPORT_PLAN.md](./db/FULL_DB_EXPORT_PLAN.md) | 完整 DB 匯出計畫 |
| [db/DATA_IMPORT_RULES.md](./db/DATA_IMPORT_RULES.md) | 資料匯入規則 |
| [db/DATA_IMPORT_TROUBLESHOOTING.md](./db/DATA_IMPORT_TROUBLESHOOTING.md) | 資料匯入故障排除 |

---

## 🔒 安全合規 (security/)

| 文件 | 說明 |
|------|------|
| [security/THREAT_MODEL.md](./security/THREAT_MODEL.md) | 系統威脅模型（STRIDE / 資產清單） |
| [security/security.md](./security/security.md) | 安全性紀錄（CVE 評估與處置） |
| [security/CREDENTIAL_ROTATION.md](./security/CREDENTIAL_ROTATION.md) | 憑證輪換 SOP |
| [security/SESSION_LOGOUT_MANAGEMENT.md](./security/SESSION_LOGOUT_MANAGEMENT.md) | Session 與登出管理 |
| [security/ELECTRONIC_SIGNATURE_COMPLIANCE.md](./security/ELECTRONIC_SIGNATURE_COMPLIANCE.md) | 電子簽章合規（21 CFR Part 11）|
| [security/GLP_VALIDATION.md](./security/GLP_VALIDATION.md) | GLP 驗證文件 |
| [security/DATA_RETENTION_POLICY.md](./security/DATA_RETENTION_POLICY.md) | 資料保留政策 |
| [security/SOC2_READINESS.md](./security/SOC2_READINESS.md) | SOC2 準備度 |
| [security/SLA.md](./security/SLA.md) | 服務水準協議 |
| [security/SECURITY_COMPLETED.md](./security/SECURITY_COMPLETED.md) | 安全工作完成清單 |

---

## 🚢 部署 (deploy/)

| 文件 | 說明 |
|------|------|
| [deploy/DEPLOYMENT.md](./deploy/DEPLOYMENT.md) | 部署與維運手冊（主文件） |
| [deploy/DEPLOYMENT_NAS_DS923.md](./deploy/DEPLOYMENT_NAS_DS923.md) | NAS DS923+ 專用部署手冊 |
| [deploy/nas-migration/migration-sop.md](./deploy/nas-migration/migration-sop.md) | NAS 遷移 SOP |

---

## 📦 歸檔 (archive/)

> **只讀。** 歷史快照、完成的設計計畫、已結案的 R 輪次報告。透過 git 仍可完整追溯。

| 子目錄 | 內容 |
|--------|------|
| [archive/code-reviews/](./archive/code-reviews/) | 各期 code review + 安全審計報告 + walkthroughs |
| [archive/improvement-plans/](./archive/improvement-plans/) | R1–R7 + CI / Dependabot 改善計畫 |
| [archive/assessments/](./archive/assessments/) | R6 財務、Dependabot、效能基準報告 |
| [archive/compliance-reports/](./archive/compliance-reports/) | GLP/ISO 合規分析交付文件 |
| [archive/heartbeat/](./archive/heartbeat/) | 每日自動 heartbeat 報告 |
| [archive/feature-designs/](./archive/feature-designs/) | 已完成功能的設計文件（E2E TDD、重構計畫、AIReview...）|
| [archive/legacy/](./archive/legacy/) | Laravel 舊分支、Spec v2（原 Profiling_Spec/archive）|
