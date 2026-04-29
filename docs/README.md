# Documentation Hub

> ipig_system 文件總索引。所有文件按主題與角色分類；本檔只負責**導航**，不重述內容。
>
> **最後更新：** 2026-04-28

---

## 依角色快速入口

### 新人 / 第一次看到本專案
1. [../README.md](../README.md) — 系統定位、tech stack、Quick Start
2. [spec/architecture/ARCHITECTURE.md](spec/architecture/ARCHITECTURE.md) — 部署圖 + 資料流 + 請求生命週期
3. [PROGRESS.md](PROGRESS.md) — 各模組完成度（先看「總體進度概覽」）
4. [user/QUICK_START.md](user/QUICK_START.md) — 5 分鐘把系統跑起來

### GLP / FDA / QAU 稽核員
1. [glp/traceability-matrix.md](glp/traceability-matrix.md) — **唯一入口**。21 CFR Part 11 + GLP 雙向追溯
2. [glp/amendment-sop.md](glp/amendment-sop.md) — Amendment 流程 SOP
3. [glp/record-lock-rationale.md](glp/record-lock-rationale.md) — 簽章後鎖定 5 表的法規依據
4. [glp/training-records-sop.md](glp/training-records-sop.md) — §11.10(i) 教育訓練紀錄
5. [security/ELECTRONIC_SIGNATURE_COMPLIANCE.md](security/ELECTRONIC_SIGNATURE_COMPLIANCE.md) — 電子簽章合規說明
6. [security/GLP_VALIDATION.md](security/GLP_VALIDATION.md) — GLP 驗證文件
7. [security/DATA_RETENTION_POLICY.md](security/DATA_RETENTION_POLICY.md) — 資料保留政策
8. [runbooks/audit-chain-broken-runbook.md](runbooks/audit-chain-broken-runbook.md) — Audit chain 斷鏈處理
9. [runbooks/dr-drill-records.md](runbooks/dr-drill-records.md) — DR 演練紀錄
10. [audit/system-review-2026-04-25.md](audit/system-review-2026-04-25.md) — 上一次內部審查

### Backend 工程師
1. [spec/architecture/ARCHITECTURE.md](spec/architecture/ARCHITECTURE.md)
2. [R26_compliance_requirements.md](R26_compliance_requirements.md) — Service-driven audit pattern 形成背景
3. [spec/architecture/03_MODULES_AND_BOUNDARIES.md](spec/architecture/03_MODULES_AND_BOUNDARIES.md) — 模組拆分與 API 前綴
4. [spec/architecture/04_DATABASE_SCHEMA.md](spec/architecture/04_DATABASE_SCHEMA.md) — 資料表 + ENUM
5. [spec/architecture/05_API_SPECIFICATION.md](spec/architecture/05_API_SPECIFICATION.md) — 完整端點
6. [spec/architecture/06_PERMISSIONS_RBAC.md](spec/architecture/06_PERMISSIONS_RBAC.md) — RBAC 矩陣
7. [spec/guides/AUDIT_LOGGING.md](spec/guides/AUDIT_LOGGING.md) — Audit 規範
8. [dev/INTEGRATION_TESTS.md](dev/INTEGRATION_TESTS.md) — 整合測試（需 Postgres）
9. [codeReviewFindings.md](codeReviewFindings.md) — 三軸 code review 結果

### Frontend 工程師
1. [../DESIGN.md](../DESIGN.md) — 設計系統（**改 UI 前必讀**）
2. [spec/guides/UI_UX_GUIDELINES.md](spec/guides/UI_UX_GUIDELINES.md)
3. [spec/architecture/05_API_SPECIFICATION.md](spec/architecture/05_API_SPECIFICATION.md) — 對應後端端點
4. [dev/e2e/README.md](dev/e2e/README.md) — Playwright E2E
5. [user/USER_GUIDE.md](user/USER_GUIDE.md) — 終端使用者操作流程（理解使用者預期）

