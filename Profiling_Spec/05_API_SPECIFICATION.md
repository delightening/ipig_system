# API 規格

> **版本**：2.0  
> **最後更新**：2026-01-18  
> **對象**：開發人員、前端工程師

---

## 1. 概覽

### 1.1 基礎 URL
- **開發環境**：`http://localhost:8080/api`
- **生產環境**：`https://yourdomain.com/api`

### 1.2 認證
所有端點（登入除外）需於標頭提供 JWT 令牌：
```
Authorization: Bearer <token>
```

### 1.3 回應格式
所有回應皆為 JSON 格式。成功回應結構：
```json
{
  "data": { ... },
  "message": "操作成功"
}
```

錯誤回應結構：
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "錯誤說明"
  }
}
```

### 1.4 分頁
列表端點支援以下查詢參數：
- `page` - 頁碼（預設：1）
- `per_page` - 每頁筆數（預設：20，最大：100）
- `sort_by` - 排序欄位
- `sort_order` - `asc` 或 `desc`

---

## 2. 認證 API

### 2.1 登入
```
POST /auth/login
```

**請求**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**回應**
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "display_name": "使用者名稱",
    "roles": ["admin", "pi"],
    "permissions": ["user.read", "protocol.create"]
  }
}
```

### 2.2 刷新令牌
```
POST /auth/refresh
```

### 2.3 登出
```
POST /auth/logout
```

### 2.4 忘記密碼
```
POST /auth/forgot-password
Content: { "email": "user@example.com" }
```

### 2.5 重設密碼
```
POST /auth/reset-password
Content: { "token": "reset_token", "new_password": "new_pass123" }
```

---

## 3. 使用者 API

### 3.1 列表使用者（管理員）
```
GET /admin/users?page=1&per_page=20&search=keyword&is_active=true
```

### 3.2 建立使用者（管理員）
```
POST /admin/users
Content: { "email": "...", "display_name": "...", "password": "...", "roles": ["uuid"] }
```

### 3.3 更新使用者（管理員）
```
PUT /admin/users/:id
Content: { "display_name": "...", "is_active": true, "roles": ["uuid"] }
```

### 3.4 取得個人資料
```
GET /users/me
```

### 3.5 更新個人資料
```
PUT /users/me
Content: { "display_name": "...", "phone": "...", "theme_preference": "dark" }
```

### 3.6 變更密碼
```
POST /users/me/change-password
Content: { "current_password": "...", "new_password": "..." }
```

---

## 4. 計畫書 API

### 4.1 列表計畫書
```
GET /protocols?status=DRAFT&search=keyword
```

### 4.2 建立計畫書
```
POST /protocols
Content: { "title": "計畫名稱", "working_content": {...} }
```

### 4.3 取得計畫書
```
GET /protocols/:id
```

### 4.4 更新計畫書
```
PUT /protocols/:id
Content: { "title": "...", "working_content": {...} }
```

### 4.5 送審計畫書
```
POST /protocols/:id/submit
```

### 4.6 變更狀態（管理員/審查員）
```
POST /protocols/:id/status
Content: { "status": "APPROVED", "remark": "核准備註" }
```

### 4.7 取得版本歷程
```
GET /protocols/:id/versions
```

### 4.8 取得特定版本
```
GET /protocols/:id/versions/:version_no
```

---

## 5. 審查 API

### 5.1 取得審查指派
```
GET /reviews/assignments?protocol_id=uuid
```

### 5.2 建立審查指派（管理員）
```
POST /reviews/assignments
Content: { "protocol_id": "uuid", "reviewer_id": "uuid" }
```

### 5.3 移除審查指派（管理員）
```
DELETE /reviews/assignments/:id
```

### 5.4 取得審查意見
```
GET /reviews/comments?protocol_version_id=uuid
```

### 5.5 建立審查意見
```
POST /reviews/comments
Content: { "protocol_version_id": "uuid", "content": "意見內容" }
```

### 5.6 解決意見
```
POST /reviews/comments/:id/resolve
```

---

## 6. 動物（豬隻）API

