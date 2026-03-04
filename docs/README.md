# 豬博士 iPig 系統 — 文件索引

> **最後更新：** 2026-03-04  
> 本目錄為 `docs/` 總索引，依主題分類便於查找。

**閱讀建議**：第一次進入可從下方「根目錄常用文件」選一篇；依角色可選：**開發** → development/、QUICK_START；**維運** → operations/OPERATIONS.md、DEPLOYMENT、runbooks/；**規格** → Profiling_Spec/；**安全合規** → security-compliance/。

---

## 根目錄常用文件（入口）

| 文件 | 說明 |
|------|------|
| [PROGRESS.md](./PROGRESS.md) | 專案進度評估表（總體進度、上線準備度、最新變更動態） |
| [TODO.md](./TODO.md) | 待辦功能清單（P0–P5 優先級） |
| [QUICK_START.md](./QUICK_START.md) | 快速啟動（Docker / 本地開發 / E2E 測試） |
| [USER_GUIDE.md](./USER_GUIDE.md) | 使用者操作手冊 |
| [DEPLOYMENT.md](./DEPLOYMENT.md) | 部署與維運手冊 |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 架構圖與資料流 |
| [operations/OPERATIONS.md](./operations/OPERATIONS.md) | 維運手冊（服務擁有者、on-call、故障排除） |
| [walkthrough.md](./walkthrough.md) | 實作說明與 walkthrough 紀錄 |

---

## 規格與架構 (Spec)

| 路徑 | 說明 |
|------|------|
| [Profiling_Spec/](./Profiling_Spec/) | 技術規格 v7.0：架構、核心領域、模組邊界、資料庫、API、權限、安全稽核、出勤、擴展性 |
| [Profiling_Spec/guides/](./Profiling_Spec/guides/) | 稽核日誌、命名慣例、UI/UX、儲存設定 |
| [Profiling_Spec/modules/](./Profiling_Spec/modules/) | 各子系統模組說明（ERP、通知、動物、HR、AUP） |

---

## 專案管理 (Project)

| 文件 | 說明 |
|------|------|
| [project/description.md](./project/description.md) | 專案描述 |
| [project/VERSION_HISTORY.md](./project/VERSION_HISTORY.md) | 版本歷程 |
| [project/AUP.md](./project/AUP.md) | AUP 相關說明 |
| [project/walkthrough.md](./project/walkthrough.md) | 專案內 walkthrough |
| [project/解構.md](./project/解構.md) | 解構說明 |
| [project/手機端架構.md](./project/手機端架構.md) | 手機端架構 |
| [project/e2e-*.md](./project/) | E2E 相關討論與調查 |

---

## 開發與測試 (Development)

| 文件 | 說明 |
|------|------|
| [development/INTEGRATION_TESTS.md](./development/INTEGRATION_TESTS.md) | 整合測試說明 |
| [development/ci-local.md](./development/ci-local.md) | 本地 CI 操作 |
| [development/REFACTOR_PLAN_USESTATE_TO_HOOKS.md](./development/REFACTOR_PLAN_USESTATE_TO_HOOKS.md) | useState 重構計畫 |
| [development/MIGRATION_REFACTOR_2026-03-01.md](./development/MIGRATION_REFACTOR_2026-03-01.md) | Migration 重構紀錄 |
| [development/PERMISSION_AUDIT_2026-03-01.md](./development/PERMISSION_AUDIT_2026-03-01.md) | 權限稽核紀錄 |
| [development/IMPROVEMENT_PLAN_R1.md](./development/IMPROVEMENT_PLAN_R1.md) | 第一輪改善計畫 |
| [development/IMPROVEMENT_PLAN_R2.md](./development/IMPROVEMENT_PLAN_R2.md) | 第二輪改善計畫 |
| [development/IMPROVEMENT_PLAN_R3.md](./development/IMPROVEMENT_PLAN_R3.md) | 第三輪改善計畫 |
| [development/IMPROVEMENT_PLAN_MARKET_REVIEW.md](./development/IMPROVEMENT_PLAN_MARKET_REVIEW.md) | 市場檢視改善計畫 |
| [development/CI_IMPROVEMENT_PLAN.md](./development/CI_IMPROVEMENT_PLAN.md) | CI 改善計畫 |
| [development/DEPENDABOT_MIGRATION_PLAN.md](./development/DEPENDABOT_MIGRATION_PLAN.md) | Dependabot 遷移計畫 |
| [development/DATA_EXPORT_IMPORT_DESIGN.md](./development/DATA_EXPORT_IMPORT_DESIGN.md) | 資料匯出匯入設計 |
| [development/E2E TDD plan.md](./development/E2E%20TDD%20plan.md) | E2E TDD 計畫 |
| [development/P2-R4-13_ANIMAL_ANY_ELIMINATION_PLAN.md](./development/P2-R4-13_ANIMAL_ANY_ELIMINATION_PLAN.md) | 動物安樂死/淘汰計畫 |
| [e2e/](./e2e/) | E2E 測試說明（README、FLOW） |

