# 實作說明

## 2026-03-09 請假與加班改為以小時計算（0.5 小時為單位）

**需求**：請假與加班應依小時計算，以 0.5 小時為一單位。

### 請假

- **前端**（`useLeaveRequestForm.ts`、`HrLeavePage.tsx`）：
  - 表單欄位由「天數」改為「時數」，`step=0.5`、`min=0.5`
  - 日期與時數雙向計算：日期變更 → 時數 = 天數 × 8 並 round 至 0.5；時數變更 → 推算結束日
  - 送出時傳 `total_hours` 與 `total_days`（total_days = total_hours / 8）
  - 列表/表格顯示「X 小時」取代「X 天」
- **後端**（`leave.rs`）：
  - `LeaveRequestWithUser` 新增 `total_hours`，list 查詢回傳該欄位
  - `create_leave` 驗證時數須為 0.5 的倍數，若傳 `total_hours` 則以 `total_hours` 為準並推算 `total_days`

### 加班

- **後端**（`overtime.rs`）：
  - `create_overtime` 計算的時數以 0.5 小時為單位四捨五入
- **前端**（`HrOvertimePage.tsx`）：
  - 新增加班 Dialog 顯示「預估加班時數」，以 start/end time 計算並 round 至 0.5

---

## 2026-03-07 附件 API 500 錯誤修復與 Console 錯誤調查

**現象**：專案詳情頁「附件」標籤顯示「尚無附件」，Console 出現：
- `GET /api/v1/attachments/...` → **500 Internal Server Error**（2 次）
- `GET /api/v1/notifications/unread-count` → **401 Unauthorized**（多筆）
- `GET /api/v1/amendments/pending-count` → **401 Unauthorized**（多筆）

### 1. 500 錯誤（優先修復）

**根因**：`attachments` 表 `entity_id` 為 PostgreSQL `UUID`，Rust `Attachment` struct 定義為 `entity_id: String`。sqlx 在 SELECT 時無法將 DB 的 UUID 直接 deserialize 成 String，造成 500。

**修正**（`backend/src/handlers/upload.rs`）：
- `list_attachments`：SELECT 改為 `entity_id::text AS entity_id`，WHERE 改為 `entity_id::text = $2`
- `download_attachment`、`delete_attachment`：SELECT 由 `SELECT *` 改為明確欄位列表，`entity_id` 用 `entity_id::text AS entity_id`

### 2. 401 錯誤（notifications / amendments）

**說明**：`/notifications/unread-count` 與 `/amendments/pending-count` 皆需認證。401 常見情境：
- Session 逾時、Token 失效
- 後端重啟後 Session 失效
- Cookie 未正確傳送（跨網域 / SameSite 等）

**現狀**：前端已用 `enabled: !!user` 僅在登入後才呼叫；401 時 api 會嘗試 refresh token，失敗則登出。無程式邏輯錯誤，屬預期行為。

---

## 2026-03-04 全專案資料夾整理與分類

**目標**：正確分類資料夾、統一導覽與連結，提升閱讀體驗。

