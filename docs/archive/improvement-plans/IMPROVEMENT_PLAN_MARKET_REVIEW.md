# iPig 系統 — 市場基準檢視與改進計劃

> **建立日期：** 2026-03-01  
> **檢視基準：** 企業 ERP 系統、GLP 合規軟體、生產環境就緒檢查清單  
> **專案狀態：** 功能完整、上線準備中（R1–R4 改善已完成）

---

## 一、檢視摘要

本文件依據**市場上企業系統的基礎要求**，對豬博士 iPig 系統進行全面檢視，並產出改進計劃。檢視基準涵蓋：

| 基準類別 | 參考來源 |
|----------|----------|
| 企業 ERP 系統 | Top 20 ERP Requirement Checklist 2025、ERP Selection Criteria |
| GLP 合規軟體 | OECD Computerized Systems Guidance、FDA 21 CFR 58 |
| 生產環境就緒 | Production Readiness Checklist、Enterprise Application Security |
| 既有改善計畫 | IMPROVEMENT_PLAN R1–R4、TODO.md、PROGRESS.md |

---

## 二、市場基準對照表

### 2.1 企業 ERP 核心功能

| 市場要求 | iPig 現況 | 符合度 |
|----------|-----------|--------|
| 財務管理（AP/AR/GL/預算） | 進銷存模組有單據流程、成本方法，無完整會計模組 | 🟡 部分 |
| HR 與考勤管理 | 打卡、請假、加班、年假/補休、行事曆同步 | ✅ 完整 |
| 庫存與供應鏈管理 | 產品/SKU、倉庫/儲位、單據流程、庫存流水、低庫存警示 | ✅ 完整 |
| 工作流程自動化 | 計畫書審查流程、單據核准流程、通知路由 | ✅ 完整 |
| 即時報表與分析 | 血檢分析、庫存報表、稽核日誌、儀表板 | ✅ 完整 |

### 2.2 技術與基礎設施

| 市場要求 | iPig 現況 | 符合度 |
|----------|-----------|--------|
| 資料一致性與主資料管理 | PostgreSQL、遷移腳本、Optimistic Locking | ✅ 完整 |
| 與既有系統整合（API） | REST API、Swagger ≥90%、OpenAPI | ✅ 完整 |
| 客製化與設定彈性 | 系統設定 API、通知路由、角色權限 | ✅ 完整 |
| 行動裝置存取 | 響應式 UI、mobile sidebar、無 PWA/原生 App | 🟡 部分 |
| 部署選項（雲端/混合/地端） | Docker Compose、prod overlay、可自架 | ✅ 完整 |

### 2.3 安全與合規

| 市場要求 | iPig 現況 | 符合度 |
|----------|-----------|--------|
| 資料安全標準（GDPR/SOC2/PCI） | 隱私政策、Cookie 同意、稽核日誌；無 SOC2 認證 | 🟡 部分 |
| 資料隱私與法規遵循 | 資料保留政策、21 CFR Part 11 電子簽章審查 | ✅ 完整 |
| 稽核軌跡與可追溯性 | audit_logs、HMAC 驗證鏈、登入追蹤 | ✅ 完整 |
| 認證與存取控制 | JWT + Refresh + TOTP 2FA、RBAC、敏感操作二級認證 | ✅ 完整 |

### 2.4 GLP 合規（實驗動物管理）

| GLP 要求 | iPig 現況 | 符合度 |
|----------|-----------|--------|
| 即時準確紀錄觀察 | 觀察/手術/血檢/體重/疫苗紀錄、版本控管 | ✅ 完整 |
| 稽核軌跡（誰/何時/為何） | audit_logs、電子簽章、版本歷程 | ✅ 完整 |
| 研究計畫書管理 | AUP 計畫書、審查流程、變更申請、核准紀錄 | ✅ 完整 |
| 人員訓練紀錄 | 無專用訓練模組 | 🔴 缺口 |
| 設備校準紀錄 | 無專用模組 | 🔴 缺口 |
| 品質保證（QAU）功能 | 審查流程、稽核日誌；無獨立 QAU 檢視 | 🟡 部分 |

### 2.5 生產環境就緒

| 市場要求 | iPig 現況 | 符合度 |
|----------|-----------|--------|
| 憑證安全儲存與輪換 | Docker Secrets、read_secret()；無 90 天輪換機制 | 🟡 部分 |
| 稽核日誌保留與匯出 | 10 年保留政策；無 CSV/JSON 匯出 API | 🟡 部分 |
| 結構化日誌與搜尋 | JSON log、Loki overlay；可強化查詢介面 | 🟡 部分 |
| 監控與可觀測性 | Prometheus、Grafana、Alertmanager、/health、/metrics | ✅ 完整 |
| 備份與災難復原 | GPG 加密備份、pg_restore 驗證、DR Runbook | ✅ 完整 |
| 服務擁有者與 on-call | 無文件化 | 🔴 缺口 |

---

## 三、改進計劃

### 3.1 優先級說明

| 優先級 | 說明 |
|--------|------|
| **P0** | 上線前必要（阻擋項） |
| **P1** | 上線前強烈建議（合規/品質） |
| **P2** | 中期改善（市場競爭力） |
| **P3** | 長期演進（差異化） |

---

### P0 — 上線前必要

*目前無 P0 項目。核心功能、安全、監控、備份均已就緒。*

---

### P1 — 上線前強烈建議

