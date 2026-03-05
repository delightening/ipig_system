# API 規格

> **版本**：7.0  
> **最後更新**：2026-03-02  
> **對象**：開發人員、前端工程師

---

## 1. 概覽

### 1.1 基礎 URL
- **開發環境**：`http://localhost:8080/api`
- **生產環境**：`https://yourdomain.com/api`
- **Swagger UI**：`http://localhost:8000/swagger-ui/`

### 1.2 認證方式
所有受保護端點需要 JWT Token，透過 **HttpOnly Cookie** 自動傳送。啟用 2FA 的使用者登入時需額外提供 TOTP code。

### 1.3 Rate Limiting

| 類型 | 限制 | 適用範圍 |
|------|------|----------|
| 認證限流 | 100/min | `/auth/login`、`/auth/forgot-password`、`/auth/reset-password`、`/auth/refresh` |
| 寫入限流 | 120/min | POST/PUT/PATCH/DELETE 端點 |
| 上傳限流 | 30/min | 檔案上傳端點 |
| 一般 API | 600/min | 其餘 `/api/*` |

### 1.4 回應格式

```json
// 成功
{ "data": { ... } }

// 分頁
{ "data": [...], "total": 100, "page": 1, "per_page": 20 }

// 錯誤
{ "error": "UNAUTHORIZED", "message": "Invalid credentials" }
```

---

## 2. 認證 API（公開）

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/auth/login` | 使用者登入（回傳 JWT Cookie；若需 2FA 則回傳 requires_2fa + temp_token）|
| POST | `/auth/refresh` | 刷新 Access Token |
| POST | `/auth/2fa/verify` | TOTP 驗證登入（temp_token + code，支援備用碼）|
| POST | `/auth/forgot-password` | 請求密碼重設（寄送 Email）|
| POST | `/auth/reset-password` | 使用 Token 重設密碼 |

---

## 3. 認證 API（受保護）

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/auth/logout` | 使用者登出 |
| POST | `/auth/heartbeat` | Heartbeat 回報（前端定期呼叫）|
| POST | `/auth/2fa/setup` | TOTP 2FA 設定（產生 secret + otpauth URI + 備用碼）|
| POST | `/auth/2fa/confirm` | 驗證第一次 code 正式啟用 2FA |
| POST | `/auth/2fa/disable` | 停用 2FA（需密碼 + code）|
| POST | `/auth/confirm-password` | 敏感操作二級認證（密碼換取 reauth token，5 分鐘有效）|
| GET | `/me` | 取得個人資訊 |
| PUT | `/me` | 更新個人資訊 |
| PUT | `/me/password` | 變更密碼 |

---

## 4. 使用者偏好設定 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/me/preferences` | 取得所有偏好設定 |
| GET | `/me/preferences/:key` | 取得特定偏好 |
| PUT | `/me/preferences/:key` | 新增/更新偏好 |
| DELETE | `/me/preferences/:key` | 刪除偏好 |

---

## 5. 使用者管理 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| GET | `/users` | 使用者列表 | user.read |
| POST | `/users` | 建立使用者 | user.create |
| GET | `/users/:id` | 使用者詳情 | user.read |
| PUT | `/users/:id` | 更新使用者 | user.update |
| DELETE | `/users/:id` | 刪除使用者 | user.delete |
| PUT | `/users/:id/password` | 重設密碼 | user.update |
| POST | `/users/:id/impersonate` | 模擬登入 | admin |

---

## 6. 角色與權限 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| GET | `/roles` | 角色列表 | role.read |
| POST | `/roles` | 建立角色 | role.create |
| GET | `/roles/:id` | 角色詳情 | role.read |
| PUT | `/roles/:id` | 更新角色 | role.update |
| DELETE | `/roles/:id` | 刪除角色 | role.delete |
| GET | `/permissions` | 權限列表 | authenticated |

---

## 7. 倉庫管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/warehouses` | 倉庫列表 |
| POST | `/warehouses` | 建立倉庫 |
| GET | `/warehouses/:id` | 倉庫詳情 |
| PUT | `/warehouses/:id` | 更新倉庫 |
| DELETE | `/warehouses/:id` | 刪除倉庫 |
| PUT | `/warehouses/:id/layout` | 更新倉庫佈局 |

---

