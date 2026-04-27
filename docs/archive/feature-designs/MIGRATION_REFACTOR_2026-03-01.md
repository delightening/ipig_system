# Migration 重構說明

> **日期：** 2026-03-01  
> **變更：** 原 12 個 migrations 重構為 10 個邏輯區塊

---

## 新結構（10 部分）

| # | 檔案 | 內容 |
|---|------|------|
| 1 | `001_types.sql` | 所有自訂 ENUM 類型（ERP、AUP、Animal、HR、Amendment 等） |
| 2 | `002_users_auth.sql` | users、roles、permissions、role_permissions、user_roles、refresh_tokens、password_reset_tokens |
| 3 | `003_notifications_attachments.sql` | notifications、notification_settings、attachments、audit_logs、觸發器 |
| 4 | `004_roles_permissions.sql` | 角色/權限 seed、role_permissions 分配、user_preferences |
| 5 | `005_animal_management.sql` | 動物管理（animal_sources、animals、observations、surgeries、blood_tests 等） |
| 6 | `006_aup_system.sql` | AUP 計畫書（protocols、amendments、review、system_settings 等） |
| 7 | `007_hr_system.sql` | HR 人事（attendance、overtime、leave、calendar 等） |
| 8 | `008_audit_erp.sql` | 稽核系統（user_activity_logs、login_events、security_alerts）+ ERP 倉儲（warehouses、products、documents、stock_ledger 等） |
| 9 | `009_supplementary.sql` | 通知路由、電子簽章、治療藥物、JWT blacklist、Enum cast、Optimistic locking、2FA、效能索引 |
| 10 | `010_glp_accounting.sql` | 訓練紀錄、設備校準、QAU、會計（chart_of_accounts、journal_entries、ap_payments、ar_receipts）、training.manage_own 權限 |

---

## 已移除的舊檔案

- `001_core_schema.sql` → 拆成 001~004
- `002_animal_management.sql` → 005
- `003_aup_system.sql` → 006
- `004_hr_system.sql` → 007
- `005_audit_system.sql` → 008（前半）
- `006_erp_warehouse.sql` → 008（後半）
- `007_supplementary.sql` → 009（前半）
- `008_fixes.sql` → 009（中段）
- `009_performance.sql` → 009（後段）
- `010_glp_qau.sql` → 010
- `011_equipment_maintenance_role.sql` → 已空檔，刪除
- `012_training_manage_own.sql` → 併入 010

---

## 既有部署注意事項

若資料庫已執行過舊版 migrations（`_sqlx_migrations` 表有紀錄），**請勿直接套用新 migrations**。新結構僅適用於**全新安裝**。

既有環境如需遷移，請：
1. 備份資料庫
2. 參考 `docs/database/DB_ROLLBACK.md` 評估回滾風險
3. 或維持既有 schema，僅新增後續 migrations
