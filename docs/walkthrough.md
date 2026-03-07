# 實作說明

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
Admin 登入 → 實驗動物管理 → 動物欄位修正審核 → 檢視待審清單 → 批准 / 拒絕
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
- **Admin 審核**：側欄「實驗動物管理」→「動物欄位修正審核」

## 5. 注意事項

- 耳號須為三位數（如 001、002）
- 出生日期格式：`YYYY-MM-DD`
- 性別：`male` | `female`
- 品種：`miniature` | `white` | `LYD` | `other`（前端 minipig 會對應 miniature）
- 僅 **admin** 角色可審核；一般 staff 僅能提交申請