## 8. 倉庫儲位 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/storage-locations` | 儲位列表 |
| POST | `/storage-locations` | 建立儲位 |
| GET | `/storage-locations/:id` | 儲位詳情 |
| PUT | `/storage-locations/:id` | 更新儲位 |
| DELETE | `/storage-locations/:id` | 刪除儲位 |
| GET | `/storage-locations/:id/inventory` | 儲位庫存 |
| POST | `/storage-locations/:id/inventory` | 新增庫存品項 |
| PUT | `/storage-locations/inventory/:item_id` | 更新庫存品項 |
| POST | `/storage-locations/inventory/:item_id/transfer` | 庫存轉移 |
| GET | `/storage-locations/generate-code/:warehouse_id` | 產生儲位代碼 |

---

## 9. 產品管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/products` | 產品列表 |
| POST | `/products` | 建立產品 |
| GET | `/products/:id` | 產品詳情 |
| PUT | `/products/:id` | 更新產品 |
| DELETE | `/products/:id` | 刪除產品 |
| POST | `/products/with-sku` | 建立產品含 SKU |
| GET | `/categories` | 分類列表 |
| POST | `/categories` | 建立分類 |

---

## 10. SKU API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/sku/categories` | SKU 大類列表 |
| GET | `/sku/categories/:code/subcategories` | SKU 小類列表 |
| POST | `/sku/generate` | 產生 SKU |
| POST | `/sku/validate` | 驗證 SKU |
| POST | `/skus/preview` | 預覽 SKU |

---

## 11. 夥伴管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/partners` | 夥伴列表 |
| POST | `/partners` | 建立夥伴 |
| GET | `/partners/generate-code` | 產生夥伴代碼 |
| GET | `/partners/:id` | 夥伴詳情 |
| PUT | `/partners/:id` | 更新夥伴 |
| DELETE | `/partners/:id` | 刪除夥伴 |

---

## 12. 單據管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/documents` | 單據列表 |
| POST | `/documents` | 建立單據 |
| GET | `/documents/:id` | 單據詳情 |
| PUT | `/documents/:id` | 更新單據 |
| DELETE | `/documents/:id` | 刪除單據 |
| POST | `/documents/:id/submit` | 送審 |
| POST | `/documents/:id/approve` | 核准 |
| POST | `/documents/:id/cancel` | 取消 |

---

## 13. 庫存查詢 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/inventory/on-hand` | 現有庫存 |
| GET | `/inventory/ledger` | 庫存異動紀錄 |
| GET | `/inventory/low-stock` | 低庫存警報 |

---

## 14. 報表 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/reports/stock-on-hand` | 庫存報表 |
| GET | `/reports/stock-ledger` | 異動報表 |
| GET | `/reports/purchase-lines` | 採購明細 |
| GET | `/reports/sales-lines` | 銷貨明細 |
| GET | `/reports/cost-summary` | 成本分析 |
| GET | `/reports/blood-test-cost` | 血檢成本報表 |
| GET | `/reports/blood-test-analysis` | 血檢分析報表 |

---

## 15. 排程報表 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/scheduled-reports` | 排程報表列表 |
| POST | `/scheduled-reports` | 建立排程報表 |
| GET | `/scheduled-reports/:id` | 報表詳情 |
| PUT | `/scheduled-reports/:id` | 更新排程 |
| DELETE | `/scheduled-reports/:id` | 刪除排程 |
| GET | `/report-history` | 報表歷程 |
| GET | `/report-history/:id/download` | 下載報表 |

---

## 16. 計畫書 (AUP) API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/protocols` | 計畫列表 |
| POST | `/protocols` | 建立計畫 |
| GET | `/protocols/:id` | 計畫詳情 |
| PUT | `/protocols/:id` | 更新計畫 |
| POST | `/protocols/:id/submit` | 送審 |
| POST | `/protocols/:id/status` | 變更狀態 |
| GET | `/protocols/:id/versions` | 版本歷程 |
| GET | `/protocols/:id/activities` | 活動紀錄 |
| GET | `/protocols/:id/animal-stats` | 動物統計 |
| GET | `/protocols/:id/export-pdf` | 匯出 PDF |
| GET | `/protocols/:id/co-editors` | 共同編輯列表 |
| POST | `/protocols/:id/co-editors` | 新增共同編輯 |
| DELETE | `/protocols/:id/co-editors/:user_id` | 移除共同編輯 |
| GET | `/protocols/:id/status-history` | 狀態歷程 |
| GET | `/my-projects` | 我的計畫 |

