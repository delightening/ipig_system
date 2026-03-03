# 實作說明

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
Admin 登入 → 系統管理 → 動物欄位修正審核 → 檢視待審清單 → 批准 / 拒絕
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
| GET | `/api/v1/admin/animal-field-corrections/pending` | 列出待審申請 | admin |
| POST | `/api/v1/admin/animal-field-corrections/:id/review` | 批准/拒絕 | admin |

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
- **Admin 審核**：側欄「系統管理」→「動物欄位修正審核」

## 5. 注意事項

- 耳號須為三位數（如 001、002）
- 出生日期格式：`YYYY-MM-DD`
- 性別：`male` | `female`
- 品種：`miniature` | `white` | `LYD` | `other`（前端 minipig 會對應 miniature）
- 僅 **admin** 角色可審核；一般 staff 僅能提交申請
