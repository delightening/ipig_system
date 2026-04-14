# GLP / ISO 17025 / ISO 9001 合規改進交付摘要

> Compliance Improvement Delivery Summary

| 項目 | 內容 |
|------|------|
| **文件編號** | iPig-DEL-001 |
| **日期** | 2026-04-03 |
| **涵蓋範圍** | P0 (3 項) + P1 (9 項) 合規缺口修復 + HR 勞基法合規 |
| **新增檔案** | 12 Backend + 8 Frontend + 1 Migration + 2 分析報告 |
| **新增資料表** | 13 張 |
| **新增權限** | 21 項 |
| **新增角色** | 2 個 |
| **新增 API 端點** | 37 個 |
| **新增前端頁面** | 8 個 |
| **新增測試** | 17 個 unit tests |
| **驗證結果** | cargo check ✓ / clippy 零警告 / 276 tests passed / tsc ✓ / npm build ✓ |

---

## 目錄

1. [P0 關鍵缺口修復](#1-p0-關鍵缺口修復)
2. [P1 重大缺口修復 — ERP 模組](#2-p1-重大缺口修復--erp-模組)
3. [P1 重大缺口修復 — QAU 模組](#3-p1-重大缺口修復--qau-模組)
4. [P1 重大缺口修復 — AUP 模組](#4-p1-重大缺口修復--aup-模組)
5. [HR 模組勞基法合規](#5-hr-模組勞基法合規)
6. [權限與角色總覽](#6-權限與角色總覽)
7. [新增檔案清單](#7-新增檔案清單)
8. [API 端點清單](#8-api-端點清單)
9. [分析報告清單](#9-分析報告清單)

---

## 1. P0 關鍵缺口修復

### P0-1 / P0-2：GLP 角色定義

| 項目 | 交付內容 |
|------|---------|
| **Study Director** | 角色 `STUDY_DIRECTOR`：PI 核心權限 + `glp.study_report.sign` + `study.report.manage` + `formulation.record.manage` |
| **Test Facility Management** | 角色 `TEST_FACILITY_MANAGEMENT`：全域唯讀 + 管理系統完整存取 (DMS/Risk/Change/Env/Competency/Management Review) |
| **角色常數** | `constants.rs:83-84` — `ROLE_STUDY_DIRECTOR`, `ROLE_TEST_FACILITY_MANAGEMENT` |
| **權限映射** | `permissions.rs:471-508` — SD 22 項、TFM 30 項權限 |
| **Migration** | `016_glp_compliance.sql:10-13` — 角色 INSERT + 權限 INSERT + role_permissions 映射 |
| **QAU 擴權** | 新增 8 項合規模組唯讀權限 |
| **ADMIN_STAFF 擴權** | 新增 13 項管理系統完整存取權限 |
| **EXPERIMENT_STAFF 擴權** | 新增 env.monitoring + formulation.record 權限 |
| **EQUIPMENT_MAINTENANCE 擴權** | 新增 change.request + env.monitoring 權限 |

### P0-3：校正計量追溯鏈 (ISO 17025 §6.5)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **參考標準器表** | `reference_standards`：name、serial_number、standard_type (primary/secondary/working)、traceable_to、national_standard_number、calibration_lab、calibration_lab_accreditation (TAF)、measurement_uncertainty、status | `migrations/016:144-169` |
| **校正擴充欄位** | `ALTER TABLE equipment_calibrations` 新增：reference_standard_id (FK)、calibration_lab_name、calibration_lab_accreditation、traceability_statement、reading_before、reading_after | `migrations/016:171-178` |
| **CRUD API** | `GET/POST /admin/reference-standards`, `GET/PUT /admin/reference-standards/:id` | `routes/admin.rs:193-202` |
| **Backend 全棧** | models + repo + service + handler | `handlers/glp_compliance.rs:23-65` |
| **Frontend API** | `listReferenceStandards`, `getReferenceStandard`, `createReferenceStandard`, `updateReferenceStandard` | `lib/api/glpCompliance.ts:28-44` |
| **權限** | 復用 `equipment.view` / `equipment.manage` |

---

## 2. P1 重大缺口修復 — ERP 模組

### P1-1：文件控制系統 DMS (ISO 17025 §8.3 / ISO 9001 §7.5)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **受控文件主表** | `controlled_documents`：doc_number (自動編號)、6 種類型 (quality_manual/sop/form/external/policy/report)、5 種狀態 (draft→under_review→approved→active→obsolete)、effective_date、review_due_date、retention_years、owner_id、approved_by | `migrations/016:182-207` |
| **版本修訂表** | `document_revisions`：version、change_summary、revised_by、reviewed_by、approved_by | `migrations/016:209-225` |
| **人員簽收表** | `document_acknowledgments`：user_id、version_acknowledged、UNIQUE 約束 | `migrations/016:227-235` |
| **自動編號** | 依類型年度遞增：`QM-2026-0001`、`SOP-2026-0001`、`FM-2026-0001` 等 | `services/glp_compliance.rs:101-115` |
| **完整工作流 API** | CRUD + `POST /:id/approve` + `POST /:id/revisions` + `POST /:id/acknowledge` | `routes/admin.rs:204-225` |
| **業務規則** | 僅 draft/under_review 可核准；建立 revision 時自動遞增 current_version | `services/glp_compliance.rs:67-99` |
| **Frontend 頁面** | `DocumentControlPage.tsx` — 類型/狀態篩選、核准按鈕、簽收按鈕、建立 dialog | `pages/admin/DocumentControlPage.tsx` |
| **路由** | `/admin/document-control` + `RequirePermission("dms.document.view")` | `App.tsx:364-368` |
| **權限** | `dms.document.view` / `dms.document.manage` / `dms.document.approve` |

### P1-6：環境監控 (GLP §3 / ISO 17025 §6.3)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **監控點表** | `environment_monitoring_points`：name、location_type (animal_room/lab/storage/clean_room)、building_id (FK)、zone_id (FK)、parameters (JSONB: [{name, unit, min, max}])、monitoring_interval (continuous/hourly/daily)、is_active | `migrations/016:332-348` |
| **環境讀數表** | `environment_readings`：monitoring_point_id (FK CASCADE)、reading_time、readings (JSONB: {temperature: 23.5, humidity: 55})、is_out_of_range、out_of_range_params (JSONB)、recorded_by、source (manual/sensor/import) | `migrations/016:350-365` |
| **自動超標偵測** | `check_out_of_range()` — 比對 JSONB parameters min/max vs readings 值，回傳超標參數名稱列表 | `services/glp_compliance.rs:252-271` |
| **API (6 端點)** | 監控點 `GET/POST/PUT /admin/env-monitoring/points/*`、讀數 `GET/POST /admin/env-monitoring/readings` | `routes/admin.rs:264-278` |
| **Frontend 頁面** | `EnvironmentMonitoringPage.tsx` — 雙區段 (監控點列表 + 讀數紀錄)、超標紅色 Badge | `pages/admin/EnvironmentMonitoringPage.tsx` |
| **路由** | `/admin/environment-monitoring` | `App.tsx:384-388` |
| **權限** | `env.monitoring.view` / `env.monitoring.manage` |

### P1-9：試驗物質管理強化 (GLP §4.3)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **Products GLP 欄位** | `ALTER TABLE products` 新增：glp_characterization (TEXT)、stability_data (JSONB)、storage_conditions、cas_number、is_test_article (BOOL)、is_control_article (BOOL) | `migrations/016:438-444` |
| **配製紀錄表** | `formulation_records`：product_id (FK)、protocol_id (FK)、formulation_date、batch_number、concentration、volume、prepared_by (FK)、verified_by (FK)、verified_at、expiry_date | `migrations/016:446-462` |
| **API** | `GET/POST /admin/formulation-records` | `routes/admin.rs:310-314` |
| **Frontend 頁面** | `FormulationRecordsPage.tsx` — 完整 CRUD、批號/濃度/體積輸入 | `pages/admin/FormulationRecordsPage.tsx` |
| **路由** | `/admin/formulation-records` | `App.tsx:398-402` |
| **權限** | `formulation.record.view` / `formulation.record.manage` |

---

## 3. P1 重大缺口修復 — QAU 模組

### P1-2：管理審查 (ISO 17025 §8.9 / ISO 9001 §9.3)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **審查紀錄表** | `management_reviews`：review_number (自動 MR-YYYY-NNNN)、4 種狀態 (planned/in_progress/completed/closed)、agenda、attendees (JSONB: [{user_id, name, role}])、minutes、decisions (JSONB: [{decision, responsible, due_date}])、action_items (JSONB: [{item, assignee_id, due_date, status}])、chaired_by | `migrations/016:237-259` |
| **API** | `GET/POST /admin/management-reviews`, `GET/PUT /admin/management-reviews/:id` | `routes/admin.rs:227-236` |
| **Frontend 頁面** | `ManagementReviewPage.tsx` — 狀態篩選、建立 dialog (title/review_date/agenda) | `pages/admin/ManagementReviewPage.tsx` |
| **路由** | `/admin/management-reviews` | `App.tsx:369-373` |
| **權限** | `glp.management_review.view` / `glp.management_review.manage` |

### P1-3：風險管理 (ISO 17025 §8.5 / ISO 9001 §6.1)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **風險登記簿** | `risk_register`：risk_number (自動 RISK-YYYY-NNNN)、5 種狀態 (identified/assessed/mitigating/accepted/closed)、4 種類別 (technical/operational/compliance/safety)、severity (1-5) × likelihood (1-5) = risk_score (GENERATED STORED)、detectability、mitigation_plan、residual_risk_score、owner_id (FK)、related_nc_id | `migrations/016:261-288` |
| **API** | `GET/POST /admin/risks`, `GET/PUT /admin/risks/:id` | `routes/admin.rs:238-247` |
| **Frontend 頁面** | `RiskRegisterPage.tsx` — 風險分數色碼 (1-6 綠 / 7-14 黃 / 15-25 紅)、類別/狀態篩選 | `pages/admin/RiskRegisterPage.tsx` |
| **路由** | `/admin/risk-register` | `App.tsx:374-378` |
| **權限** | `risk.register.view` / `risk.register.manage` |

### P1-4：變更控制 (ISO 9001 §6.3)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **變更申請表** | `change_requests`：change_number (自動 CR-YYYY-NNNN)、7 種狀態 (draft→submitted→under_review→approved→implemented→verified→rejected)、6 種類型 (equipment/method/personnel/facility/system/process)、description、justification、impact_assessment、多角色審核鏈 (requested_by→reviewed_by→approved_by→verified_by) | `migrations/016:290-328` |
| **業務規則** | 僅 submitted/under_review 狀態可核准 | `services/glp_compliance.rs:197-202` |
| **API (5 端點)** | CRUD + `POST /:id/approve` | `routes/admin.rs:249-262` |
| **Frontend 頁面** | `ChangeControlPage.tsx` — 類型/狀態篩選、核准按鈕 (需 change.request.approve) | `pages/admin/ChangeControlPage.tsx` |
| **路由** | `/admin/change-control` | `App.tsx:379-383` |
| **權限** | `change.request.view` / `change.request.manage` / `change.request.approve` |

### P1-5：SOP 審核簽署強化 (GLP §6)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **表結構擴充** | `ALTER TABLE qa_sop_documents` 新增 6 欄位：reviewed_by (FK users)、reviewed_at、approved_by (FK users)、approved_at、review_due_date、revision_history (JSONB: [{version, change_summary, date}]) | `migrations/016:320-325` |

### P1-7：能力評鑑框架 (ISO 17025 §6.2 / ISO 9001 §7.2)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **能力評鑑表** | `competency_assessments`：3 種評鑑類型 (initial/periodic/requalification)、3 種結果 (competent/not_yet_competent/requires_supervision)、4 種方法 (observation/written_test/practical_test/peer_review)、score (NUMERIC 5,2)、valid_until、assessor_id (FK) | `migrations/016:368-387` |
| **訓練需求矩陣** | `role_training_requirements`：role_code→training_topic 對應、is_mandatory、recurrence_months、UNIQUE (role_code, training_topic) | `migrations/016:389-400` |
| **API (5 端點)** | 評鑑 `GET/POST/PUT /admin/competency-assessments/*`、需求 `GET/POST/DELETE /admin/training-requirements/*` | `routes/admin.rs:280-297` |
| **Frontend 頁面** | `CompetencyAssessmentPage.tsx` — 結果色碼 (competent=綠 / not_yet=紅 / supervision=黃) | `pages/admin/CompetencyAssessmentPage.tsx` |
| **路由** | `/admin/competency-assessments` | `App.tsx:389-393` |
| **權限** | `competency.assessment.view` / `competency.assessment.manage` |

---

## 4. P1 重大缺口修復 — AUP 模組

### P1-8：最終報告 (GLP §9)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **最終報告表** | `study_final_reports`：report_number (自動 SFR-YYYY-NNNN)、protocol_id (FK protocols)、4 種狀態 (draft/under_review/approved/signed)、結構化內容 (summary/methods/results/conclusions/deviations)、SD 簽署 (signed_by/signed_at/signature_id FK electronic_signatures)、QAU 聲明 (qau_statement/qau_signed_by/qau_signed_at) | `migrations/016:402-435` |
| **API** | `GET/POST /admin/study-reports`, `GET/PUT /admin/study-reports/:id` | `routes/admin.rs:299-308` |
| **Frontend 頁面** | `StudyFinalReportPage.tsx` — 狀態篩選、protocol_id 連結 | `pages/admin/StudyFinalReportPage.tsx` |
| **路由** | `/admin/study-reports` | `App.tsx:393-397` |
| **權限** | `study.report.view` / `study.report.manage` / `glp.study_report.sign` |

---

## 5. HR 模組勞基法合規

### 5.1 特休假自動計算 (勞基法 §38)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **年資計算函數** | `seniority_months(hire_date, as_of)` — 精確至月，考慮日期前後差異 | `services/hr/balance.rs:50-55` |
| **天數計算函數** | `calculate_annual_leave_days(seniority_months)` — 完整勞基法 §38 對照表 | `services/hr/balance.rs:25-47` |
| **單一員工自動核算** | `auto_calculate_annual_leave(pool, user_id, year, creator_id)` — 查詢 hire_date→計算年資→推算天數→建立額度 | `services/hr/balance.rs:159-207` |
| **批次全員核算** | `batch_auto_calculate_annual_leave(pool, year, creator_id)` — 遍歷所有 is_active + hire_date 員工 | `services/hr/balance.rs:209-226` |
| **API** | `POST /hr/balances/annual-auto-calc`、`POST /hr/balances/annual-batch-calc` | `routes/hr.rs:66-71` |
| **測試** | 10 個 unit tests 涵蓋所有年資區間 (0-30 年) + 30 天上限 + 年資月數計算 | `services/hr/balance.rs` tests |

**§38 對照表實作：**

| 年資 | 天數 | 測試覆蓋 |
|------|------|---------|
| < 6 個月 | 0 天 | ✓ |
| 6 個月 ~ 1 年 | 3 天 | ✓ |
| 1 年 ~ 2 年 | 7 天 | ✓ |
| 2 年 ~ 3 年 | 10 天 | ✓ |
| 3 年 ~ 5 年 | 14 天 | ✓ |
| 5 年 ~ 10 年 | 15 天 | ✓ |
| 10 年以上 | 每年 +1 天，上限 30 天 | ✓ |

### 5.2 加班上限規則 (勞基法 §32)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **法定常數** | `MONTHLY_OVERTIME_LIMIT = 46.0`、`MONTHLY_OVERTIME_LIMIT_EXTENDED = 54.0`、`QUARTERLY_OVERTIME_LIMIT = 138.0` | `services/hr/overtime.rs` |
| **上限驗證函數** | `check_monthly_overtime_limit(pool, user_id, date, hours)` — 查詢當月非 rejected/cancelled 加班累計 + 本次申請合計是否超限 | `services/hr/overtime.rs` |
| **回傳結構** | `OvertimeLimitCheck { monthly_total, requested_hours, projected_total, exceeds_standard_limit, exceeds_extended_limit, warnings[] }` | `services/hr/overtime.rs` |
| **API** | `GET /hr/overtime/limit-check?user_id=&overtime_date=&hours=` | `routes/hr.rs:36` |

### 5.3 日/週工時驗證 (勞基法 §30)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **法定常數** | `DAILY_REGULAR_HOURS = 8.0`、`WEEKLY_REGULAR_HOURS = 40.0` | `services/hr/overtime.rs` |
| **驗證函數** | `validate_work_hours(pool, user_id, work_date)` — 查詢當日工時 + 本週 (Mon-Sun) 累計工時 | `services/hr/overtime.rs` |
| **回傳結構** | `WorkHoursValidation { daily_hours, weekly_hours, exceeds_daily_limit, exceeds_weekly_limit, suggested_overtime_hours, warnings[] }` | `services/hr/overtime.rs` |
| **API** | `GET /hr/work-hours/validate?user_id=&work_date=` | `routes/hr.rs:38` |

### 5.4 平日加班分段 (勞基法 §24)

| 項目 | 交付內容 | 位置 |
|------|---------|------|
| **分段計算函數** | `split_weekday_overtime(hours)` — 前 2hr 為 tier1、超過 2hr 為 tier2，四捨五入至 0.5hr | `services/hr/overtime.rs` |
| **回傳結構** | `WeekdayOvertimeTiers { tier1_hours, tier2_hours, total_hours }` | `services/hr/overtime.rs` |
| **API** | `GET /hr/overtime/weekday-tiers?hours=` | `routes/hr.rs:37` |
| **測試** | 7 個 unit tests：< 2hr、= 2hr、> 2hr、4hr、小數四捨五入、0hr | `services/hr/overtime.rs` tests |

---

## 6. 權限與角色總覽

### 新增角色 (2 個)

| 角色代碼 | 名稱 | 權限數 |
|----------|------|--------|
| `STUDY_DIRECTOR` | 研究主持人 (GLP Study Director) | 22 |
| `TEST_FACILITY_MANAGEMENT` | 試驗機構管理階層 (GLP TFM) | 30 |

### 新增權限 (21 項)

| 模組 | 權限碼 | 說明 |
|------|--------|------|
| **glp** | `glp.study_director.designate` | 指定 Study Director |
| **glp** | `glp.study_report.sign` | 簽署最終報告 |
| **glp** | `glp.compliance.overview` | GLP 遵循總覽 |
| **glp** | `glp.management_review.view` | 查看管理審查 |
| **glp** | `glp.management_review.manage` | 管理管理審查 |
| **dms** | `dms.document.view` | 查看受控文件 |
| **dms** | `dms.document.manage` | 管理受控文件 |
| **dms** | `dms.document.approve` | 核准受控文件 |
| **risk** | `risk.register.view` | 查看風險登記簿 |
| **risk** | `risk.register.manage` | 管理風險登記簿 |
| **change** | `change.request.view` | 查看變更申請 |
| **change** | `change.request.manage` | 管理變更申請 |
| **change** | `change.request.approve` | 核准變更申請 |
| **env** | `env.monitoring.view` | 查看環境監控 |
| **env** | `env.monitoring.manage` | 管理環境監控 |
| **competency** | `competency.assessment.view` | 查看能力評鑑 |
| **competency** | `competency.assessment.manage` | 管理能力評鑑 |
| **study** | `study.report.view` | 查看最終報告 |
| **study** | `study.report.manage` | 管理最終報告 |
| **formulation** | `formulation.record.view` | 查看配製紀錄 |
| **formulation** | `formulation.record.manage` | 管理配製紀錄 |

### 既有角色擴權

| 角色 | 新增權限數 | 內容 |
|------|-----------|------|
| QAU | +8 | 合規模組唯讀 (dms/risk/change/env/competency/study/formulation/glp.compliance) |
| ADMIN_STAFF | +13 | 管理系統完整存取 (dms/risk/change/env/competency/management_review) |
| EXPERIMENT_STAFF | +5 | env.monitoring + formulation.record + competency.assessment.view |
| EQUIPMENT_MAINTENANCE | +3 | change.request + env.monitoring.view |

---

## 7. 新增檔案清單

### Migration

| 檔案 | 說明 |
|------|------|
| `backend/migrations/016_glp_compliance.sql` | 13 張新表 + 3 個 ALTER + 角色/權限/映射 seed |

### Backend (Rust)

| 檔案 | 說明 | 行數 |
|------|------|------|
| `backend/src/models/glp_compliance.rs` | 所有新模組 DTO/Entity/Request/Query | ~550 |
| `backend/src/repositories/glp_compliance.rs` | 所有新模組 CRUD SQL 查詢 | ~650 |
| `backend/src/services/glp_compliance.rs` | 所有新模組業務邏輯 + 編號產生 + 超標偵測 | ~330 |
| `backend/src/handlers/glp_compliance.rs` | 所有新模組 API handler + 權限檢查 | ~430 |

### Backend 修改檔案

| 檔案 | 修改內容 |
|------|---------|
| `backend/src/constants.rs` | +2 角色常數 |
| `backend/src/startup/permissions.rs` | +21 權限定義 + 2 角色權限映射 |
| `backend/src/models/mod.rs` | +1 模組註冊 |
| `backend/src/repositories/mod.rs` | +1 模組註冊 |
| `backend/src/services/mod.rs` | +1 模組註冊 + 4 型別 re-export |
| `backend/src/services/hr/mod.rs` | overtime/balance 改 pub mod |
| `backend/src/services/hr/balance.rs` | +特休假自動計算 (calculate_annual_leave_days + seniority_months + auto_calculate + batch) + 10 tests |
| `backend/src/services/hr/overtime.rs` | +加班上限/工時驗證/分段計算 (常數 + 3 函數 + 3 struct) + 7 tests |
| `backend/src/handlers/mod.rs` | +1 模組註冊 |
| `backend/src/handlers/hr/balance.rs` | +2 handler (auto_calc + batch) |
| `backend/src/handlers/hr/overtime.rs` | +3 handler (limit-check + validate + weekday-tiers) |
| `backend/src/routes/admin.rs` | +30 route 定義 (GLP 合規模組) |
| `backend/src/routes/hr.rs` | +3 route 定義 (勞基法 API) |

### Frontend (TypeScript/React)

| 檔案 | 說明 |
|------|------|
| `frontend/src/lib/api/glpCompliance.ts` | 所有新模組 API 函數 + TypeScript 型別 (~530 行) |
| `frontend/src/pages/admin/DocumentControlPage.tsx` | 文件控制頁面 |
| `frontend/src/pages/admin/ManagementReviewPage.tsx` | 管理審查頁面 |
| `frontend/src/pages/admin/RiskRegisterPage.tsx` | 風險登記簿頁面 |
| `frontend/src/pages/admin/ChangeControlPage.tsx` | 變更控制頁面 |
| `frontend/src/pages/admin/EnvironmentMonitoringPage.tsx` | 環境監控頁面 |
| `frontend/src/pages/admin/CompetencyAssessmentPage.tsx` | 能力評鑑頁面 |
| `frontend/src/pages/admin/StudyFinalReportPage.tsx` | 最終報告頁面 |
| `frontend/src/pages/admin/FormulationRecordsPage.tsx` | 配製紀錄頁面 |

### Frontend 修改檔案

| 檔案 | 修改內容 |
|------|---------|
| `frontend/src/lib/api/index.ts` | +1 模組 re-export |
| `frontend/src/App.tsx` | +8 lazy import + 8 route 定義 |

---

## 8. API 端點清單

### GLP 合規模組 (30 端點)

| Method | Path | 說明 | 權限 |
|--------|------|------|------|
| GET | `/admin/reference-standards` | 列出參考標準器 | equipment.view |
| POST | `/admin/reference-standards` | 建立參考標準器 | equipment.manage |
| GET | `/admin/reference-standards/:id` | 取得參考標準器 | equipment.view |
| PUT | `/admin/reference-standards/:id` | 更新參考標準器 | equipment.manage |
| GET | `/admin/documents` | 列出受控文件 | dms.document.view |
| POST | `/admin/documents` | 建立受控文件 | dms.document.manage |
| GET | `/admin/documents/:id` | 取得文件 + 修訂歷史 | dms.document.view |
| PUT | `/admin/documents/:id` | 更新受控文件 | dms.document.manage |
| POST | `/admin/documents/:id/approve` | 核准文件 | dms.document.approve |
| POST | `/admin/documents/:id/revisions` | 建立版本修訂 | dms.document.manage |
| POST | `/admin/documents/:id/acknowledge` | 簽收文件 | dms.document.view |
| GET | `/admin/management-reviews` | 列出管理審查 | glp.management_review.view |
| POST | `/admin/management-reviews` | 建立管理審查 | glp.management_review.manage |
| GET | `/admin/management-reviews/:id` | 取得管理審查 | glp.management_review.view |
| PUT | `/admin/management-reviews/:id` | 更新管理審查 | glp.management_review.manage |
| GET | `/admin/risks` | 列出風險 | risk.register.view |
| POST | `/admin/risks` | 建立風險 | risk.register.manage |
| GET | `/admin/risks/:id` | 取得風險 | risk.register.view |
| PUT | `/admin/risks/:id` | 更新風險 | risk.register.manage |
| GET | `/admin/change-requests` | 列出變更申請 | change.request.view |
| POST | `/admin/change-requests` | 建立變更申請 | change.request.manage |
| GET | `/admin/change-requests/:id` | 取得變更申請 | change.request.view |
| PUT | `/admin/change-requests/:id` | 更新變更申請 | change.request.manage |
| POST | `/admin/change-requests/:id/approve` | 核准變更申請 | change.request.approve |
| GET | `/admin/env-monitoring/points` | 列出監控點 | env.monitoring.view |
| POST | `/admin/env-monitoring/points` | 建立監控點 | env.monitoring.manage |
| GET | `/admin/env-monitoring/points/:id` | 取得監控點 | env.monitoring.view |
| PUT | `/admin/env-monitoring/points/:id` | 更新監控點 | env.monitoring.manage |
| GET | `/admin/env-monitoring/readings` | 列出環境讀數 | env.monitoring.view |
| POST | `/admin/env-monitoring/readings` | 登錄環境讀數 | env.monitoring.manage |
| GET | `/admin/competency-assessments` | 列出能力評鑑 | competency.assessment.view |
| POST | `/admin/competency-assessments` | 建立能力評鑑 | competency.assessment.manage |
| PUT | `/admin/competency-assessments/:id` | 更新能力評鑑 | competency.assessment.manage |
| GET | `/admin/training-requirements` | 列出訓練需求 | competency.assessment.view |
| POST | `/admin/training-requirements` | 建立訓練需求 | competency.assessment.manage |
| DELETE | `/admin/training-requirements/:id` | 刪除訓練需求 | competency.assessment.manage |
| GET | `/admin/study-reports` | 列出最終報告 | study.report.view |
| POST | `/admin/study-reports` | 建立最終報告 | study.report.manage |
| GET | `/admin/study-reports/:id` | 取得最終報告 | study.report.view |
| PUT | `/admin/study-reports/:id` | 更新最終報告 | study.report.manage |
| GET | `/admin/formulation-records` | 列出配製紀錄 | formulation.record.view |
| POST | `/admin/formulation-records` | 建立配製紀錄 | formulation.record.manage |

### HR 勞基法合規 (5 端點)

| Method | Path | 說明 | 權限 |
|--------|------|------|------|
| POST | `/hr/balances/annual-auto-calc` | 單一員工特休假自動計算 | hr.balance.manage |
| POST | `/hr/balances/annual-batch-calc` | 批次全員特休假計算 | hr.balance.manage |
| GET | `/hr/overtime/limit-check` | 加班上限驗證 | (登入即可) |
| GET | `/hr/overtime/weekday-tiers` | 平日加班分段計算 | (登入即可) |
| GET | `/hr/work-hours/validate` | 日/週工時驗證 | (登入即可) |

---

## 9. 分析報告清單

| 檔案 | 說明 |
|------|------|
| `docs/COMPLIANCE_GAP_ANALYSIS.md` | GLP / ISO 17025 / ISO 9001 三大法規逐條差距分析 (P0-P3 共 28 項) |
| `docs/AUP_HR_COMPLIANCE_ANALYSIS.md` | AUP 模組 GLP 合規 + HR 模組台灣勞基法合規分析 (13 項差距) |
| `docs/COMPLIANCE_DELIVERY_SUMMARY.md` | 本文件：完整交付摘要 |

---

## 10. 安全審計改進項目追蹤

本章節追蹤安全審計 (2026-04-14 report) 後續改進工作的優先級、責任人與進度。

### Security Remediation Items

| 項目編號 | 描述 | 優先級 | 預期交付 | 狀態 | 相關審計 |
|---------|------|--------|---------|------|---------|
| **SEC-REM-01** | 強化帳號刪除流程的撤銷機制（JWT 黑名單 + Session 終止） | **P0** | 2026-04-28 | Backlog | SEC-AUDIT-017 |
| **SEC-REM-02** | Permission check 命名規範化與 CI 掃描規則 | **P1** | 2026-04-28 | Backlog | SEC-AUDIT-020 |
| **SEC-REM-03** | 編譯時 access check 可行性評估（3 大方案對比） | **P1** | 2026-05-12 | Backlog | SEC-AUDIT-020 |
| **SEC-REM-04** | Permission 檢查覆蓋率改進（301/580 → >95%） | **P2** | 2026-05-26 | Backlog | SEC-AUDIT-020 |
| **SEC-REM-05** | Test skeleton 品質標準實施（#[ignore] + failing assertion） | **P2** | 2026-05-12 | Backlog | Test Artifacts |

### Implementation Workflow

1. **Issue Creation** — 每個 SEC-REM-XX 應建立 GitHub Issue，含詳細描述與驗收標準
2. **Assignment** — Tech Lead / PM 在 backlog planning 中指派 owner
3. **PR Review** — 所有改進應透過 PR 進行代碼審查，參考 `DESIGN.md` 的 Permission 命名規範
4. **Verification** — 使用 `dev/COMPILE_TIME_ACCESS_CHECK_EVALUATION.md` 報告驗收編譯時檢查方案
5. **Progress Update** — 每週 standup / sprint review 中更新狀態

### Expected Outcomes

- ✅ JWT 殘留風險窗口從「不清楚」→「明確定義 (5 min vs 6 hour)」
- ✅ Permission 命名規範文件化於 `DESIGN.md` § 13
- ✅ CI 自動掃描規則新增，檢測缺失的 permission check
- ✅ 編譯時驗證方案評估報告，為下階段架構決策輸入
- ✅ Test skeleton 品質標準應用於新測試用例，避免假陰性