---

## 17. 審查 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/reviews/assignments` | 審查指派列表 |
| POST | `/reviews/assignments` | 指派審查委員 |
| GET | `/reviews/comments` | 審查意見列表 |
| POST | `/reviews/comments` | 新增審查意見 |
| POST | `/reviews/comments/:id/resolve` | 解決意見 |
| POST | `/reviews/comments/reply` | 回覆意見 |
| POST | `/reviews/comments/draft` | 儲存草稿回覆 |
| GET | `/reviews/comments/:id/draft` | 取得草稿 |
| POST | `/reviews/comments/submit-draft` | 提交草稿 |
| POST | `/reviews/vet-form` | 獸醫審查表單 |

---

## 18. 變更申請 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/amendments` | 變更列表 |
| POST | `/amendments` | 建立變更 |
| GET | `/amendments/pending-count` | 待處理數量 |
| GET | `/amendments/:id` | 變更詳情 |
| PATCH | `/amendments/:id` | 更新變更 |
| POST | `/amendments/:id/submit` | 送審 |
| POST | `/amendments/:id/classify` | 分類 |
| POST | `/amendments/:id/start-review` | 開始審查 |
| POST | `/amendments/:id/decision` | 審查決定 |
| POST | `/amendments/:id/status` | 狀態變更 |
| GET | `/amendments/:id/versions` | 版本歷程 |
| GET | `/amendments/:id/history` | 異動歷程 |
| GET | `/amendments/:id/assignments` | 審查指派 |
| GET | `/protocols/:id/amendments` | 計畫下的變更 |

---

## 19. 動物來源 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animal-sources` | 來源列表 |
| POST | `/animal-sources` | 建立來源 |
| PUT | `/animal-sources/:id` | 更新來源 |
| DELETE | `/animal-sources/:id` | 刪除來源 |

---

## 20. 動物管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals` | 動物列表 |
| POST | `/animals` | 建立動物 |
| GET | `/animals/by-pen` | 依欄位分組 |
| POST | `/animals/batch/assign` | 批次分配 |
| POST | `/animals/batch/start-experiment` | 批次進入實驗 |
| GET | `/animals/vet-comments` | 獸醫待閱 |
| GET | `/animals/:id` | 動物詳情 |
| PUT | `/animals/:id` | 更新動物 |
| DELETE | `/animals/:id` | 刪除動物 |
| POST | `/animals/:id/vet-read` | 標記已讀 |
| GET | `/animals/:id/data-boundary` | 資料隔離邊界 |
| POST | `/animals/:id/field-corrections` | 建立欄位修正申請 |
| GET | `/animals/:id/field-corrections` | 查詢該動物修正申請列表 |
| GET | `/animal/animal-field-corrections/pending` | 待審修正申請列表（admin） |
| POST | `/animal/animal-field-corrections/:id/review` | 批准/拒絕修正申請（admin） |

---

## 21. 觀察紀錄 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals/:id/observations` | 觀察列表 |
| POST | `/animals/:id/observations` | 新增觀察 |
| GET | `/animals/:id/observations/with-recommendations` | 含獸醫建議 |
| POST | `/animals/:id/observations/copy` | 複製紀錄 |
| GET | `/observations/:id` | 觀察詳情 |
| PUT | `/observations/:id` | 更新觀察 |
| DELETE | `/observations/:id` | 刪除觀察 |
| POST | `/observations/:id/vet-read` | 獸醫標記已讀 |
| GET | `/observations/:id/versions` | 版本歷程 |

---

## 22. 手術紀錄 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals/:id/surgeries` | 手術列表 |
| POST | `/animals/:id/surgeries` | 新增手術 |
| GET | `/animals/:id/surgeries/with-recommendations` | 含獸醫建議 |
| POST | `/animals/:id/surgeries/copy` | 複製紀錄 |
| GET | `/surgeries/:id` | 手術詳情 |
| PUT | `/surgeries/:id` | 更新手術 |
| DELETE | `/surgeries/:id` | 刪除手術 |
| POST | `/surgeries/:id/vet-read` | 獸醫標記已讀 |
| GET | `/surgeries/:id/versions` | 版本歷程 |

---

