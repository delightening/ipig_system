# iPig 系統技術規格文件

> 豬博士動物科技系統 — 完整技術文件  
> **最後更新**：2026-02-15

---

## 文件目錄

### 核心規格（01-09）

| # | 文件 | 說明 |
|---|------|------|
| 01 | [系統架構概覽](./01_ARCHITECTURE_OVERVIEW.md) | 技術堆疊、分層架構、部署 |
| 02 | [核心領域模型](./02_CORE_DOMAIN_MODEL.md) | 實體關係、商業規則 |
| 03 | [模組與邊界](./03_MODULES_AND_BOUNDARIES.md) | 模組定義、檔案路徑、API |
| 04 | [資料庫綱要](./04_DATABASE_SCHEMA.md) | 資料表、ENUM、索引 |
| 05 | [API 規格](./05_API_SPECIFICATION.md) | 完整 273 個端點 |
| 06 | [權限與 RBAC](./06_PERMISSIONS_RBAC.md) | 84 權限、11 角色 |
| 07 | [安全與稽核](./07_SECURITY_AUDIT.md) | 中間件、GeoIP、Session |
| 08 | [出勤模組](./08_ATTENDANCE_MODULE.md) | 打卡、請假、加班、行事曆 |
| 09 | [擴展性](./09_EXTENSIBILITY.md) | 已完成/未來擴展規劃 |

### 子系統模組文件

| 文件 | 說明 |
|------|------|
| [動物管理系統](./modules/ANIMAL_MANAGEMENT.md) | 豬隻生命週期管理 |
| [AUP 審查系統](./modules/AUP_SYSTEM.md) | IACUC 計畫書審查 |
| [ERP 進銷存系統](./modules/ERP_SYSTEM.md) | 採購、庫存、銷售 |
| [HR 人事系統](./modules/HR_SYSTEM.md) | 出勤、請假、加班 |
| [通知系統](./modules/NOTIFICATION_SYSTEM.md) | Email、站內通知、排程 |

### 指南

| 文件 | 說明 |
|------|------|
| [稽核日誌](./guides/AUDIT_LOGGING.md) | GLP 合規稽核操作指南 |
| [命名慣例](./guides/NAMING_CONVENTIONS.md) | 程式碼命名規範 |
| [儲存設定](./guides/STORAGE_SETUP.md) | 檔案儲存設定 |
| [UI/UX 指南](./guides/UI_UX_GUIDELINES.md) | 前端設計規範 |

### 其他

| 文件 | 說明 |
|------|------|
| [資料庫 ERD](./database_erd.md) | 實體關係圖 |

---

## 技術堆疊摘要

| 層級 | 技術 |
|------|------|
| 前端 | React 18 + TypeScript + TailwindCSS + shadcn/ui |
| 後端 | Rust + Axum 0.7 + SQLx 0.8 |
| 資料庫 | PostgreSQL 16 |
| 部署 | Docker Compose (4 services) |

---

## 系統統計

| 指標 | 數值 |
|------|------|
| API 端點 | ~273 |
| 資料庫表 | 65+ |
| 權限 | 84 |
| 角色 | 11 |
| 遷移檔案 | 10 |
| 前端頁面 | 62 |
| 前端元件 | 62 |
| 後端服務檔 | 74 |
| 後端處理器 | 42 |