| # | 項目 | 說明 | 工時估計 |
|---|------|------|----------|
| P1-M0 | **稽核日誌匯出 API** | 新增 `GET /api/admin/audit-logs/export?format=csv|json&from=&to=`，供合規稽核與外部系統整合。市場標準要求稽核日誌可匯出。 | 0.5 天 |
| P1-M1 | **API 版本路徑** | 引入 `/api/v1/` 前綴，為未來 API 變更保留向後相容空間。可採漸進式：新路由用 v1，舊路由維持並標 deprecated。 | 1 天 |
| P1-M2 | **GDPR 資料主體權利** | 若服務歐盟使用者，實作「存取權」「刪除權」「可攜權」：`GET /api/me/export`（個人資料匯出）、`DELETE /api/me/account`（帳號刪除請求）。隱私政策補充對應說明。 | 1–2 天 |
| P1-M3 | **服務擁有者與維運文件** | 新增 `docs/OPERATIONS.md`：服務擁有者、on-call 輪值、升級聯絡人、故障排除流程。 | 0.5 天 |
| P1-M4 | **憑證輪換文件** | ~~撰寫~~ **已完成**：`docs/security-compliance/CREDENTIAL_ROTATION.md` 已存在，含 JWT/DB/SMTP 輪換流程與驗證。 | — |
| P1-M5 | **Dependabot Phase 2 收尾** | 完成 `DEPENDABOT_MIGRATION_PLAN.md` 中 Phase 2 剩餘項目（react-ecosystem、dev-dependencies），確保依賴與安全更新同步。 | 0.5–1 天 |

---

### P2 — 中期改善（市場競爭力）

| # | 項目 | 說明 | 工時估計 |
|---|------|------|----------|
| P2-M1 | **PWA 離線支援** | ~~Service Worker、離線快取打卡/觀察草稿、掃碼入庫相機優化~~ **暫不需評估**：無離線功能需求，維持現有響應式 Web 即可。 | — |
| P2-M2 | **人員訓練紀錄模組** | GLP 合規缺口：新增「人員訓練紀錄」表與 CRUD，可連結使用者與訓練項目（課程名稱、完成日期、有效期限）。 | 1–2 天 |
| P2-M3 | **設備校準紀錄（可選）** | 若實驗室有設備校準需求，新增「設備」與「校準紀錄」模組；否則以文件說明「本系統不涵蓋設備校準，由外部流程管理」。 | 1–2 天 |
| P2-M4 | **稽核日誌查詢 UI** | 管理後台新增稽核日誌進階查詢：依時間範圍、使用者、動作類型、資源類型篩選，並支援匯出按鈕。 | 1 天 |
| P2-M5 | **SOC 2 合規準備文件** | 撰寫 `docs/security-compliance/SOC2_READINESS.md`，對照 Trust Services Criteria，列出已滿足項目與待補強項目，供未來認證參考。 | 0.5 天 |

---

### P3 — 長期演進（差異化）

| # | 項目 | 說明 | 工時估計 |
|---|------|------|----------|
| P3-M1 | **財務模組擴充** | 若業務需要完整會計：AP/AR、總帳、預算、財務報表。可採模組化設計，與既有 ERP 單據串接。 | 5+ 天 |
| P3-M2 | **QAU 獨立檢視** | GLP 品質保證：新增「QAU 檢視」角色與專用儀表板，可檢視研究狀態、審查進度、稽核摘要，不具編輯權限。 | 2 天 |
| P3-M3 | **原生 App / React Native** | ~~若需離線打卡、現場掃碼等強行動需求~~ **暫不需評估**：無離線打卡、現場掃碼等需求，維持現有響應式 Web 即可。 | — |
| P3-M4 | **多租戶架構** | ~~若規劃 SaaS 多客戶部署~~ **暫不需評估**：無多客戶需求，維持單租戶部署即可。 | — |

---

## 四、既有優勢（無需改動）

以下項目已達市場水準，維持現狀即可：

| 面向 | 現況 |
|------|------|
| **認證與授權** | JWT + Refresh + TOTP 2FA、RBAC、敏感操作二級認證 |
| **安全防護** | CSRF、Rate Limiting、DOMPurify、Argon2、WAF overlay |
| **容器與部署** | 三層網路隔離、Docker Secrets、非 root 容器 |
| **監控與告警** | Prometheus、Grafana、Alertmanager、DB Pool 指標 |
| **備份與還原** | GPG 加密、pg_restore 驗證、還原腳本 |
| **測試覆蓋** | 119 unit tests、25+ API 整合測試、34 E2E tests |
| **文件** | USER_GUIDE、DEPLOYMENT、ARCHITECTURE、電子簽章合規、資料保留政策 |
| **GLP 核心** | 電子簽章、稽核軌跡、計畫書管理、動物醫療紀錄 |

---

## 五、執行建議

1. **上線前**：優先完成 P0-M1、P0-M2、P1-M3，確保合規與維運基礎。
2. **上線後 3 個月**：依業務需求評估 P1-M1（API 版本）、P1-M2（GDPR）、P2-M1（PWA）。
3. **GLP 稽核前**：若稽核單位要求，補齊 P2-M2（人員訓練紀錄）、P2-M3（設備校準或說明文件）。
4. **長期**：P3 項目依產品路線圖與資源配置排程。

---

## 六、附錄：市場參考來源

- [Top 20 ERP Requirement Checklist 2025](https://taloflow.ai/guides/requirements/erp)
- [ERP Selection Criteria](https://www.captivix.com/blog/erp-selection-criteria-checklist)
- [OECD GLP Computerized Systems](https://www.oecd.org/en/publications/reports/2016/04/application-of-glp-principles-to-computerised-systems_7d05366e.html)
- [Production Readiness Checklist](https://docs.redpanda.com/current/deploy/redpanda/manual/production/production-readiness/)
- [Automating Production Readiness 2025](https://cortex.io/post/automating-production-readiness-guide-2025)

---

*文件產出於 2026-03-01，依據市場基準與專案現況撰寫。*
