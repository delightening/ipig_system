# iPig 系統規格文件

> **版本**：2.0  
> **最後更新**：2026-02-03  
> **維護者**：開發團隊

---

## 系統概述

**豬博士 iPig 系統**（豬博士動物科技系統）是一套整合型實驗動物管理平台，包含四大子系統：

| 子系統 | 說明 |
|--------|------|
| **AUP 審查系統** | IACUC 動物使用計畫書提交、審查、核准流程 |
| **iPig ERP** | 進銷存管理、採購、庫存追蹤 |
| **動物管理系統** | 豬隻生命週期管理、實驗紀錄、醫療紀錄 |
| **人事管理系統** | 出勤、請假、加班、Google 行事曆同步 |

---

## 文件導覽

### 核心規格

| 文件 | 說明 |
|------|------|
| [系統總覽](./01_SYSTEM_OVERVIEW.md) | 系統架構、技術堆疊、部署架構 |
| [領域模型](./02_DOMAIN_MODEL.md) | 核心實體、關係、列舉 |
| [模組邊界](./03_MODULES_AND_BOUNDARIES.md) | 模組拆分、有界上下文 |
| [資料庫設計](./04_DATABASE_SCHEMA.md) | 資料表定義、遷移 |
| [API 規格](./05_API_SPECIFICATION.md) | REST API 完整參考 |
| [權限控制](./06_PERMISSIONS_RBAC.md) | 角色權限、RBAC |

### 子系統模組

| 文件 | 說明 |
|------|------|
| [AUP 審查系統](./modules/AUP_SYSTEM.md) | 計畫書管理、審查流程、狀態機 |
| [進銷存系統](./modules/ERP_SYSTEM.md) | 產品、庫存、採購、銷售 |
| [動物管理系統](./modules/ANIMAL_MANAGEMENT.md) | 豬隻管理、7 Tab 紀錄 |
| [人事管理系統](./modules/HR_SYSTEM.md) | 請假、特休、補休 |
| [通知系統](./modules/NOTIFICATION_SYSTEM.md) | Email、站內通知、排程 |

### 開發指南

| 文件 | 說明 |
|------|------|
| [UI/UX 指南](./guides/UI_UX_GUIDELINES.md) | 設計規範 |
| [命名規範](./guides/NAMING_CONVENTIONS.md) | 命名標準 |
| [稽核日誌](./guides/AUDIT_LOGGING.md) | GLP 合規稽核 |
| [儲存設定](./guides/STORAGE_SETUP.md) | 檔案儲存配置 |

### 專案管理

| 文件 | 說明 |
|------|------|
| [專案進度](./project/PROGRESS.md) | 開發進度追蹤 |
| [待辦事項](./project/TODO.md) | 功能規劃清單 |
| [版本歷程](./project/VERSION_HISTORY.md) | 文件變更紀錄 |

---

## 快速參考

### 技術堆疊

| 層級 | 技術 |
|------|------|
| 前端 | React 18, TypeScript, Vite 5, TailwindCSS, shadcn/ui, Zustand, React Query |
| 後端 | Rust 1.75+, Axum 0.7, SQLx 0.7, Tokio, Serde |
| 資料庫 | PostgreSQL 15 |
| 認證 | JWT (Access + Refresh tokens) |
| 部署 | Docker, Docker Compose, Nginx |

### 關鍵角色

| 角色代碼 | 名稱 | 說明 |
|----------|------|------|
| `SYSTEM_ADMIN` | 系統管理員 | 全系統權限 |
| `IACUC_STAFF` | 執行秘書 | 計畫管理、人事存取 |
| `EXPERIMENT_STAFF` | 試驗工作人員 | 動物紀錄、實驗操作 |
| `VET` | 獸醫師 | 動物健康、建議 |
| `WAREHOUSE_MANAGER` | 倉庫管理員 | ERP 操作 |
| `PI` | 計畫主持人 | 計畫提交 |
| `CLIENT` | 委託人 | 檢視委託計畫 |

### API 基礎路徑

- **開發環境**：`http://localhost:8080/api`
- **正式環境**：`https://ipig.example.com/api`

---

## 目錄結構

```
Profiling_Spec/
├── README.md                # 本文件
├── 01_SYSTEM_OVERVIEW.md    # 系統總覽
├── 02_DOMAIN_MODEL.md       # 領域模型
├── 03_MODULES_AND_BOUNDARIES.md
├── 04_DATABASE_SCHEMA.md
├── 05_API_SPECIFICATION.md
├── 06_PERMISSIONS_RBAC.md
├── modules/                 # 子系統規格
│   ├── AUP_SYSTEM.md
│   ├── ERP_SYSTEM.md
│   ├── ANIMAL_MANAGEMENT.md
│   ├── HR_SYSTEM.md
│   └── NOTIFICATION_SYSTEM.md
├── guides/                  # 開發指南
│   ├── UI_UX_GUIDELINES.md
│   ├── NAMING_CONVENTIONS.md
│   ├── AUDIT_LOGGING.md
│   └── STORAGE_SETUP.md
├── project/                 # 專案管理
│   ├── PROGRESS.md
│   ├── TODO.md
│   └── VERSION_HISTORY.md
└── archive/                 # 歸檔文件
```

---

*最後更新：2026-02-03*
