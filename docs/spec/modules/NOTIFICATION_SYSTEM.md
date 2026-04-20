# iPig 通知系統規格書

> **版本**：7.0  
> **最後更新**：2026-03-01

---

## 1. 概述

本文件定義系統內所有自動化通知的類型、觸發條件、收件者與內容格式。

**新增功能**：通知路由可配置化（`notification_routing` 表），管理員可動態調整每種事件的收件者。

---

## 2. 通知類型總覽

| 通知類型 | 觸發條件 | 收件者 | 通知方式 |
|---------|---------|-------|---------| 
| 帳號開通 | 管理員建立帳號 | 新使用者 | Email |
| 密碼重設 | 申請或管理員重設 | 使用者 | Email |
| 密碼變更成功 | 使用者變更密碼 | 使用者 | Email |
| 計畫提交 | PI 提交計畫 | IACUC_STAFF | Email + 站內 |
| 計畫狀態變更 | 狀態機轉換 | PI、相關人員 | Email + 站內 |
| 審查指派 | 指派審查人員 | REVIEWER/VET | Email + 站內 |
| 審查意見 | 審查人員新增意見 | PI | 站內 |
| 獸醫師建議 | VET 新增建議 | EXPERIMENT_STAFF | Email + 站內 |
| 低庫存提醒 | 庫存 ≤ safety_stock | WAREHOUSE_MANAGER | Email |
| 效期提醒 | 效期 ≤ 30 天 | WAREHOUSE_MANAGER | Email |
| 效期緊急提醒 | 效期 ≤ 7 天 | WM + ADMIN | Email |

---

## 3. 通知路由可配置化

### 3.1 notification_routing 表

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| event_type | VARCHAR(50) | 事件類型 |
| description | VARCHAR(200) | 說明 |
| target_roles | TEXT[] | 目標角色（陣列） |
| channels | TEXT[] | 通知管道（email/in_app） |
| is_active | BOOLEAN | 是否啟用 |

### 3.2 動態查詢

```sql
-- 根據事件類型取得收件者角色與管道
SELECT target_roles, channels
FROM notification_routing
WHERE event_type = $1 AND is_active = true;
```

### 3.3 管理 API

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/admin/notification-routing` | 路由列表 |
| PUT | `/admin/notification-routing/:id` | 更新路由 |
| POST | `/admin/notification-routing` | 建立路由 |
| DELETE | `/admin/notification-routing/:id` | 刪除路由 |

> 前端管理介面：`NotificationRoutingSection.tsx`（Settings 頁面內）

### 3.4 預設種子資料

系統內建 21 筆預設通知路由規則，涵蓋：
- 帳號相關（3 筆）
- AUP 審查相關（4 筆）
- 動物管理相關（3 筆）
- ERP 庫存相關（3 筆）
- HR 相關（3 筆）
- 安全警報相關（3 筆）
- 變更申請相關（2 筆）

---

## 4. 通知詳細規格

### 4.1 帳號相關通知

#### 帳號開通通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | SYSTEM_ADMIN 建立新帳號 |
| 收件者 | 新使用者 Email |
| 主旨 | 豬博士 iPig 系統帳號開通通知 |

**Email 內容：**
```
您好 {display_name}，

您的豬博士 iPig 系統帳號已開通：

【登入資訊】
帳號：{email}
初始密碼：{password}
登入網址：{login_url}

請於首次登入後立即變更密碼。
如有任何問題，請聯繫工作人員（電話：037-433789）。

豬博士動物科技有限公司
```

#### 密碼重設通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 使用者申請重設 或 管理員重設 |
| 收件者 | 使用者 Email |
| 主旨 | 豬博士 iPig 密碼重設通知 |

#### 密碼變更成功通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 使用者成功變更密碼 |
| 收件者 | 使用者 Email |
| 主旨 | 豬博士 iPig 密碼變更通知 |

---

### 4.2 計畫審查通知

#### 計畫提交通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | PI 提交計畫（DRAFT → SUBMITTED） |
| 收件者 | 所有 IACUC_STAFF（依通知路由） |
| 主旨 | [iPig] 新計畫提交 - {protocol_no} |

#### 計畫狀態變更通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 計畫狀態轉換 |
| 收件者 | 依狀態決定（見下表） |
| 主旨 | [iPig] 計畫狀態更新 - {protocol_no} |

**收件者對照表：**

| 新狀態 | 收件者 |
|-------|-------|
| PRE_REVIEW | IACUC_STAFF |
| VET_REVIEW | VET |
| UNDER_REVIEW | PI（通知審查中） |
| REVISION_REQUIRED | PI |
| APPROVED | PI、CLIENT（若有） |
| APPROVED_WITH_CONDITIONS | PI、CLIENT（若有） |
| REJECTED | PI |
| SUSPENDED | PI、CLIENT（若有） |
| CLOSED | PI、CLIENT（若有） |

#### 審查指派通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | IACUC_STAFF/CHAIR 指派審查人員 |
| 收件者 | 被指派的 REVIEWER 或 VET |
| 主旨 | [iPig] 審查指派 - {protocol_no} |

---

### 4.3 實驗動物管理通知

#### 獸醫師建議通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | VET 對觀察/手術紀錄新增建議 |
| 收件者 | 該計畫的 EXPERIMENT_STAFF |
| 主旨 | [iPig] 獸醫師建議 - 耳號 {ear_tag} |

**Email 內容：**
```
獸醫師已對以下動物新增照護建議，請查閱並執行。