### 維運 / SRE / On-call
1. [ops/OPERATIONS.md](ops/OPERATIONS.md) — 維運手冊（服務擁有者、on-call）
2. [ops/COMPOSE.md](ops/COMPOSE.md) — Docker Compose 各檔用途
3. [deploy/DEPLOYMENT.md](deploy/DEPLOYMENT.md) — 部署主文件
4. [deploy/DEPLOYMENT_NAS_DS923.md](deploy/DEPLOYMENT_NAS_DS923.md) — NAS 專用
5. [runbooks/DR_RUNBOOK.md](runbooks/DR_RUNBOOK.md) — 災難復原
6. [runbooks/audit-chain-broken-runbook.md](runbooks/audit-chain-broken-runbook.md)
7. [ops/OBSERVABILITY_PLAN.md](ops/OBSERVABILITY_PLAN.md) — Prometheus / Grafana / 告警
8. [security/CREDENTIAL_ROTATION.md](security/CREDENTIAL_ROTATION.md) — 憑證輪換 SOP

### QA
1. [dev/INTEGRATION_TESTS.md](dev/INTEGRATION_TESTS.md)
2. [dev/e2e/README.md](dev/e2e/README.md) + [dev/e2e/FLOW.md](dev/e2e/FLOW.md)
3. [dev/ci-local.md](dev/ci-local.md)
4. [glp/traceability-matrix.md](glp/traceability-matrix.md) — 確認新測試對應條款

---

## 任務追蹤

| 文件 | 用途 |
|---|---|
| [TODO.md](TODO.md) | 任務追蹤（R 系列）。**勿刪 / 勿改結構**（CLAUDE.md 定義） |
| [PROGRESS.md](PROGRESS.md) | 系統完成度 + §9 變更日誌（**唯一變更日誌**）。**勿刪 / 勿改結構** |

---

## 主題目錄

```
docs/
├── README.md              ← 本檔
├── TODO.md                ← 任務追蹤
├── PROGRESS.md            ← 進度 + §9 變更日誌
├── R26_compliance_requirements.md
├── codeReviewFindings.md
│
├── glp/                   ← GLP / 21 CFR Part 11 SOP + traceability
├── audit/                 ← 系統審查報告
├── runbooks/              ← 事故 / DR / 演練
│
├── spec/                  ← 技術規格 v7.x
│   ├── architecture/      ← 架構、API、RBAC、ERD
│   ├── modules/           ← 各子系統規格
│   └── guides/            ← 命名、稽核、UI/UX
│
├── dev/                   ← 開發者（整合測試、CI、E2E）
├── ops/                   ← 維運（操作、Compose、SKU）
├── deploy/                ← 部署（DEPLOYMENT、NAS 遷移）
├── db/                    ← Migration、回滾、匯入
├── security/              ← 威脅模型、合規、憑證輪換
├── user/                  ← 終端使用者（QUICK_START、USER_GUIDE）
└── archive/               ← 歷史快照（只讀）
```

---

## glp/ — GLP / 21 CFR Part 11 SOP

| 文件 | 一行說明 | 最後更新 |
|---|---|---|
| [glp/traceability-matrix.md](glp/traceability-matrix.md) | 條款 ↔ migration / service / test 雙向追溯（稽核入口） | R30-I |
| [glp/amendment-sop.md](glp/amendment-sop.md) | Amendment 流程 SOP（含電子簽章流程） | R30 |
| [glp/record-lock-rationale.md](glp/record-lock-rationale.md) | 簽章後鎖定 5 表的法規依據 + 程式對應 | R30 |
| [glp/training-records-sop.md](glp/training-records-sop.md) | §11.10(i) 教育訓練紀錄 SOP | R30 |

---

## audit/ — 內部審查報告

| 文件 | 一行說明 |
|---|---|
| [audit/system-review-2026-04-25.md](audit/system-review-2026-04-25.md) | 2026-04-25 全系統審查（含合規、安全、效能、文件） |

