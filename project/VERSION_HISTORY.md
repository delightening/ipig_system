# 版本歷程紀錄

> **最後更新**：2026-01-18

---

## 文件版本

### Profiling_Spec v2.0（2026-01-18）

**完整重寫**

所有規格文件均已重新撰寫，以反映實際程式碼庫的實作情況：

| 文件 | 版本 | 狀態 |
|----------|---------|--------|
| 00_INDEX.md | 2.0 | ✅ 已更新 |
| 01_ARCHITECTURE_OVERVIEW.md | 1.0 | ✅ 新增（原始版本）|
| 02_CORE_DOMAIN_MODEL.md | 2.0 | ✅ 新增 |
| 03_MODULES_AND_BOUNDARIES.md | 2.0 | ✅ 新增 |
| 04_DATABASE_SCHEMA.md | 2.0 | ✅ 新增 |
| 05_API_SPECIFICATION.md | 2.0 | ✅ 新增 |
| 06_PERMISSIONS_RBAC.md | 2.0 | ✅ 新增 |
| 07_AUDIT_LOGGING.md | 1.0 | ✅ 新增（原始版本）|
| 08_ATTENDANCE_MODULE.md | 1.0 | ✅ 新增（原始版本）|
| 09_EXTENSIBILITY.md | 1.0 | ✅ 新增（原始版本）|
| 10_UI_UX_GUIDELINES.md | 2.0 | ✅ 新增 |
| 11_NAMING_CONVENTIONS.md | 2.0 | ✅ 新增 |
| 12_VERSION_HISTORY.md | 2.0 | ✅ 已更新 |

**主要變更**：
- 更新索引，反映正確的系統概覽與技術堆疊
- 建立完整的核心領域模型，包含所有實體與列舉型別
- 建立模組與邊界文件，詳細描述各模組分工
- 建立完整的資料庫 Schema，基於 10 個遷移腳本
- 建立完整的 API 規格，涵蓋超過 250 個端點
- 建立權限與 RBAC 文件，包含所有系統角色與權限
- 建立前端開發用 UI/UX 指南
- 建立各程式碼層級的命名慣例

---

### Profiling_Spec v1.0（2026-01-17）

**初始發布**

建立初版規格文件框架：
- 00_INDEX.md - 索引
- 01_ARCHITECTURE_OVERVIEW.md - 系統架構
- 07_AUDIT_LOGGING.md - GLP 合規稽核紀錄
- 08_ATTENDANCE_MODULE.md - 人資考勤模組
- 09_EXTENSIBILITY.md - 擴展性設計
- 12_VERSION_HISTORY.md - 版本追蹤

---

## 資料庫遷移歷程

### 2026-01-18

| 遷移腳本 | 說明 |
|-----------|-------------|
| 010_add_deleted_at_column.sql | 新增 deleted_at 欄位用於豬隻軟刪除 |

### 2026-01-17

| 遷移腳本 | 說明 |
|-----------|-------------|
| 001_aup_system.sql | 核心 Schema：使用者、角色、ERP、計畫書、豬隻、通知 |
| 002_erp_base_data.sql | SKU 類別、產品類別種子資料 |
| 003_seed_accounts.sql | 初始管理員帳號與角色 |
| 004_hr_system.sql | 考勤、加班、請假管理 |
| 005_calendar_sync.sql | Google 日曆整合 |
| 006_audit_system.sql | GLP 合規稽核紀錄（含分區表） |
| 007_seed_data.sql | 參考資料（豬隻來源、權限） |
| 008_reset_admin.sql | 管理員密碼重設工具 |
| 009_add_roles_is_active.sql | 角色表新增 is_active 旗標 |

---

## 變更紀錄格式

更新文件時，請依以下格式新增紀錄：

```markdown
### 文件名稱 vX.Y（YYYY-MM-DD）

**變更摘要**

- 新增：XYZ 章節
- 變更：ABC 更新以反映新需求
- 修正：DEF 章節錯字
- 移除：GHI 過時章節

**破壞性變更**（如有）

- API 端點 `/old/path` 更名為 `/new/path`

**遷移注意事項**（如有）

- 部署前請執行遷移腳本 `XXX.sql`
```

---

*本文件會在規格變更時自動更新。*