### 6.1 列表豬隻
```
GET /pigs?status=in_experiment&iacuc_no=IACUC-2026-001&search=耳號
```

### 6.2 依欄位取得
```
GET /pigs/by-pen?building=A
回應：以 pen_location 分組的豬隻
```

### 6.3 取得豬隻詳情
```
GET /pigs/:id
包含：觀察紀錄、手術紀錄、體重、疫苗
```

### 6.4 建立豬隻
```
POST /pigs
Content: { "ear_tag": "001", "breed": "miniature", "gender": "male", ... }
```

### 6.5 更新豬隻
```
PUT /pigs/:id
Content: { "pen_location": "A02", "status": "assigned", ... }
```

### 6.6 刪除豬隻（軟刪除）
```
DELETE /pigs/:id
Content: { "reason": "刪除原因" }
```

### 6.7 批次指派
```
POST /pigs/batch/assign
Content: { "pig_ids": [1, 2, 3], "iacuc_no": "IACUC-2026-001" }
```

### 6.8 批次開始實驗
```
POST /pigs/batch/start-experiment
Content: { "pig_ids": [1, 2, 3], "experiment_date": "2026-01-20" }
```

---

## 7. 豬隻紀錄 API

### 7.1 觀察紀錄
```
GET /pigs/:id/observations
POST /pigs/:id/observations
Content: { "event_date": "2026-01-20", "record_type": "observation", "content": "正常" }

PUT /observations/:id
DELETE /observations/:id
```

### 7.2 手術紀錄
```
GET /pigs/:id/surgeries
POST /pigs/:id/surgeries
Content: { "surgery_date": "2026-01-20", "surgery_site": "腹部", ... }

PUT /surgeries/:id
DELETE /surgeries/:id
```

### 7.3 體重紀錄
```
GET /pigs/:id/weights
POST /pigs/:id/weights
Content: { "measure_date": "2026-01-20", "weight": 25.5 }
```

### 7.4 疫苗紀錄
```
GET /pigs/:id/vaccinations
POST /pigs/:id/vaccinations
Content: { "administered_date": "2026-01-20", "vaccine": "狂犬病疫苗" }
```

### 7.5 犧牲紀錄
```
GET /pigs/:id/sacrifice
POST /pigs/:id/sacrifice
Content: { "sacrifice_date": "2026-01-20", "confirmed_sacrifice": true }
```

### 7.6 病理報告
```
GET /pigs/:id/pathology
POST /pigs/:id/pathology
Content: { ... }
```

### 7.7 獸醫建議
```
POST /observations/:id/vet-recommendations
POST /surgeries/:id/vet-recommendations
Content: { "content": "建議內容" }
```

### 7.8 匯出醫療資料
```
POST /pigs/:id/export
回應：PDF 檔案
```

---

## 8. ERP API

### 8.1 單據
```
GET /documents?doc_type=PO&status=draft
POST /documents
PUT /documents/:id
POST /documents/:id/submit
POST /documents/:id/approve
POST /documents/:id/cancel
```

### 8.2 產品
```
GET /products?category_code=DRG&search=keyword
POST /products
PUT /products/:id
POST /products/with-sku（建立並自動產生 SKU）
```

### 8.3 SKU 管理
```
GET /sku/categories
GET /sku/categories/:code/subcategories
POST /sku/generate
Content: { "category_code": "DRG", "subcategory_code": "ANT" }
回應：{ "sku": "DRG-ANT-001" }
```

### 8.4 倉庫
```
GET /warehouses
POST /warehouses
PUT /warehouses/:id
```

### 8.5 夥伴
```
GET /partners?partner_type=supplier
POST /partners
PUT /partners/:id
```

### 8.6 庫存
```
GET /inventory/on-hand?warehouse_id=uuid
GET /inventory/ledger?product_id=uuid&from=2026-01-01&to=2026-01-31
GET /inventory/low-stock
GET /inventory/expiring?days=30
```

---

## 9. 人事 API

### 9.1 出勤
```
GET /hr/attendance?from=2026-01-01&to=2026-01-31
POST /hr/attendance/clock-in
POST /hr/attendance/clock-out
PUT /hr/attendance/:id（手動修正）
```