---

## runbooks/

| 文件 | 一行說明 |
|---|---|
| [runbooks/DR_RUNBOOK.md](runbooks/DR_RUNBOOK.md) | 災難復原流程（DB 還原、服務重啟、驗證） |
| [runbooks/DR_DRILL_CHECKLIST.md](runbooks/DR_DRILL_CHECKLIST.md) | DR 演練檢查表 |
| [runbooks/dr-drill-records.md](runbooks/dr-drill-records.md) | 已執行 DR 演練紀錄 |
| [runbooks/audit-chain-broken-runbook.md](runbooks/audit-chain-broken-runbook.md) | Audit HMAC chain 斷鏈偵測與處理 |
| [runbooks/PRODUCTS_SOFT_ROLLBACK_BY_TIME.md](runbooks/PRODUCTS_SOFT_ROLLBACK_BY_TIME.md) | 產品資料按時間點軟回滾 |

---

## spec/ — 技術規格 v7.x

### architecture/
| 文件 | 說明 |
|---|---|
| [spec/architecture/ARCHITECTURE.md](spec/architecture/ARCHITECTURE.md) | 架構圖 + 資料流（含 mermaid） |
| [spec/architecture/SYSTEM_RELATIONSHIPS.md](spec/architecture/SYSTEM_RELATIONSHIPS.md) | 模組關聯與法規遵循 |
| [spec/architecture/01_ARCHITECTURE_OVERVIEW.md](spec/architecture/01_ARCHITECTURE_OVERVIEW.md) | 系統目的、技術堆疊 |
| [spec/architecture/02_CORE_DOMAIN_MODEL.md](spec/architecture/02_CORE_DOMAIN_MODEL.md) | 實體、關係、商業規則 |
| [spec/architecture/03_MODULES_AND_BOUNDARIES.md](spec/architecture/03_MODULES_AND_BOUNDARIES.md) | 模組拆分、API 前綴 |
| [spec/architecture/04_DATABASE_SCHEMA.md](spec/architecture/04_DATABASE_SCHEMA.md) | 資料表 + ENUM + migration 統計 |
| [spec/architecture/05_API_SPECIFICATION.md](spec/architecture/05_API_SPECIFICATION.md) | 完整端點 + 認證 + 回應格式 |
| [spec/architecture/06_PERMISSIONS_RBAC.md](spec/architecture/06_PERMISSIONS_RBAC.md) | 角色 / 權限 / 存取矩陣 |
| [spec/architecture/07_SECURITY_AUDIT.md](spec/architecture/07_SECURITY_AUDIT.md) | 安全 middleware、2FA、WAF、GLP |
| [spec/architecture/09_EXTENSIBILITY.md](spec/architecture/09_EXTENSIBILITY.md) | 已完成 + 未來規劃 |
| [spec/architecture/database_erd.md](spec/architecture/database_erd.md) | 完整 ER 圖（mermaid） |

### modules/
| 文件 | 說明 |
|---|---|
| [spec/modules/AUP_SYSTEM.md](spec/modules/AUP_SYSTEM.md) | AUP 提交與審查 |
| [spec/modules/AUP.md](spec/modules/AUP.md) | AUP 表單欄位（詳細） |
| [spec/modules/ERP_SYSTEM.md](spec/modules/ERP_SYSTEM.md) | ERP 進銷存 |
| [spec/modules/ANIMAL_MANAGEMENT.md](spec/modules/ANIMAL_MANAGEMENT.md) | 動物管理 |
| [spec/modules/HR_SYSTEM.md](spec/modules/HR_SYSTEM.md) | 人事管理 |
| [spec/modules/08_ATTENDANCE_MODULE.md](spec/modules/08_ATTENDANCE_MODULE.md) | 出勤（打卡、請假、加班） |
| [spec/modules/NOTIFICATION_SYSTEM.md](spec/modules/NOTIFICATION_SYSTEM.md) | 通知 |
| [spec/modules/MCP_REVIEW_SERVER.md](spec/modules/MCP_REVIEW_SERVER.md) | MCP AI 接入規格 |
| [spec/modules/AI_API_GUIDE.md](spec/modules/AI_API_GUIDE.md) | 外部 AI 資料查詢 |
| [spec/modules/EMS_DEPLOYMENT.md](spec/modules/EMS_DEPLOYMENT.md) | 環境監控硬體部署（R21） |

