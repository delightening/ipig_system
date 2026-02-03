# 豬博士 iPig 通知系統規格書

## 1. 概述

本文件定義系統內所有自動化通知的類型、觸發條件、收件者與內容格式。

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
| 效期緊急提醒 | 效期 ≤ 7 天 | WAREHOUSE_MANAGER | Email |

---

## 3. 通知詳細規格

### 3.1 帳號相關通知

#### 3.1.1 帳號開通通知

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

---

#### 3.1.2 密碼重設通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 使用者申請重設 或 管理員重設 |
| 收件者 | 使用者 Email |
| 主旨 | 豬博士 iPig 密碼重設通知 |

**Email 內容（使用者申請）：**
```
您好 {display_name}，

您已申請重設豬博士 iPig 系統密碼。

請點擊以下連結重設密碼（連結有效期 24 小時）：
{reset_link}

若您未申請此操作，請忽略此信件。

豬博士動物科技有限公司
```

**Email 內容（管理員重設）：**
```
您好 {display_name}，

您的豬博士 iPig 系統密碼已由管理員重設：

【新密碼】
{new_password}

請於登入後立即變更密碼。

豬博士動物科技有限公司
```

---

#### 3.1.3 密碼變更成功通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 使用者成功變更密碼 |
| 收件者 | 使用者 Email |
| 主旨 | 豬博士 iPig 密碼變更通知 |

**Email 內容：**
```
您好 {display_name}，

您的豬博士 iPig 系統密碼已於 {timestamp} 變更成功。

若非本人操作，請立即聯繫工作人員（電話：037-433789）。

豬博士動物科技有限公司
```

---

### 3.2 計畫審查通知

#### 3.2.1 計畫提交通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | PI 提交計畫（DRAFT → SUBMITTED） |
| 收件者 | 所有 IACUC_STAFF |
| 主旨 | [iPig] 新計畫提交 - {protocol_no} |

**Email 內容：**
```
新計畫已提交，請進行行政預審。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{title}
計畫主持人：{pi_name}
提交時間：{submitted_at}

請登入系統處理：{protocol_url}

豬博士動物科技有限公司
```

---

#### 3.2.2 計畫狀態變更通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 計畫狀態轉換 |
| 收件者 | 依狀態決定（見下表） |
| 主旨 | [iPig] 計畫狀態更新 - {protocol_no} |

**收件者對照表：**

| 新狀態 | 收件者 |
|-------|-------|
| PRE_REVIEW | IACUC_STAFF |
| UNDER_REVIEW | PI（通知審查中） |
| REVISION_REQUIRED | PI |
| APPROVED | PI、CLIENT（若有） |
| APPROVED_WITH_CONDITIONS | PI、CLIENT（若有） |
| REJECTED | PI |
| SUSPENDED | PI、CLIENT（若有） |
| CLOSED | PI、CLIENT（若有） |

**Email 內容：**
```
您的計畫狀態已更新。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{title}
新狀態：{new_status}
變更時間：{changed_at}
變更原因：{reason}

請登入系統查看：{protocol_url}

豬博士動物科技有限公司
```

---

#### 3.2.3 審查指派通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | IACUC_STAFF/CHAIR 指派審查人員 |
| 收件者 | 被指派的 REVIEWER 或 VET |
| 主旨 | [iPig] 審查指派 - {protocol_no} |

**Email 內容：**
```
您已被指派審查以下計畫，請於期限內完成審查。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{title}
計畫主持人：{pi_name}
審查期限：{due_date}

請登入系統審查：{protocol_url}

豬博士動物科技有限公司
```

---

### 3.3 實驗動物管理通知

#### 3.3.1 獸醫師建議通知

| 項目 | 內容 |
|-----|------|
| 觸發條件 | VET 對觀察/手術紀錄新增建議 |
| 收件者 | 該計畫的 EXPERIMENT_STAFF |
| 主旨 | [iPig] 獸醫師建議 - 耳號 {ear_tag} |

**Email 內容：**
```
獸醫師已對以下豬隻新增照護建議，請查閱並執行。

【豬隻資訊】
耳號：{ear_tag}
IACUC NO.：{iacuc_no}
紀錄類型：{record_type}（觀察試驗紀錄/手術紀錄）
建議內容：{recommendation_content}
建議時間：{created_at}

請登入系統查看：{record_url}

豬博士動物科技有限公司
```

**站內通知：**
- 顯示於 Dashboard 通知區塊
- 點擊可直接跳轉至該紀錄

---

### 3.4 進銷存提醒通知

#### 3.4.1 低庫存提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 庫存現況 ≤ safety_stock |
| 檢查時機 | 每日排程（建議 08:00） |
| 收件者 | 所有 WAREHOUSE_MANAGER |
| 主旨 | [iPig] 低庫存提醒 - {date} |