### 9.2 加班
```
GET /hr/overtime?status=approved
POST /hr/overtime
Content: { "overtime_date": "2026-01-20", "start_time": "...", "end_time": "...", "overtime_type": "平日" }

POST /hr/overtime/:id/submit
POST /hr/overtime/:id/approve
POST /hr/overtime/:id/reject
```

### 9.3 請假
```
GET /hr/leaves?status=PENDING_L1
POST /hr/leaves
Content: { "leave_type": "ANNUAL", "start_date": "...", "end_date": "...", "reason": "..." }

GET /hr/leaves/:id
POST /hr/leaves/:id/submit
POST /hr/leaves/:id/approve
POST /hr/leaves/:id/reject
POST /hr/leaves/:id/cancel
POST /hr/leaves/:id/revoke
```

### 9.4 餘額
```
GET /hr/balances/annual
GET /hr/balances/comp-time
GET /hr/balances/summary
```

---

## 10. 行事曆同步 API

```
GET /hr/calendar/status
POST /hr/calendar/connect
Content: { "calendar_id": "...", "calendar_name": "..." }

POST /hr/calendar/disconnect
POST /hr/calendar/sync（手動同步）
PUT /hr/calendar/settings

GET /hr/calendar/conflicts
POST /hr/calendar/conflicts/:id/resolve
Content: { "resolution": "keep_ipig" | "accept_google" | "dismiss" }
```

---

## 11. 通知 API

```
GET /notifications?is_read=false
GET /notifications/unread-count
POST /notifications/read
Content: { "notification_ids": ["uuid1", "uuid2"] }

POST /notifications/read-all
GET /notifications/settings
PUT /notifications/settings
```

---

## 12. 稽核 API（管理員）

```
GET /admin/audit/activities?user_id=uuid&from=2026-01-01
GET /admin/audit/activities/user/:id（使用者時間軸）
GET /admin/audit/activities/entity/:type/:id（實體歷程）

GET /admin/audit/logins?user_id=uuid
GET /admin/audit/sessions?is_active=true
POST /admin/audit/sessions/:id/logout（強制登出）

GET /admin/audit/alerts?status=open
POST /admin/audit/alerts/:id/acknowledge

GET /admin/audit/dashboard
```

---

## 13. 報表 API

```
GET /reports/stock-on-hand?warehouse_id=uuid
GET /reports/stock-ledger?from=2026-01-01&to=2026-01-31
GET /reports/purchase-lines?from=2026-01-01&to=2026-01-31
GET /reports/sales-lines?from=2026-01-01&to=2026-01-31
GET /reports/cost-summary?from=2026-01-01&to=2026-01-31

GET /scheduled-reports
POST /scheduled-reports
PUT /scheduled-reports/:id
DELETE /scheduled-reports/:id

GET /report-history
```

---

## 14. 設施 API

```
GET /facilities/species
POST /facilities/species
PUT /facilities/species/:id

GET /facilities
POST /facilities
PUT /facilities/:id

GET /facilities/buildings
POST /facilities/buildings
PUT /facilities/buildings/:id

GET /facilities/zones
POST /facilities/zones
PUT /facilities/zones/:id

GET /facilities/pens
POST /facilities/pens
PUT /facilities/pens/:id

GET /facilities/departments
POST /facilities/departments
PUT /facilities/departments/:id
```

---

## 15. 常見錯誤代碼

| 代碼 | HTTP 狀態 | 說明 |
|------|-----------|------|
| UNAUTHORIZED | 401 | 未授權或令牌無效 |
| FORBIDDEN | 403 | 權限不足 |
| NOT_FOUND | 404 | 資源不存在 |
| VALIDATION_ERROR | 400 | 輸入驗證失敗 |
| DUPLICATE_ENTRY | 409 | 資源已存在 |
| INTERNAL_ERROR | 500 | 伺服器錯誤 |

---

*下一章：[權限與 RBAC](./06_PERMISSIONS_RBAC.md)*