## 23. 體重 / 疫苗 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals/:id/weights` | 體重紀錄 |
| POST | `/animals/:id/weights` | 新增體重 |
| PUT | `/weights/:id` | 更新體重 |
| DELETE | `/weights/:id` | 刪除體重 |
| GET | `/animals/:id/vaccinations` | 疫苗紀錄 |
| POST | `/animals/:id/vaccinations` | 新增疫苗 |
| PUT | `/vaccinations/:id` | 更新疫苗 |
| DELETE | `/vaccinations/:id` | 刪除疫苗 |

---

## 24. 犧牲 / 病理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals/:id/sacrifice` | 犧牲紀錄 |
| POST | `/animals/:id/sacrifice` | 新增/更新犧牲 |
| GET | `/animals/:id/pathology` | 病理報告 |
| POST | `/animals/:id/pathology` | 新增/更新病理 |

---

## 25. 猝死登記 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/animals/:id/sudden-death` | 登記猝死 |
| GET | `/animals/:id/sudden-death` | 查詢猝死紀錄 |

---

## 26. 動物轉讓 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/animals/:id/transfers` | 發起轉讓 |
| GET | `/animals/:id/transfers` | 轉讓紀錄列表 |
| GET | `/transfers/:id` | 轉讓詳情 |
| POST | `/transfers/:id/source-pi-confirm` | 來源 PI 確認 |
| POST | `/transfers/:id/vet-evaluate` | 獸醫評估 |
| POST | `/transfers/:id/target-pi-confirm` | 目標 PI 確認 |
| POST | `/transfers/:id/iacuc-approve` | IACUC 核准 |
| POST | `/transfers/:id/complete` | 執行完成 |

---

## 27. 血液檢查 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/animals/:id/blood-tests` | 血檢列表 |
| POST | `/animals/:id/blood-tests` | 新增血檢 |
| GET | `/blood-tests/:id` | 血檢詳情 |
| PUT | `/blood-tests/:id` | 更新血檢 |
| DELETE | `/blood-tests/:id` | 刪除血檢 |
| GET | `/blood-test-templates` | 模板列表（分頁）|
| GET | `/blood-test-templates/all` | 全部模板 |
| POST | `/blood-test-templates` | 建立模板 |
| PUT | `/blood-test-templates/:id` | 更新模板 |
| DELETE | `/blood-test-templates/:id` | 刪除模板 |
| GET | `/blood-test-panels` | 組合列表（分頁）|
| GET | `/blood-test-panels/all` | 全部組合 |
| POST | `/blood-test-panels` | 建立組合 |
| PUT | `/blood-test-panels/:id` | 更新組合 |
| DELETE | `/blood-test-panels/:id` | 刪除組合 |
| PUT | `/blood-test-panels/:id/items` | 更新組合項目 |

---

## 28. 獸醫建議 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/observations/:id/recommendations` | 觀察獸醫建議 |
| POST | `/observations/:id/recommendations` | 新增觀察建議 |
| POST | `/observations/:id/recommendations/with-attachments` | 建議含附件 |
| GET | `/surgeries/:id/recommendations` | 手術獸醫建議 |
| POST | `/surgeries/:id/recommendations` | 新增手術建議 |
| POST | `/surgeries/:id/recommendations/with-attachments` | 建議含附件 |

---

## 29. 動物匯入匯出 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/animals/:id/export` | 匯出個別醫療資料 |
| POST | `/projects/:iacuc_no/export` | 匯出計畫醫療資料 |
| GET | `/animals/import/batches` | 匯入批次列表 |
| GET | `/animals/import/template/basic` | 下載基本資料模板 |
| GET | `/animals/import/template/weight` | 下載體重模板 |
| POST | `/animals/import/basic` | 匯入基本資料 |
| POST | `/animals/import/weights` | 匯入體重資料 |

---

## 30. 電子簽章 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/signatures/sacrifice/:id` | 簽署犧牲紀錄（支援密碼/手寫）|
| GET | `/signatures/sacrifice/:id` | 取得犧牲簽章狀態 |
| POST | `/signatures/observation/:id` | 簽署觀察紀錄 |
| POST | `/signatures/euthanasia/:id` | 簽署安樂死紀錄 |
| GET | `/signatures/euthanasia/:id/status` | 安樂死簽章狀態 |
| POST | `/signatures/transfer/:id` | 簽署轉讓紀錄 |
| GET | `/signatures/transfer/:id/status` | 轉讓簽章狀態 |
| POST | `/signatures/protocol/:id` | 簽署計畫審查 |
| GET | `/signatures/protocol/:id/status` | 計畫簽章狀態 |
| GET | `/annotations/:record_type/:record_id` | 取得紀錄標註 |
| POST | `/annotations/:record_type/:record_id` | 新增紀錄標註 |

