# iPig 系統重構計畫書：Rust 至 Laravel 改寫方案

本計畫旨在指導將現有的 **Rust (Axum) + React** 系統重構為 **Laravel + Livewire** 架構。此重構將以 `描述/` 目錄下的 SRS 文件為功能基準，並確保 GLP 合規性。

## 1. 重構目標與技術棧
- **後端框架**: Laravel 11.x
- **前端互動**: Livewire 3.x (取代 React)
- **樣式系統**: Tailwind CSS (保持一致)
- **資料庫**: PostgreSQL 16 (沿用現有數據)
- **認證機制**: Laravel Fortify + Jetstream (或自訂 Session-based Auth)
- **GLP 合規**: 保留電子簽章、稽核日誌及資料版本控制

## 2. 技術對齊表 (Mapping)

| 組件 | 目前 Rust (Axum) | Laravel 重構對應 |
|------|-----------------|-----------------|
| **路由** | `routes/` (Axum) | `routes/web.php` & `routes/api.php` |
| **控制器** | `handlers/` | `app/Http/Controllers/` |
| **業務邏輯** | `services/` | `app/Services/` |
| **資料存取** | `repositories/` (SQLx) | `app/Models/` (Eloquent) |
| **DTO/模型** | `models/` | `app/Http/Requests/` & `app/Models/` |
| **認證** | JWT (Cookie) | Laravel Session/Sanctum + Fortify |
| **授權** | `services/access.rs` | Laravel Policies & Middleware |
| **前端** | React (TanStack Query) | Livewire Components (Reactive UI) |
| **驗證** | Zod (Frontend) | Laravel Request Validation |
| **背景標籤** | `tokio` spawn | Laravel Queues (Redis/Database) |
| **文件處理** | `image/PDF` scripts | Laravel Media Library / Browsershot |

## 3. 第一階段：基礎環境與架構遷移 (P0)
### [NEW] 專案初始化
- 使用 `Composer` 建立 Laravel 專案。
- 配置 `docker-compose.yml` 以整合 PHP-FPM, Nginx 及既有的 PostgreSQL。
- **遺失的 Migrations 補齊**：針對 SRS 提到的 19 個遷移，將 Rust `migrations/` 中的 15 個 SQL 轉化為 Laravel Migration，並找出缺少的 4 個功能點。

### [Service] 認證與權限 (RBAC)
- 實作 `描述/01_User_and_Roles_Module.md` 定義的 38 項功能。
- 建立 `User`, `Role`, `Permission` 模型與關聯。
- 移植 Rust 的 Argon2 密碼驗證邏輯（或全體重設）。

## 4. 第二階段：核心業務重構 (P0)
### [Module] AUP 計畫書審查
- 基於 `描述/02_AUP_Protocol_Review_Module.md` 實作。
- 關鍵挑戰：將 Rust 處理的複雜 AUP JSON 結構 (`working_content`) 完整對齊至 Laravel 模型。
- 實作多層審查狀態機（State Pattern）。

### [Module] 動物管理與醫療紀錄
- 實作動物生命週期與 8 大醫療操作。
- 確保 PDF 匯出功能與 Rust 版本一致。

## 5. 第三階段：ERP 與人資支援系統 (P1-P2)
- 重構進銷存 (ERP) 邏輯，特別是庫存流水與 GLP 追蹤。
- 實作 HR 出勤與 Google Calendar 同步（使用 Laravel Socialite/Spatie Google Calendar）。

## 6. 驗證與切割計畫
- **並行測試**：建立整合測試環境，比對 Rust API 與 Laravel API 的輸出一致性。
- **資料遷移**：編寫遷移腳本將 Rust 系統中的舊資料導入 Laravel 新結構。
- **GLP 驗證**：重新進行電子簽章效力與稽核軌跡完整性檢查。

---

## 具體執行步驟
1. **[環境準備]** 使用 `uv` 輔助開發輔助腳本，並初始化 Laravel 基礎結構。
2. **[模型對齊]** 根據 SQLx 結構產生 Eloquent Models。
3. **[介面開發]** 利用 Livewire 將原本的 React 元件邏輯轉化為伺服器端渲染組件。

> [!IMPORTANT]
> **關於資料庫遷移**：目前的 15 個 SQL 檔案需優先轉化為 Laravel 檔案，以確保資料結構一致性。
