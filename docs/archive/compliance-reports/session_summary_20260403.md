# Session Summary — 2026-04-03

## 目標
對 iPig ERP/QAU 系統進行 GLP、ISO 17025、ISO 9001 法規合規分析，並實作所有 P0+P1 缺口修復。

---

## 完成事項（按時序）

### 1. 合規差距分析報告
- 產出 `docs/COMPLIANCE_GAP_ANALYSIS.md`
- GLP (OECD) / ISO 17025:2017 / ISO 9001:2015 三大法規逐條分析
- 識別 3 P0 + 9 P1 + 16 P2 + 6 P3 共 34 項差距

### 2. P0+P1 合規改進實作

**Migration 016** — 13 張新表 + 3 個 ALTER：
- reference_standards（校正追溯鏈）
- controlled_documents / document_revisions / document_acknowledgments（文件控制 DMS）
- management_reviews（管理審查）
- risk_register（風險管理）
- change_requests（變更控制）
- environment_monitoring_points / environment_readings（環境監控）
- competency_assessments / role_training_requirements（能力評鑑）
- study_final_reports（最終報告）
- formulation_records（配製紀錄）
- ALTER: equipment_calibrations（追溯欄位）、qa_sop_documents（審核欄位）、products（GLP 欄位）

**Backend (Rust/Axum)**：
- `models/glp_compliance.rs` — 全部 DTO/Entity/Request
- `repositories/glp_compliance.rs` — 全部 CRUD SQL
- `services/glp_compliance.rs` — 業務邏輯 + 自動編號 + 超標偵測
- `handlers/glp_compliance.rs` — 42 個 handler + 權限檢查
- `routes/admin.rs` — 30+ route 定義
- `constants.rs` — STUDY_DIRECTOR / TEST_FACILITY_MANAGEMENT 角色
- `permissions.rs` — 21 新權限 + 6 角色映射

**Frontend (React/TypeScript)**：
- `lib/api/glpCompliance.ts` — API 函數 + 型別
- 8 個新頁面：DocumentControlPage、ManagementReviewPage、RiskRegisterPage、ChangeControlPage、EnvironmentMonitoringPage、CompetencyAssessmentPage、StudyFinalReportPage、FormulationRecordsPage
- `App.tsx` — lazy import + 8 route 註冊

### 3. 架構審計
- 以架構工程師角色驗證所有 P0+P1 項目的前後端覆蓋完整性
- 發現 FormulationRecordsPage 前端缺失 → 立即修補

### 4. AUP + HR 模組合規分析
- 產出 `docs/AUP_HR_COMPLIANCE_ANALYSIS.md`
- AUP：8 項 GLP 缺口（SD 文字欄位、無 QAU 閘門、無鎖定等）
- HR：6 項勞基法缺口（加班乘數、特休計算、工時驗證等）

### 5. HR 勞基法合規實作
- **特休假自動計算**（勞基法 §38）：`calculate_annual_leave_days()` + `seniority_months()` + `auto_calculate` + `batch` + 10 tests
- **加班上限規則**（勞基法 §32）：`check_monthly_overtime_limit()` + 46hr/54hr 常數
- **日/週工時驗證**（勞基法 §30）：`validate_work_hours()` + 8hr/day 40hr/week
- **平日加班分段**（勞基法 §24）：`split_weekday_overtime()` tier1(≤2hr) + tier2(>2hr) + 7 tests
- 5 個新 API 端點

### 6. 交付文件
- 產出 `docs/COMPLIANCE_DELIVERY_SUMMARY.md` — 全模組完整交付摘要

---

## 驗證結果

| 項目 | 結果 |
|------|------|
| `cargo check` | ✓ 通過 |
| `cargo clippy` | ✓ 零警告 |
| `cargo test --lib` | ✓ 276 passed (新增 17) |
| `tsc --noEmit` | ✓ 零錯誤 |
| `npm run build` | ✓ 14.91s |
| `docker compose up -d --build` | ✓ 全部容器 Healthy |

## 數量統計

| 指標 | 數量 |
|------|------|
| 新增資料表 | 13 |
| ALTER 擴充表 | 3 |
| 新增權限 | 21 |
| 新增角色 | 2 |
| 新增 API 端點 | 37 (GLP) + 5 (HR) = 42 |
| 新增前端頁面 | 8 |
| 新增 unit tests | 17 |
| 新增/修改 Backend 檔案 | 17 |
| 新增/修改 Frontend 檔案 | 11 |
| 產出文件 | 4 (.md) |