---

## 31. 安樂死管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/euthanasia/orders` | 建立安樂死申請 |
| GET | `/euthanasia/orders/pending` | 待核准列表 |
| GET | `/euthanasia/orders/:id` | 申請詳情 |
| POST | `/euthanasia/orders/:id/approve` | 核准申請 |
| POST | `/euthanasia/orders/:id/appeal` | 提出申訴 |
| POST | `/euthanasia/orders/:id/execute` | 執行安樂死 |
| POST | `/euthanasia/appeals/:id/decide` | 申訴裁決 |

---

## 32. 通知 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/notifications` | 通知列表 |
| GET | `/notifications/unread-count` | 未讀數量 |
| POST | `/notifications/read` | 標記已讀 |
| POST | `/notifications/read-all` | 全部已讀 |
| DELETE | `/notifications/:id` | 刪除通知 |
| GET | `/notifications/settings` | 通知設定 |
| PUT | `/notifications/settings` | 更新通知設定 |

---

## 33. 通知路由管理 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| GET | `/admin/notification-routing` | 路由列表 | admin |
| POST | `/admin/notification-routing` | 建立路由 | admin |
| PUT | `/admin/notification-routing/:id` | 更新路由 | admin |
| DELETE | `/admin/notification-routing/:id` | 刪除路由 | admin |

---

## 34. 警報 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/alerts/low-stock` | 低庫存警報 |
| GET | `/alerts/expiry` | 效期警報 |

---

## 35. 管理員觸發 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| POST | `/admin/trigger/low-stock-check` | 觸發庫存檢查 | admin |
| POST | `/admin/trigger/expiry-check` | 觸發效期檢查 | admin |
| POST | `/admin/trigger/notification-cleanup` | 觸發通知清理 | admin |

---

## 36. 安全稽核 API

| 方法 | 路徑 | 說明 | 權限 |
|------|------|------|------|
| GET | `/admin/audit/activities` | 活動紀錄列表 | admin |
| GET | `/admin/audit/activities/export` | 匯出活動紀錄 | admin |
| GET | `/admin/audit/activities/user/:user_id` | 使用者活動時間軸 | admin |
| GET | `/admin/audit/activities/entity/:entity_type/:entity_id` | 實體歷程 | admin |
| GET | `/admin/audit/logins` | 登入事件列表 | admin |
| GET | `/admin/audit/sessions` | 工作階段列表 | admin |
| POST | `/admin/audit/sessions/:id/logout` | 強制登出 | admin |
| GET | `/admin/audit/alerts` | 安全警報列表 | admin |
| POST | `/admin/audit/alerts/:id/resolve` | 解決警報 | admin |
| GET | `/admin/audit/dashboard` | 安全儀表板 | admin |
| GET | `/audit-logs` | 稽核日誌 | authenticated |

---

## 37. HR 出勤 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/attendance` | 出勤紀錄 |
| POST | `/hr/attendance/clock-in` | 上班打卡 |
| POST | `/hr/attendance/clock-out` | 下班打卡 |
| GET | `/hr/attendance/stats` | 出勤統計 |
| PUT | `/hr/attendance/:id` | 修正打卡 |

---

## 38. HR 加班 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/overtime` | 加班列表 |
| POST | `/hr/overtime` | 新增加班 |
| GET | `/hr/overtime/:id` | 加班詳情 |
| PUT | `/hr/overtime/:id` | 更新加班 |
| DELETE | `/hr/overtime/:id` | 刪除加班 |
| POST | `/hr/overtime/:id/submit` | 送審 |
| POST | `/hr/overtime/:id/approve` | 核准 |
| POST | `/hr/overtime/:id/reject` | 駁回 |

---

