# 權限稽核報告

> **稽核日期：** 2026-03-01  
> **範圍：** 全專案頁面查看權限、訓練紀錄、設備維護

---

## 1. 稽核摘要

| 模組 | 需求 | 現況 | 差異 |
|------|------|------|------|
| 訓練紀錄 | experiment_staff 管理自己的紀錄；admin_staff 審批 | EXPERIMENT_STAFF 無權限；ADMIN_STAFF 有完整權限 | 需補齊 EXPERIMENT_STAFF 權限 |
| 設備維護 | 特定人員維護 | ADMIN_STAFF 有 equipment.view / equipment.manage | 符合（特定人員 = ADMIN_STAFF） |
| 各頁面 | 皆有正確權限保護 | 已使用 RequirePermission / Route 守衛 | 符合 |

---

## 2. 各頁面權限一覽

### 2.1 Dashboard 與導向

| 路由 | 存取條件 | 說明 |
|------|----------|------|
| `/` | 依角色導向 dashboard 或 my-projects | admin / ERP 相關 / EXPERIMENT_STAFF / REVIEWER / VET / IACUC_CHAIR → dashboard |
| `/dashboard` | DashboardRoute | 同上，否則導向 /my-projects |
| `/erp` | DashboardRoute + 依 tab 過濾 | ERP 模組依 `erp.*` 權限；equipment tab 需 `equipment.view` |

### 2.2 系統管理（Admin）

| 路由 | 權限 | 說明 |
|------|------|------|
| `/admin/*` | `admin` 角色 | AdminRoute 保護 |
| `/admin/qau` | `qau.dashboard.view` | 額外 RequirePermission |
| `/admin/notification-routing` | admin | 同上 |
| `/admin/treatment-drugs` | admin | 同上 |

### 2.3 人員訓練

| 路由 | 權限 | 說明 |
|------|------|------|
| `/hr/training-records` | `admin` 或 `training.view` 或 `training.manage` | RequirePermission anyOf |

**角色權限現況（修正前）：**

| 角色 | training.view | training.manage | 行為 |
|------|---------------|-----------------|------|
| admin | ✓ (透過角色) | ✓ | 完整存取 |
| ADMIN_STAFF | ✓ | ✓ | 完整存取、可審批/管理所有人紀錄 |
| EXPERIMENT_STAFF | ✗ | ✗ | **無法存取訓練紀錄頁面** |
| 其他 | — | — | 無權限 |

**預期行為（修正後）：**

| 角色 | training.view | training.manage_own | training.manage | 行為 |
|------|---------------|---------------------|-----------------|------|
| admin | ✓ | — | ✓ | 完整存取 |
| ADMIN_STAFF | ✓ | — | ✓ | 完整存取、審批所有人紀錄 |
| EXPERIMENT_STAFF | ✓ | ✓ | ✗ | 僅查看與管理**自己的**訓練紀錄 |
| 其他 | — | — | — | 無權限 |

### 2.4 設備維護

| 路由 | 權限 | 說明 |
|------|------|------|
| `/erp?tab=equipment` | `equipment.view` 或 `equipment.manage` | ERP 頁面依權限過濾 tab |

**角色權限：**

| 角色 | equipment.view | equipment.manage | 說明 |
|------|----------------|------------------|------|
| admin | ✓ | ✓ | 完整存取 |
| ADMIN_STAFF | ✓ | ✓ | **特定人員**，負責設備維護 |
| 其他 | ✗ | ✗ | 無權限 |

設備維護由 **ADMIN_STAFF**（行政人員）負責，符合「特定人員維護」需求。

### 2.5 HR 模組

| 路由 | 權限 | 說明 |
|------|------|------|
| `/hr/attendance` | 登入即可 | DashboardRoute 下，基本 HR 權限 |
| `/hr/leaves` | 同上 | 請假：EXPERIMENT_STAFF 可申請；ADMIN_STAFF 可審核 |
| `/hr/overtime` | 同上 | 加班：同上 |
| `/hr/annual-leave` | `hr.balance.manage` 或 `admin` | 特休額度管理 |
| `/hr/calendar` | 登入即可 | 日曆設定 |

**HR 審核流程（與需求一致）：**

- EXPERIMENT_STAFF：建立自己的請假/加班紀錄
- ADMIN_STAFF：審核（approve）這些紀錄

### 2.6 其他受保護路由

| 路由 | 權限 |
|------|------|
| Dashboard 下產品/倉庫/單據/庫存/報表 | `erp.*` 或相應角色 |
| AUP 計畫書 | 依 aup.* 權限 |
| 動物管理 | 依 animal.* 權限 |
| 來源管理 | `animal.source.manage` |
| 個人設定 | 登入即可 |

---

## 3. 後端權限檢查

- **Training API**：`training.view`（僅自己）、`training.manage`（全部）、新增 `training.manage_own`（僅自己 CRUD）
- **Equipment API**：`equipment.view`、`equipment.manage`
- **HR 加班/請假**：建立者為自己；審核者需 `ADMIN_STAFF` 或 `admin`

---

## 4. 修正項目（已實作）

1. 新增權限 `training.manage_own`：管理自己的訓練紀錄
2. 為 EXPERIMENT_STAFF 指派：`training.view`、`training.manage_own`
3. 後端 Training 服務：create/update/delete 支援 `training.manage_own`（僅操作自己的紀錄）
4. 前端 TrainingRecordsPage：`canManage` 涵蓋 `training.manage_own`

---

## 5. 參考文件

- 權限規格：`docs/Profiling_Spec/06_PERMISSIONS_RBAC.md`
- 角色權限後端：`backend/src/startup/permissions.rs`