### guides/
| 文件 | 說明 |
|---|---|
| [spec/guides/AUDIT_LOGGING.md](spec/guides/AUDIT_LOGGING.md) | 稽核日誌規範 |
| [spec/guides/NAMING_CONVENTIONS.md](spec/guides/NAMING_CONVENTIONS.md) | 命名慣例 |
| [spec/guides/STORAGE_SETUP.md](spec/guides/STORAGE_SETUP.md) | 儲存設定 |
| [spec/guides/UI_UX_GUIDELINES.md](spec/guides/UI_UX_GUIDELINES.md) | UI/UX 準則 |

---

## dev/ — 開發者

| 文件 | 說明 |
|---|---|
| [dev/INTEGRATION_TESTS.md](dev/INTEGRATION_TESTS.md) | 後端整合測試（需 Docker Postgres） |
| [dev/ci-local.md](dev/ci-local.md) | 本地跑 CI |
| [dev/e2e/README.md](dev/e2e/README.md) | E2E 完整指南 |
| [dev/e2e/FLOW.md](dev/e2e/FLOW.md) | E2E CI 與本機流程 |

---

## ops/ — 維運

| 文件 | 說明 |
|---|---|
| [ops/OPERATIONS.md](ops/OPERATIONS.md) | 維運手冊（on-call、故障排除） |
| [ops/COMPOSE.md](ops/COMPOSE.md) | 各 Compose 檔用途 |
| [ops/ENV_AND_DB.md](ops/ENV_AND_DB.md) | 環境變數與 DB |
| [ops/TUNNEL.md](ops/TUNNEL.md) | Cloudflare Tunnel 設定 |
| [ops/SSL_SETUP.md](ops/SSL_SETUP.md) | SSL 設定 |
| [ops/WINDOWS_BUILD.md](ops/WINDOWS_BUILD.md) | Windows 本地建置 |
| [ops/infrastructure.md](ops/infrastructure.md) | 基礎設施總覽 |
| [ops/IMAGE_DIGESTS.md](ops/IMAGE_DIGESTS.md) | Docker image digest 釘選 |
| [ops/SMTP_CREDENTIALS.md](ops/SMTP_CREDENTIALS.md) | SMTP 憑證管理 |
| [ops/VPS_CHEATSHEET.md](ops/VPS_CHEATSHEET.md) | VPS 速查 |
| [ops/OBSERVABILITY_PLAN.md](ops/OBSERVABILITY_PLAN.md) | 可觀測性計畫 |
| [ops/PRODUCT_IMPORT_ROLLBACK.md](ops/PRODUCT_IMPORT_ROLLBACK.md) | 產品匯入還原 |
| [ops/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md](ops/PRODUCT_IMPORT_LLM_SKU_GUIDELINES.md) | LLM 產生 SKU 準則 |
| [ops/STOCKLIST_TO_PRODUCT_IMPORT.md](ops/STOCKLIST_TO_PRODUCT_IMPORT.md) | 庫存清表轉匯入範本 |
| [ops/SKU_CATEGORY_MANAGEMENT.md](ops/SKU_CATEGORY_MANAGEMENT.md) | 品類管理 |
| [ops/CSV_TO_MD_PRODUCT_LIST_RULES.md](ops/CSV_TO_MD_PRODUCT_LIST_RULES.md) | CSV → md 規則 |

---

## deploy/

