# iPig System Profiling Spec

> **iPig 豬博士動物科技系統 — 技術規格文件**  
> **版本**：7.1 | **最後更新**：2026-03-14
> **狀態**：功能完整，上線準備中

---

## 📋 文件索引

| # | 文件 | 說明 |
|---|------|------|
| 01 | [架構概覽](./architecture/01_ARCHITECTURE_OVERVIEW.md) | 系統目的、技術堆疊、分層架構、部署 |
| 02 | [核心領域模型](./architecture/02_CORE_DOMAIN_MODEL.md) | 實體定義、關係、商業規則 |
| 03 | [模組與邊界](./architecture/03_MODULES_AND_BOUNDARIES.md) | 模組拆分、API 前綴、檔案結構 |
| 04 | [資料庫綱要](./architecture/04_DATABASE_SCHEMA.md) | 資料表定義、ENUM、遷移統計 |
| 05 | [API 規格](./architecture/05_API_SPECIFICATION.md) | 完整端點列表、認證、回應格式 |
| 06 | [權限與 RBAC](./architecture/06_PERMISSIONS_RBAC.md) | 角色、權限、存取控制矩陣 |
| 07 | [安全與稽核](./architecture/07_SECURITY_AUDIT.md) | 安全中間件、2FA、WAF、GLP 合規 |
| 08 | [出勤模組](./modules/08_ATTENDANCE_MODULE.md) | 打卡、請假、加班、行事曆同步 |
| 09 | [擴展性](./architecture/09_EXTENSIBILITY.md) | 已完成功能、未來規劃、擴展指南 |
| — | [資料庫 ERD](./architecture/database_erd.md) | 完整 ER 圖（Mermaid）|
| — | [guides/](./guides/) | 稽核日誌、命名慣例、UI/UX、儲存設定 |

---

## 🏗️ 系統概覽

| 項目 | 說明 |
|------|------|
| **前端** | React 18 + TypeScript + Vite + TailwindCSS + shadcn/ui |
| **後端** | Rust + Axum 0.7 + SQLx 0.8 + Tokio |
| **資料庫** | PostgreSQL 16 |
| **認證** | JWT (HttpOnly Cookie) + Refresh Token + TOTP 2FA |
| **部署** | Docker Compose（核心 5 服務 + prod/waf/monitoring overlay） |

---

## 🧩 功能模組

| 模組 | 說明 |
|------|------|
| **動物管理** | 動物紀錄、觀察/手術、血液檢查、安樂死、轉讓、猝死登記、**欄位修正申請（admin 批准）**、手寫簽章、疼痛評估 |
| **AUP 審查** | IACUC 計畫書、多層審查、變更申請、電子簽章、PDF 匯出 |
| **ERP 進銷存** | 產品/SKU、倉庫/儲位、單據流程、庫存追蹤、低庫存/效期警示 |
| **人事管理** | 出勤打卡（IP+GPS）、多級請假審核、加班、Google 行事曆 |
| **安全稽核** | Activity Logger、登入追蹤、Session 管理、GeoIP、稽核完整性 (HMAC) |
| **通知系統** | Email/站內通知、可配置路由、排程報表 |
| **設施管理** | 物種、設施、棟舍、區域、欄位階層管理 |

---

## 📊 系統統計

| 指標 | 數值 |
|------|------|
| 前端頁面 | 62+ |
| 前端元件 | 67+ |
| API 端點 | ~293 |
| 資料表 | ~73 |
| 資料庫遷移 | 11 (001_ ~ 011_) |
| 後端 Handler 檔案 | 59 |
| 後端 Service 檔案 | 94 |
| 後端 Repository 檔案 | 6 |
| 後端 Model 檔案 | 26 |
| E2E 測試 | 7 spec / 34 tests |
| Rust 單元測試 | 192+ |
| API 整合測試 | 25+ cases |
| OpenAPI 覆蓋率 | ≥ 90% |

---

## 🔗 相關文件

| 文件 | 說明 |
|------|------|
| [README.md](../README.md) | 專案總覽 |
| [QUICK_START.md](../user/QUICK_START.md) | 快速啟動 |
| [DEPLOYMENT.md](../deploy/DEPLOYMENT.md) | 部署與維運 |
| [ARCHITECTURE.md](./architecture/ARCHITECTURE.md) | 架構圖與資料流 |

---

*最後更新：2026-03-14*