**變更摘要**：
- **docs**：維運手冊 `OPERATIONS.md` 移入 `docs/operations/`，與 COMPOSE、ENV_AND_DB、TUNNEL、SSL_SETUP 等歸為「環境與建置」；`docs/README.md` 補齊維運手冊入口、閱讀建議、目錄結構註解。
- **根目錄 README**：新增「資料夾一覽」表（backend、frontend、docs、scripts、tests、monitoring、deploy、.github）及「依角色閱讀」；文件導覽加入 `docs/operations/OPERATIONS.md`。
- **security-compliance**：`SOC2_READINESS.md`、`SLA.md` 內對 OPERATIONS、DR_RUNBOOK、DEPLOYMENT 的連結改為相對路徑（`../operations/OPERATIONS.md`、`../runbooks/DR_RUNBOOK.md`、`../DEPLOYMENT.md`）。
- **monitoring/**：新增 `README.md`，說明 Prometheus、Alertmanager、Promtail 目錄結構、用途與啟用方式。
- **deploy/**：新增 `README.md`，說明 deploy 目錄下監控、隧道、WAF 設定分類與相關文件。

---

## 2026-03-03 疫苗紀錄刪除失效修復與刪除功能檢視

**問題**：疫苗紀錄按刪除後顯示「成功」但列表仍顯示該筆紀錄。

**根因**：`list_vaccinations` 查詢未過濾 `deleted_at IS NULL`，軟刪除後列表仍回傳已刪除紀錄。

**修正**：
- `backend/src/services/animal/medical.rs`：`list_vaccinations` 加入 `AND deleted_at IS NULL`
- 照護紀錄：Migration 012 新增軟刪除欄位；handler 改為軟刪除 + DeleteRequest + AuditService；PainAssessmentTab 改用 DeleteReasonDialog

**刪除功能檢視**：疫苗、體重、觀察、手術、血液檢查、動物、照護紀錄均已為軟刪除 + 操作日誌。

### 軟刪除欄位統一：deleted_at

血液檢查、報表、安樂死等原使用 `is_deleted = false` 過濾，已改為 `deleted_at IS NULL`，與照護紀錄、疫苗紀錄等一致。

- **Migration 013**：`animal_blood_tests` 移除 `is_deleted` 欄位，僅保留 `deleted_at`
- **程式碼**：`blood_test.rs`、`report.rs`、`euthanasia.rs` 改用 `deleted_at IS NULL` 過濾
- **型別**：`AnimalBloodTest` 移除 `is_deleted`，前端 `AnimalBloodTestWithItems.blood_test` 改為 `deleted_at?: string | null`

---

## 動物欄位修正申請功能

> **日期**：2026-03-02  
> **需求**：耳號、出生日期、性別、品種等欄位建立後不可直接修改；若 staff 輸入錯誤，可經 admin 批准後修正。

## 1. 流程概覽

```
Staff 發現資料錯誤 → 點擊「申請修正」→ 填寫欄位、新值、原因 → 提交
                                                              ↓
Admin 登入 → 實驗動物管理 → 修正審核 → 檢視待審清單 → 批准 / 拒絕
                                                              ↓
批准後 → 系統自動套用修正至動物資料
```

## 2. 資料庫

- **Migration**：`backend/migrations/011_animal_field_correction_requests.sql`
- **資料表**：`animal_field_correction_requests`
  - `field_name`：`ear_tag` | `birth_date` | `gender` | `breed`
  - `status`：`pending` | `approved` | `rejected`
  - 含 `requested_by`、`reviewed_by`、`reviewed_at` 等審核欄位

**執行遷移**（需 DATABASE_URL 環境變數）：

```bash
cd backend
cargo sqlx migrate run
# 或啟動後端時會自動執行
```

## 3. 後端 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| POST | `/api/v1/animals/:id/field-corrections` | 建立修正申請 | `animal.animal.edit` |
| GET | `/api/v1/animals/:id/field-corrections` | 查詢該動物申請列表 | `animal.animal.edit` |
| GET | `/api/v1/animals/animal-field-corrections/pending` | 列出待審申請 | admin |
| POST | `/api/v1/animals/animal-field-corrections/:id/review` | 批准/拒絕 | admin |

**建立申請 Request Body**：

```json
{
  "field_name": "ear_tag",
  "new_value": "002",
  "reason": "建檔時誤植"
}
```

**審核 Request Body**：

```json
{
  "approved": true
}
// 或
{
  "approved": false,
  "reject_reason": "經查證原資料正確"
}
```

## 4. 前端入口

- **動物詳情頁**（動物資料 Tab）：右上角「申請修正」按鈕
- **動物編輯頁**：標題列右側「申請修正（耳號/出生日期/性別/品種）」按鈕
- **Admin 審核**：側欄「實驗動物管理」→「修正審核」

## 5. 注意事項

- 耳號須為三位數（如 001、002）
- 出生日期格式：`YYYY-MM-DD`
- 性別：`male` | `female`
- 品種：`miniature` | `white` | `LYD` | `other`（前端 minipig 會對應 miniature）
- 僅 **admin** 角色可審核；一般 staff 僅能提交申請

## 2026-03-23 AI 資料查詢接口

**需求**：設計一個 AI 接口，讓外部 AI 系統能夠透過 API key 認證進入 iPig System 查閱資料。

### 架構設計

採用獨立的認證機制（API key），與現有 JWT 使用者認證分離：

```
AI Client → Bearer ipig_ai_xxx → ai_auth_middleware → AI Handlers → Service → Repository → DB
                                                          ↑
管理員 → JWT Auth → Admin Handlers（API key CRUD）
```

### 新增檔案

| 檔案 | 職責 |
|------|------|
| `migrations/017_ai_api_keys.sql` | 新增 `ai_api_keys` 與 `ai_query_logs`（分區表） |
| `src/models/ai.rs` | AI 相關 request/response DTOs + DB entities |
| `src/middleware/ai_auth.rs` | API key 認證 middleware（SHA-256 驗證、過期檢查、scope 權限） |
| `src/repositories/ai.rs` | AI 相關 SQL 查詢（key CRUD、資料查詢、日誌寫入） |
| `src/services/ai.rs` | AI 業務邏輯（key 管理、查詢執行、schema 描述） |
| `src/handlers/ai.rs` | HTTP handlers（管理端 + AI 端） |
| `src/routes/ai.rs` | 路由定義（管理端走 JWT、AI 端走 API key） |

### API 端點

**管理端（需管理員 JWT）：**

| Method | Path | 說明 |
|--------|------|------|
| POST | `/api/ai/admin/keys` | 建立 AI API key |
| GET | `/api/ai/admin/keys` | 列出所有 API keys |
| PUT | `/api/ai/admin/keys/:id/toggle` | 停用/啟用 key |
| DELETE | `/api/ai/admin/keys/:id` | 刪除 key |

**AI 資料查詢端（需 API key）：**

| Method | Path | 說明 |
|--------|------|------|
| GET | `/api/ai/overview` | 系統概覽（動物/計畫/觀察數量統計） |
| GET | `/api/ai/schema` | API schema（告訴 AI 可查什麼、有哪些 filter） |
| POST | `/api/ai/query` | 執行資料查詢（指定 domain + filters + pagination） |

### 支援的查詢領域 (domains)

- `animals` — 動物基本資料（耳標、品種、狀態、性別等），支援 status/breed/keyword 篩選
- `observations` — 觀察紀錄，支援 animal_id/days 篩選
- `surgeries` — 手術紀錄，支援 animal_id 篩選
- `weights` — 體重量測紀錄，支援 animal_id 篩選
- `protocols` — AUP 計畫書，支援 status 篩選
- `facilities` — 設施清單

### 安全設計

- API key 以 SHA-256 hash 儲存，不存明文
- Key 前綴 `ipig_ai_` 用於辨識類型
- 每個 key 有獨立的 scope 權限（如 `read`, `animal.read`, `protocol.read`, `*`）
- 支援過期時間設定
- 每次使用自動記錄至 `ai_query_logs`（分區表）
- 排序欄位使用白名單驗證（防 SQL injection）
- 資料查詢為唯讀，AI 無法修改任何資料

### 使用範例

```bash
# 1. 管理員建立 API key
curl -X POST /api/ai/admin/keys \
  -H "Authorization: Bearer <admin_jwt>" \
  -d '{"name": "Claude AI", "scopes": ["read"]}'

# 2. AI 查看系統概覽
curl /api/ai/overview -H "Authorization: Bearer ipig_ai_xxx..."

# 3. AI 查看可用 schema
curl /api/ai/schema -H "Authorization: Bearer ipig_ai_xxx..."

# 4. AI 查詢動物資料
curl -X POST /api/ai/query \
  -H "Authorization: Bearer ipig_ai_xxx..." \
  -d '{"domain": "animals", "filters": {"status": "alive"}, "page": 1, "per_page": 20}'
```