**Email 內容：**
```
以下品項庫存已低於安全庫存，請安排補貨。

【低庫存品項】
{table: SKU | 品名 | 倉庫 | 現有量 | 安全庫存}

請登入系統處理：{inventory_url}

豬博士動物科技有限公司
```

---

#### 3.4.2 效期提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 效期 ≤ 30 天 |
| 檢查時機 | 每日排程（建議 08:00） |
| 收件者 | 所有 WAREHOUSE_MANAGER |
| 主旨 | [iPig] 效期提醒 - {date} |

**Email 內容：**
```
以下品項即將到期，請注意處理。

【即將到期品項（30 天內）】
{table: SKU | 品名 | 批號 | 效期 | 剩餘天數 | 現有量}

【已過期品項】
{table: SKU | 品名 | 批號 | 效期 | 現有量}

請登入系統處理：{inventory_url}

豬博士動物科技有限公司
```

---

#### 3.4.3 效期緊急提醒

| 項目 | 內容 |
|-----|------|
| 觸發條件 | 效期 ≤ 7 天（且尚未處理） |
| 檢查時機 | 每日排程（建議 08:00） |
| 收件者 | 所有 WAREHOUSE_MANAGER + SYSTEM_ADMIN |
| 主旨 | [緊急][iPig] 效期緊急提醒 - {date} |

---

## 4. 站內通知規格

### 4.1 通知資料模型

#### notifications 表

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | FK → users.id |
| type | VARCHAR(50) | 通知類型 |
| title | VARCHAR(200) | 通知標題 |
| content | TEXT | 通知內容 |
| link | VARCHAR(500) | 點擊跳轉連結 |
| is_read | BOOLEAN | 是否已讀（預設 false） |
| read_at | TIMESTAMP | 已讀時間 |
| created_at | TIMESTAMP | 建立時間 |

### 4.2 站內通知 API

| 端點 | 方法 | 說明 |
|-----|------|------|
| `/notifications` | GET | 取得通知列表（支援分頁、篩選已讀/未讀） |
| `/notifications/unread-count` | GET | 取得未讀數量 |
| `/notifications/{id}/read` | POST | 標記單筆已讀 |
| `/notifications/read-all` | POST | 標記全部已讀 |

### 4.3 前端顯示

- **Header 通知圖示**：顯示未讀數量 Badge
- **通知下拉選單**：顯示最新 5 筆，可展開查看全部
- **Dashboard 通知區塊**：依重要程度顯示

---

## 5. 排程設定

### 5.1 排程任務

| 任務 | 執行時間 | 說明 |
|-----|---------|------|
| 低庫存檢查 | 每日 08:00 | 檢查並發送低庫存提醒 |
| 效期檢查 | 每日 08:00 | 檢查並發送效期提醒 |
| 清理過期通知 | 每週日 03:00 | 刪除 90 天前的已讀站內通知 |

### 5.2 排程實作（Rust 範例）

```rust
use tokio_cron_scheduler::{Job, JobScheduler};

async fn setup_scheduler() -> Result<(), Box<dyn std::error::Error>> {
    let sched = JobScheduler::new().await?;

    // 每日 08:00 執行庫存檢查
    sched.add(Job::new_async("0 0 8 * * *", |_uuid, _l| {
        Box::pin(async {
            check_low_stock().await;
            check_expiry().await;
        })
    })?).await?;

    sched.start().await?;
    Ok(())
}
```

---

## 6. Email 發送設定

> 詳細 SMTP 設定請參考 `role.md` 第 1.3 節。

### 6.1 發送節流

| 限制 | 數值 | 說明 |
|-----|------|------|
| 單一使用者 | 10 封/小時 | 防止重複觸發 |
| 全系統 | 500 封/小時 | Gmail 限制 |
| 密碼重設 | 3 次/小時 | 防濫用 |

### 6.2 重試機制

- 發送失敗自動重試 3 次
- 重試間隔：1 分鐘、5 分鐘、30 分鐘
- 超過重試次數記錄至 `notification_logs` 表

---

## 7. 通知偏好設定（可選，v0.2）

使用者可自訂通知偏好：

| 設定項 | 選項 |
|-------|------|
| Email 通知 | 全部開啟 / 僅重要 / 關閉 |
| 站內通知 | 全部開啟 / 關閉 |
| 低庫存提醒 | 開啟 / 關閉（僅 WAREHOUSE_MANAGER） |
| 效期提醒 | 開啟 / 關閉（僅 WAREHOUSE_MANAGER） |

---

## 8. 相關文件

- `role.md` - SMTP 設定與 Email 範本
- `spec.md` - 系統總覽
- `ERPSpec.md` - 進銷存系統規格

