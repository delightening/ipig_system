# 動物欄位修正申請功能 — 實作說明

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
