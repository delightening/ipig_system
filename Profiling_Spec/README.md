# iPig System Profiling Spec

> **iPig 豬博士動物科技系統 — 技術規格文件**  
> **版本**：4.0 | **最後更新**：2026-02-16

---

## 📋 文件索引

| # | 文件 | 說明 |
|---|------|------|
| 01 | [架構概覽](./01_ARCHITECTURE_OVERVIEW.md) | 系統目的、技術堆疊、分層架構、部署 |
| 02 | [核心領域模型](./02_CORE_DOMAIN_MODEL.md) | 實體定義、關係、商業規則 |
| 03 | [模組與邊界](./03_MODULES_AND_BOUNDARIES.md) | 模組拆分、API 前綴、檔案結構 |
| 04 | [資料庫綱要](./04_DATABASE_SCHEMA.md) | 資料表定義、ENUM、遷移統計 |
| 05 | [API 規格](./05_API_SPECIFICATION.md) | 完整端點列表、認證、回應格式 |
| 06 | [權限與 RBAC](./06_PERMISSIONS_RBAC.md) | 角色、權限、存取控制矩陣 |
| 07 | [安全與稽核](./07_SECURITY_AUDIT.md) | 安全中間件、登入追蹤、GLP 合規 |
| 08 | [出勤模組](./08_ATTENDANCE_MODULE.md) | 打卡、請假、加班、行事曆同步 |
| 09 | [擴展性](./09_EXTENSIBILITY.md) | 已完成功能、未來規劃、擴展指南 |
| — | [資料庫 ERD](./database_erd.md) | 完整 ER 圖（Mermaid）|

---

## 🏗️ 系統概覽

| 項目 | 說明 |
|------|------|
| **前端** | React 18 + TypeScript + Vite + TailwindCSS + shadcn/ui |
| **後端** | Rust + Axum + SQLx + Tokio |
| **資料庫** | PostgreSQL 16 |
| **部署** | Docker Compose (4 服務) |

---

## 🧩 功能模組

| 模組 | 說明 |
|------|------|
| **動物管理** | 動物紀錄、觀察/手術、血液檢查、安樂死、匯入匯出 |
| **AUP 審查** | IACUC 計畫書、多層審查、變更申請、PDF 匯出 |
| **ERP 進銷存** | 產品/SKU、倉庫/儲位、單據流程、庫存追蹤 |
| **人事管理** | 出勤打卡、多級請假審核、加班、Google 行事曆 |
| **安全稽核** | Activity Logger、登入追蹤、工作階段管理、GeoIP |
| **設施管理** | 物種、設施、棟舍、區域、欄位階層管理 |

---

## 📊 系統統計

| 指標 | 數值 |
|------|------|
| 前端頁面 | 62+ |
| 前端元件 | 62+ |
| API 端點 | ~273 |
| 資料表 | ~70 |
| 後端 Handler 檔案 | 42 |
| 後端 Service 檔案 | 74 |
| 後端 Model 檔案 | 23 |

---

*最後更新：2026-02-16*
