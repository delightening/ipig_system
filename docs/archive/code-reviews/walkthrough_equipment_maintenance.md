# 設備維護管理系統 — Walkthrough

> 日期：2026-03-23

## 概述

擴充現有設備管理模組，新增以下功能：

1. **設備狀態**：啟用/停用/維修/報廢 四種狀態，含狀態變更 audit trail
2. **設備廠商關聯**：多對多，關聯 partners 表，含聯絡人/電話/email
3. **校正/確效/查核**：三種措施類型（calibration/validation/inspection），每台設備二擇一 + 可選查核
4. **維修/保養紀錄**：同一張表用類型區分，維修中自動變更設備狀態，完修自動恢復
5. **報廢紀錄**：含申請→核准簽核流程，核准後自動變更設備狀態為報廢
6. **年度維護校正計畫表**：設備×月份矩陣，依週期隨機排程

## 資料表設計

### 新增 Enum 類型
- `equipment_status`: active / inactive / under_repair / decommissioned
- `calibration_type`: calibration / validation / inspection
- `calibration_cycle`: monthly / quarterly / semi_annual / annual
- `maintenance_type`: repair / maintenance
- `maintenance_status`: pending / in_progress / completed / unrepairable
- `disposal_status`: pending / approved / rejected

### 修改既有表
- `equipment` 表新增欄位：`status`, `calibration_type`, `calibration_cycle`, `inspection_cycle`
- `equipment_calibrations` 表新增欄位：`calibration_type`, `partner_id`, `report_number`, `inspector`, `equipment_serial_number`

### 新增資料表
| 資料表 | 用途 |
|--------|------|
| `equipment_suppliers` | 設備-廠商多對多關聯 |
| `equipment_status_logs` | 狀態變更 audit trail |
| `equipment_maintenance_records` | 維修/保養紀錄 |
| `equipment_disposals` | 報廢紀錄（含簽核） |
| `equipment_annual_plans` | 年度維護校正計畫（12 月份矩陣） |

### 新增權限
- `equipment.disposal.approve` — 核准設備報廢
- `equipment.maintenance.manage` — 管理維修保養
- `equipment.plan.manage` — 管理年度計畫

### 新增通知類型
- `equipment_overdue` — 校正/確效逾期
- `equipment_unrepairable` — 設備無法維修
- `equipment_disposal` — 報廢申請

## 後端變更

### Models (`backend/src/models/equipment.rs`)
- 新增 6 個 Enum 類型
- 擴充 `Equipment` struct 新增 status/calibration_type/calibration_cycle/inspection_cycle
- 擴充 `EquipmentCalibration` / `CalibrationWithEquipment` 新增 calibration_type/partner_id/report_number/inspector/equipment_serial_number
- 新增 struct: `EquipmentSupplier`, `EquipmentStatusLog`, `EquipmentMaintenanceRecord`, `EquipmentDisposal`, `EquipmentAnnualPlan` 及對應的 request/query 型別

### Repository (`backend/src/repositories/equipment.rs`)
- 新增 `find_maintenance_record_by_id`, `find_disposal_by_id`

### Service (`backend/src/services/equipment.rs`)
- 擴充既有 CRUD 支援新欄位
- 新增：設備廠商管理、狀態日誌查詢、維修保養 CRUD、報廢申請/核准、年度計畫產生
- 狀態自動變更邏輯（維修中→完修恢復、報廢核准→報廢狀態）
- 年度計畫隨機排程演算法

### Handler (`backend/src/handlers/equipment.rs`)
- 新增 12 個 API endpoint

### Routes (`backend/src/routes/hr.rs`)
- 新增設備廠商、狀態日誌、維修保養、報廢、年度計畫路由

## 前端變更

### Types (`frontend/src/pages/admin/types.ts`)
- 新增所有 Enum 類型及其中文標籤對應
- 新增 interface: `EquipmentSupplierWithPartner`, `MaintenanceRecordWithDetails`, `DisposalWithDetails`, `AnnualPlanWithEquipment`
- 擴充 `Equipment`, `CalibrationWithEquipment`, `EquipmentForm`, `CalibrationForm`

### 設備管理頁面
- **EquipmentTabContent**: 新增「廠商」「校正/確效到期」「查核到期」欄位，廠商點擊彈出 Dialog 顯示詳細資訊
- **EquipmentFormDialog**: 新增校正/確效類型、週期、查核週期選擇器
- **CalibrationTabContent**: 新增「序號」「類型」「報告/人員」欄位，類型以色彩 Badge 區分
- **CalibrationFormDialog**: 新增類型選擇、報告編號、查核人員欄位
- **EquipmentStatsCards**: 改為 4 欄統計（啟用數、維修中、紀錄數、逾期待處理）

## 設計決策

1. **is_active 保留**：為向後相容保留 `is_active` 欄位，同時新增 `status` enum；migration 中自動同步既有資料
2. **equipment_serial_number denormalized**：在 calibrations 表中冗餘儲存設備序號，避免每次查詢都 JOIN
3. **年度計畫月份用 12 個 boolean 欄位**：比 JSONB 或陣列更易查詢和索引
4. **隨機排程**：使用 `rand::thread_rng` 在非 async 區塊中預先計算所有月份，避免跨 await 問題