## 39. HR 請假 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/leaves` | 請假列表 |
| POST | `/hr/leaves` | 新增請假 |
| GET | `/hr/leaves/:id` | 請假詳情 |
| PUT | `/hr/leaves/:id` | 更新請假 |
| DELETE | `/hr/leaves/:id` | 刪除請假 |
| POST | `/hr/leaves/:id/submit` | 送審 |
| POST | `/hr/leaves/:id/approve` | 核准 |
| POST | `/hr/leaves/:id/reject` | 駁回 |
| POST | `/hr/leaves/:id/cancel` | 撤銷 |
| POST | `/hr/leaves/attachments` | 上傳附件 |

---

## 40. HR 假期餘額 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/balances/annual` | 特休餘額 |
| GET | `/hr/balances/comp-time` | 補休餘額 |
| GET | `/hr/balances/summary` | 餘額摘要 |
| POST | `/hr/balances/annual-entitlements` | 新增特休配額 |
| POST | `/hr/balances/:id/adjust` | 調整餘額 |
| GET | `/hr/balances/expired-compensation` | 過期補償 |

---

## 41. HR 儀表板 / 人員 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/dashboard/calendar` | 行事曆 |
| GET | `/hr/staff` | 代理人選單 |
| GET | `/hr/internal-users` | 內部使用者 |

---

## 42. HR 行事曆同步 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/calendar/status` | 同步狀態 |
| GET | `/hr/calendar/config` | 取得設定 |
| PUT | `/hr/calendar/config` | 更新設定 |
| POST | `/hr/calendar/connect` | 連接日曆 |
| POST | `/hr/calendar/disconnect` | 中斷連接 |
| POST | `/hr/calendar/sync` | 觸發同步 |
| GET | `/hr/calendar/history` | 同步歷程 |
| GET | `/hr/calendar/pending` | 待同步項目 |
| GET | `/hr/calendar/conflicts` | 衝突列表 |
| GET | `/hr/calendar/conflicts/:id` | 衝突詳情 |
| POST | `/hr/calendar/conflicts/:id/resolve` | 解決衝突 |
| GET | `/hr/calendar/events` | 日曆事件 |

---

## 43. 設施管理 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET/POST | `/facilities/species` | CRUD 物種 |
| GET/PUT/DELETE | `/facilities/species/:id` | 物種操作 |
| GET/POST | `/facilities` | CRUD 設施 |
| GET/PUT/DELETE | `/facilities/:id` | 設施操作 |
| GET/POST | `/facilities/buildings` | CRUD 棟舍 |
| GET/PUT/DELETE | `/facilities/buildings/:id` | 棟舍操作 |
| GET/POST | `/facilities/zones` | CRUD 區域 |
| GET/PUT/DELETE | `/facilities/zones/:id` | 區域操作 |
| GET/POST | `/facilities/pens` | CRUD 欄位 |
| GET/PUT/DELETE | `/facilities/pens/:id` | 欄位操作 |
| GET/POST | `/facilities/departments` | CRUD 部門 |
| GET/PUT/DELETE | `/facilities/departments/:id` | 部門操作 |

---

## 44. 檔案上傳 API

| 方法 | 路徑 | 說明 |
|------|------|------|
| POST | `/protocols/:id/attachments` | 計畫書附件 |
| POST | `/animals/:id/photos` | 動物照片 |
| POST | `/animals/:id/pathology/attachments` | 病理附件 |
| POST | `/animals/:id/sacrifice/photos` | 犧牲照片 |
| POST | `/vet-recommendations/:record_type/:record_id/attachments` | 獸醫建議附件 |
| GET | `/attachments` | 附件列表 |
| GET | `/attachments/:id` | 下載附件 |
| DELETE | `/attachments/:id` | 刪除附件 |

---

## 端點統計

| 分類 | 端點數 |
|------|--------|
| 認證/個人 | 12 |
| 使用者/角色/權限 | 13 |
| ERP (產品/SKU/倉庫/儲位/夥伴) | 30 |
| 單據/庫存/報表 | 23 |
| AUP (計畫/審查/變更) | 38 |
| 動物管理 (動物/醫療) | 50 |
| 猝死/轉讓 | 10 |
| 安樂死/簽章 | 18 |
| 通知/警報/排程/路由管理 | 21 |
| 稽核/管理 | 14 |
| HR (出勤/請假/加班/日曆) | 34 |
| 設施管理 | 22 |
| 檔案上傳 | 8 |
| **合計** | **~293** |

---

*下一章：[權限與 RBAC](./06_PERMISSIONS_RBAC.md)*