---

## 資料庫 (Database)

| 文件 | 說明 |
|------|------|
| [database/DB_ROLLBACK.md](./database/DB_ROLLBACK.md) | Migration 回滾步驟與 SQL |
| [database/ZERO_DOWNTIME_MIGRATIONS.md](./database/ZERO_DOWNTIME_MIGRATIONS.md) | 零停機遷移策略 |
| [database/RESTORE_OLD_DUMP.md](./database/RESTORE_OLD_DUMP.md) | 舊 dump 還原 |
| [database/FULL_DB_EXPORT_PLAN.md](./database/FULL_DB_EXPORT_PLAN.md) | 完整 DB 匯出計畫 |
| [database/DATA_IMPORT_RULES.md](./database/DATA_IMPORT_RULES.md) | 資料匯入規則 |
| [database/DATA_IMPORT_TROUBLESHOOTING.md](./database/DATA_IMPORT_TROUBLESHOOTING.md) | 資料匯入故障排除 |

---

## 安全與合規 (Security & Compliance)

| 文件 | 說明 |
|------|------|
| [security-compliance/security.md](./security-compliance/security.md) | 安全性紀錄（CVE 評估與處置） |
| [security-compliance/WAF.md](./security-compliance/WAF.md) | WAF 設定與說明 |
| [security-compliance/CREDENTIAL_ROTATION.md](./security-compliance/CREDENTIAL_ROTATION.md) | 憑證輪換 |
| [security-compliance/ELECTRONIC_SIGNATURE_COMPLIANCE.md](./security-compliance/ELECTRONIC_SIGNATURE_COMPLIANCE.md) | 電子簽章合規 |
| [security-compliance/SOC2_READINESS.md](./security-compliance/SOC2_READINESS.md) | SOC2 準備度 |
| [security-compliance/GLP_VALIDATION.md](./security-compliance/GLP_VALIDATION.md) | GLP 驗證 |
| [security-compliance/DATA_RETENTION_POLICY.md](./security-compliance/DATA_RETENTION_POLICY.md) | 資料保留政策 |
| [security-compliance/SLA.md](./security-compliance/SLA.md) | 服務水準協議 |

---

## 維運與災難復原 (Runbooks)

| 文件 | 說明 |
|------|------|
| [runbooks/DR_DRILL_CHECKLIST.md](./runbooks/DR_DRILL_CHECKLIST.md) | 災難復原演練檢查表 |
| [runbooks/DR_RUNBOOK.md](./runbooks/DR_RUNBOOK.md) | 災難復原 runbook |

---

## 環境與建置 (Operations)

| 文件 | 說明 |
|------|------|
| [operations/OPERATIONS.md](./operations/OPERATIONS.md) | 維運手冊（服務擁有者、on-call、升級流程、故障排除） |
| [operations/COMPOSE.md](./operations/COMPOSE.md) | Docker Compose 總覽 |
| [operations/ENV_AND_DB.md](./operations/ENV_AND_DB.md) | 環境變數與資料庫 |
| [operations/TUNNEL.md](./operations/TUNNEL.md) | 隧道設定 |
| [operations/WINDOWS_BUILD.md](./operations/WINDOWS_BUILD.md) | Windows 建置說明 |
| [operations/SSL_SETUP.md](./operations/SSL_SETUP.md) | SSL 設定 |
| [operations/infrastructure.md](./operations/infrastructure.md) | 基礎設施說明 |

---

## 評估與報告 (Assessments)

| 文件 | 說明 |
|------|------|
| [assessments/R6-5_DEPENDABOT_PHASE25_ASSESSMENT.md](./assessments/R6-5_DEPENDABOT_PHASE25_ASSESSMENT.md) | Dependabot Phase 25 評估 |
| [assessments/R6-4_FINANCE_PHASE2_5_ASSESSMENT.md](./assessments/R6-4_FINANCE_PHASE2_5_ASSESSMENT.md) | 財務 Phase 2.5 評估 |
| [assessments/PERFORMANCE_BENCHMARK.md](./assessments/PERFORMANCE_BENCHMARK.md) | 效能基準測試 |

---

## 目錄結構摘要

```
docs/
├── README.md              ← 本索引
├── PROGRESS.md
├── TODO.md
├── QUICK_START.md
├── USER_GUIDE.md
├── DEPLOYMENT.md
├── ARCHITECTURE.md
├── walkthrough.md
├── Profiling_Spec/        # 規格與架構
├── project/               # 專案管理
├── development/           # 開發與測試
├── e2e/                   # E2E 測試說明
├── database/              # 資料庫遷移與還原
├── security-compliance/   # 安全與合規
├── runbooks/              # 維運 runbook（DR 等）
├── operations/            # 環境與建置（維運手冊、Compose、隧道、SSL）
└── assessments/           # 評估報告
```