| 文件 | 說明 |
|---|---|
| [deploy/DEPLOYMENT.md](deploy/DEPLOYMENT.md) | 部署主文件 |
| [deploy/DEPLOYMENT_NAS_DS923.md](deploy/DEPLOYMENT_NAS_DS923.md) | NAS DS923+ 專用 |
| [deploy/nas-migration/migration-sop.md](deploy/nas-migration/migration-sop.md) | NAS 遷移 SOP |

---

## db/

| 文件 | 說明 |
|---|---|
| [db/DB_ROLLBACK.md](db/DB_ROLLBACK.md) | Migration 回滾步驟 |
| [db/ZERO_DOWNTIME_MIGRATIONS.md](db/ZERO_DOWNTIME_MIGRATIONS.md) | 零停機遷移策略 |
| [db/RESTORE_OLD_DUMP.md](db/RESTORE_OLD_DUMP.md) | 舊 dump 還原 |
| [db/FULL_DB_EXPORT_PLAN.md](db/FULL_DB_EXPORT_PLAN.md) | 完整 DB 匯出計畫 |
| [db/DATA_IMPORT_RULES.md](db/DATA_IMPORT_RULES.md) | 資料匯入規則 |
| [db/DATA_IMPORT_TROUBLESHOOTING.md](db/DATA_IMPORT_TROUBLESHOOTING.md) | 匯入故障排除 |

---

## security/

| 文件 | 說明 |
|---|---|
| [security/THREAT_MODEL.md](security/THREAT_MODEL.md) | 系統威脅模型（STRIDE） |
| [security/security.md](security/security.md) | 安全紀錄（CVE 評估與處置） |
| [security/CREDENTIAL_ROTATION.md](security/CREDENTIAL_ROTATION.md) | 憑證輪換 SOP |
| [security/SESSION_LOGOUT_MANAGEMENT.md](security/SESSION_LOGOUT_MANAGEMENT.md) | Session / 登出管理 |
| [security/ELECTRONIC_SIGNATURE_COMPLIANCE.md](security/ELECTRONIC_SIGNATURE_COMPLIANCE.md) | 電子簽章合規（21 CFR Part 11） |
| [security/GLP_VALIDATION.md](security/GLP_VALIDATION.md) | GLP 驗證文件 |
| [security/DATA_RETENTION_POLICY.md](security/DATA_RETENTION_POLICY.md) | 資料保留政策 |
| [security/SOC2_READINESS.md](security/SOC2_READINESS.md) | SOC2 準備度 |
| [security/SLA.md](security/SLA.md) | 服務水準協議 |
| [security/SECURITY_COMPLETED.md](security/SECURITY_COMPLETED.md) | 安全工作完成清單 |

---

## user/ — 終端使用者

| 文件 | 說明 |
|---|---|
| [user/QUICK_START.md](user/QUICK_START.md) | 快速啟動（Docker / 本地 / E2E） |
| [user/USER_GUIDE.md](user/USER_GUIDE.md) | 操作手冊（9 章節） |

---

## archive/ — 歷史快照

> **只讀。** 已封存的歷次 R 輪報告、舊版設計、舊 spec。git 歷程仍可追溯。

| 子目錄 | 內容 |
|---|---|
| [archive/code-reviews/](archive/code-reviews/) | 各期 code review + walkthroughs |
| [archive/improvement-plans/](archive/improvement-plans/) | R1–R7 + CI / Dependabot 改善計畫 |
| [archive/assessments/](archive/assessments/) | R6 財務、Dependabot、效能基準 |
| [archive/compliance-reports/](archive/compliance-reports/) | 早期 GLP / ISO 合規分析 |
| [archive/heartbeat/](archive/heartbeat/) | 自動 heartbeat 報告 |
| [archive/feature-designs/](archive/feature-designs/) | 已完成功能的設計文件 |
| [archive/legacy/](archive/legacy/) | Laravel 舊分支、Spec v2 |