【動物資訊】
耳號：{ear_tag}
IACUC No.：{iacuc_no}
紀錄類型：{record_type}（觀察試驗紀錄/手術紀錄）
建議內容：{recommendation_content}
建議時間：{created_at}

請登入系統查看：{record_url}

豬博士動物科技有限公司
```

---

### 4.4 進銷存提醒通知

#### 低庫存提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 庫存現況 ≤ safety_stock |
| 檢查時機 | 每日排程 08:00 |
| 收件者 | 所有 WAREHOUSE_MANAGER |
| 主旨 | [iPig] 低庫存提醒 - {date} |

#### 效期提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 效期 ≤ 30 天 |
| 檢查時機 | 每日排程 08:00 |
| 收件者 | 所有 WAREHOUSE_MANAGER |
| 主旨 | [iPig] 效期提醒 - {date} |

#### 效期緊急提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 效期 ≤ 7 天（且尚未處理） |
| 收件者 | WAREHOUSE_MANAGER + SYSTEM_ADMIN |
| 主旨 | [緊急][iPig] 效期緊急提醒 - {date} |

---

## 5. 站內通知規格

### 5.1 通知資料模型

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | FK → users.id |
| type | VARCHAR(50) | 通知類型 |
| title | VARCHAR(200) | 通知標題 |
| content | TEXT | 通知內容 |
| link | VARCHAR(500) | 點擊跳轉連結 |
| is_read | BOOLEAN | 是否已讀 |
| read_at | TIMESTAMPTZ | 已讀時間 |
| created_at | TIMESTAMPTZ | 建立時間 |

### 5.2 站內通知 API

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/notifications` | 通知列表（分頁） |
| GET | `/notifications/unread-count` | 未讀數量 |
| POST | `/notifications/:id/read` | 標記單筆已讀 |
| POST | `/notifications/read-all` | 標記全部已讀 |
| GET | `/notifications/settings` | 通知設定 |
| PUT | `/notifications/settings` | 更新通知設定 |

### 5.3 前端顯示

- **Header 通知圖示**：顯示未讀數量 Badge
- **通知下拉選單**：顯示最新通知，含點擊導航
- **通知設定**：SettingsPage 整合

---

## 6. 排程任務

| 任務 | 執行時間 | 說明 |
|-----|---------|------|
| 低庫存檢查 | 每日 08:00 | 檢查並發送低庫存提醒 |
| 效期檢查 | 每日 08:00 | 檢查並發送效期提醒 |
| 清理過期通知 | 每週日 03:00 | 刪除 90 天前的已讀站內通知 |
| 手動觸發 | Admin API | 管理員可手動觸發檢查 |

---

## 7. Email 發送設定

### 7.1 SMTP 設定

使用 Gmail SMTP 整合（`lettre` 套件），支援 HTML + 純文字格式。

### 7.2 發送節流

| 限制 | 數值 | 說明 |
|-----|------|------|
| 單一使用者 | 10 封/小時 | 防止重複觸發 |
| 全系統 | 500 封/小時 | Gmail 限制 |
| 密碼重設 | 3 次/小時 | 防濫用 |

### 7.3 重試機制

- 發送失敗自動重試 3 次
- 重試間隔：1 分鐘、5 分鐘、30 分鐘

---

## 8. 通知偏好設定

使用者可自訂通知偏好（SettingsPage）：

| 設定項 | 選項 |
|-------|------|
| Email 通知 | 全部開啟 / 僅重要 / 關閉 |
| 站內通知 | 全部開啟 / 關閉 |
| 低庫存提醒 | 開啟 / 關閉（僅 WAREHOUSE_MANAGER） |
| 效期提醒 | 開啟 / 關閉（僅 WAREHOUSE_MANAGER） |

---

## 9. 相關文件

- [AUP 系統](./AUP_SYSTEM.md) - 計畫審查通知
- [ERP 系統](./ERP_SYSTEM.md) - 庫存提醒通知
- [動物管理](./ANIMAL_MANAGEMENT.md) - 獸醫師建議通知
