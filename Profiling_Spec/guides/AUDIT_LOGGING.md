# 稽核與日誌規格

> **版本**：1.0  
> **最後更新**：2026-01-17  
> **對象**：資安團隊、開發人員、法規遵循人員

---

## 1. 目的

稽核日誌系統提供完整的使用者活動追蹤，用於：

- **GLP 法規遵循** - 誰、在何時、對哪個實體做了什麼
- **安全監控** - 偵測可疑活動
- **管理洞察** - 了解系統使用模式
- **事件調查** - 追溯安全事件期間的操作

---

## 2. 稽核需求

### 2.1 GLP 法規遵循

作為實驗動物研究系統，iPig 必須維護符合優良實驗室規範（GLP）要求的稽核軌跡：

| 需求 | 實作方式 |
|------|----------|
| **不可變更** | 稽核日誌僅能新增，不可更新或刪除 |
| **完整性** | 所有資料異動皆需記錄 |
| **歸屬** | 每個操作都連結至使用者 |
| **時間戳記** | 伺服器端時間戳記，不可修改 |
| **保留期限** | 總計 7 年（熱儲存 2 年，冷儲存 5 年）|

### 2.2 資料主體權利

| 考量 | 政策 |
|------|------|
| 存取請求 | 使用者可請求存取自己的活動資料 |
| 刪除請求 | 稽核資料不受刪除權規範 |
| 匿名化 | 不適用 - 需要保留歸屬資訊 |

---

## 3. 記錄項目

### 3.1 必須記錄（高優先）

| 類別 | 事件 | 範例 |
|------|------|------|
| **認證** | 登入成功/失敗、登出、密碼變更 | 使用者 alice@example.com 登入 |
| **資料異動** | 所有實體的新增、更新、刪除 | 建立豬隻耳號=A001 |
| **核准** | 需要授權的狀態變更 | 計畫書 P-2026-001 已核准 |
| **匯出** | 資料匯出、下載 | 匯出豬隻醫療紀錄 |
| **管理操作** | 使用者管理、角色變更 | 將 VET 角色指派給使用者 bob |
| **敏感存取** | 敏感紀錄的檢視 | 檢視豬隻 A001 的病理報告 |

### 3.2 應該記錄（中優先）

| 類別 | 事件 |
|------|------|
| **頁面瀏覽** | 敏感頁面的存取（彙總）|
| **搜尋查詢** | 敏感資料的搜尋 |
| **批次操作** | 批次更新、匯入 |

### 3.3 不應記錄（效能/隱私）

| 避免 | 原因 |
|------|------|
| 鍵盤輸入 | 隱私、效能 |
| 滑鼠移動、捲動 | 無稽核價值 |
| 草稿自動儲存 | 太過嘈雜 |
| 健康檢查請求 | 系統雜訊 |
| 靜態資源請求 | 無稽核價值 |

---

## 4. 資料庫綱要

### 4.1 user_activity_logs（分區表）

主要稽核表，按季度分區以提升效能。

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| actor_user_id | UUID | 執行操作的人 |
| actor_email | VARCHAR(255) | 操作時的信箱快照 |
| actor_display_name | VARCHAR(100) | 名稱快照 |
| actor_roles | JSONB | 操作時的角色代碼 |
| session_id | UUID | 連結至使用者工作階段 |
| event_category | VARCHAR(50) | auth, data, admin, export, navigation |
| event_type | VARCHAR(100) | 具體事件（pig.create, protocol.approve）|
| event_severity | VARCHAR(20) | info, warning, critical |
| entity_type | VARCHAR(50) | 受影響實體的類型 |
| entity_id | UUID | 受影響實體的 ID |
| entity_display_name | VARCHAR(255) | 人類可讀識別碼 |
| before_data | JSONB | 變更前狀態 |
| after_data | JSONB | 變更後狀態 |
| changed_fields | TEXT[] | 修改欄位清單 |
| ip_address | INET | 客戶端 IP |
| user_agent | TEXT | 瀏覽器/客戶端資訊 |
| request_path | VARCHAR(500) | 呼叫的 API 端點 |
| request_method | VARCHAR(10) | HTTP 方法 |
| response_status | INTEGER | HTTP 回應碼 |
| is_suspicious | BOOLEAN | 異常偵測標記 |
| suspicious_reason | TEXT | 標記原因 |
| created_at | TIMESTAMPTZ | 不可變時間戳記 |
| partition_date | DATE | 分區鍵 |

**分區**：按季度（2026_q1、2026_q2 等）

### 4.2 login_events

專用認證事件表。

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 嘗試登入的使用者（若找不到則為 null）|
| email | VARCHAR(255) | 登入嘗試使用的信箱 |
| event_type | VARCHAR(20) | login_success, login_failure, logout |
| ip_address | INET | 客戶端 IP |
| user_agent | TEXT | 瀏覽器資訊 |
| device_type | VARCHAR(50) | desktop, mobile, tablet |
| browser | VARCHAR(50) | Chrome, Firefox 等 |
| os | VARCHAR(50) | Windows, macOS 等 |
| is_unusual_time | BOOLEAN | 非上午 7 時至晚上 10 時 |
| is_unusual_location | BOOLEAN | 與常用 IP 不同 |
| is_new_device | BOOLEAN | 首次見到的裝置 |
| device_fingerprint | VARCHAR(255) | 裝置識別碼 |
| failure_reason | VARCHAR(100) | 若失敗：invalid_password 等 |
| created_at | TIMESTAMPTZ | 時間戳記 |

### 4.3 user_sessions

追蹤活動工作階段以支援強制登出功能。

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 工作階段擁有者 |
| started_at | TIMESTAMPTZ | 工作階段開始 |
| ended_at | TIMESTAMPTZ | 工作階段結束時間 |
| last_activity_at | TIMESTAMPTZ | 最後 API 請求 |
| refresh_token_id | UUID | 連結至刷新令牌 |
| ip_address | INET | 工作階段 IP |
| user_agent | TEXT | 瀏覽器資訊 |
| page_view_count | INTEGER | 瀏覽頁數 |
| action_count | INTEGER | 執行的異動數 |
| is_active | BOOLEAN | 目前是否活動中 |
| ended_reason | VARCHAR(50) | logout, expired, forced_logout |

### 4.4 security_alerts

異常偵測警報。

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| alert_type | VARCHAR(50) | brute_force, unusual_login 等 |
| severity | VARCHAR(20) | info, warning, critical |
| title | VARCHAR(255) | 警報標題 |
| description | TEXT | 詳細說明 |
| user_id | UUID | 相關使用者 |
| context_data | JSONB | 額外上下文 |
| status | VARCHAR(20) | open, acknowledged, resolved |
| resolved_by | UUID | 解決者 |
| resolved_at | TIMESTAMPTZ | 解決時間 |
| resolution_notes | TEXT | 解決詳情 |

---

## 5. 事件類型

### 5.1 認證事件

| 事件類型 | 嚴重度 | 觸發條件 |
|----------|--------|----------|
| auth.login_success | info | 登入成功 |
| auth.login_failure | warning | 登入嘗試失敗 |
| auth.logout | info | 使用者登出 |
| auth.token_refresh | info | Token 刷新 |
| auth.password_change | info | 使用者變更密碼 |
| auth.password_reset | info | 透過信件重設密碼 |
| auth.session_expired | info | 工作階段逾時 |

### 5.2 資料事件

| 事件類型 | 嚴重度 | 觸發條件 |
|----------|--------|----------|
| {entity}.create | info | 實體建立 |
| {entity}.update | info | 實體更新 |
| {entity}.delete | warning | 實體刪除 |
| {entity}.status_change | info | 狀態欄位變更 |

其中 `{entity}` 為：pig, protocol, document, user, role 等。

### 5.3 管理事件

| 事件類型 | 嚴重度 | 觸發條件 |
|----------|--------|----------|
| admin.user.create | info | 建立新使用者 |
| admin.user.role_change | warning | 使用者角色修改 |
| admin.user.deactivate | warning | 使用者停用 |
| admin.role.permission_change | warning | 角色權限修改 |
| admin.session.force_logout | warning | 管理員強制登出 |

### 5.4 匯出事件

| 事件類型 | 嚴重度 | 觸發條件 |
|----------|--------|----------|
| export.pig_medical | info | 醫療紀錄匯出 |
| export.audit_logs | info | 稽核日誌匯出 |
| export.report | info | 報表產生 |

---

## 6. 異常偵測

### 6.1 偵測規則

| 規則 | 門檻 | 嚴重度 |
|------|------|--------|
| 暴力攻擊 | 15 分鐘內 5 次登入失敗 | critical |
| 異常時間 | 上午 7 時前或晚上 10 時後登入 | warning |
| 異常地點 | 新 IP 範圍 | warning |
| 高流量 | 1 小時內超過 100 次異動 | warning |
| 非上班時間資料存取 | 上班時間外存取敏感資料 | warning |

### 6.2 警報流程

```
偵測 → 建立警報 → 發送通知 → 管理員審查
                                    │
             ┌──────────────────────┼──────────────────────┐
             ▼                      ▼                      ▼
        確認收到              調查                    忽略
      （標記為已閱讀）    （連結至工作階段）         （誤判）
             │                      │                      │
             └──────────────────────┼──────────────────────┘
                                    ▼
                                 解決
                            （新增解決備註）
```

---

## 7. 保留政策

| 儲存層級 | 期限 | 位置 | 存取方式 |
|----------|------|------|----------|
| 熱儲存 | 2 年 | 主資料庫 | 完整查詢 |
| 歸檔 | 5 年 | 冷儲存 | 依請求 |
| **總計** | **7 年** | | |

### 7.1 歸檔程序

1. 超過 2 年的季度分區進行歸檔
2. 歸檔分區壓縮並移至冷儲存
3. 保留索引中繼資料供搜尋
4. 可於 24 小時內還原

---

## 8. API 端點

所有端點需要 `admin.audit.*` 權限。

### 8.1 活動日誌

```
GET /api/admin/audit/activities
```

查詢參數：
- `user_id` - 依操作者篩選
- `entity_type` - 依實體類型篩選
- `entity_id` - 依特定實體篩選
- `event_category` - auth, data, admin, export
- `event_type` - 具體事件類型
- `from`, `to` - 日期範圍
- `is_suspicious` - 僅可疑活動
- `page`, `limit` - 分頁

### 8.2 登入事件

```
GET /api/admin/audit/logins
```

查詢參數：
- `user_id` - 依使用者篩選
- `event_type` - login_success, login_failure, logout
- `is_unusual` - 僅異常登入
- `from`, `to` - 日期範圍

### 8.3 工作階段

```
GET /api/admin/audit/sessions
POST /api/admin/audit/sessions/:id/force-logout
```

### 8.4 使用者時間軸

```
GET /api/admin/audit/users/:id/timeline
GET /api/admin/audit/users/:id/summary
```

### 8.5 實體歷程

```
GET /api/admin/audit/entities/:type/:id/history
```

### 8.6 安全警報

```
GET /api/admin/audit/security-alerts
POST /api/admin/audit/security-alerts/:id/resolve
```

### 8.7 匯出

```
POST /api/admin/audit/activities/export
```

---

## 9. UI 元件

### 9.1 管理員選單結構

```
⚙️ 系統管理
  └── 安全審計
        ├── 活動日誌
        ├── 登入紀錄
        ├── 使用者分析
        └── 安全警報
```

### 9.2 活動日誌檢視

- 可篩選資料表格
- 時間軸視覺化選項
- 匯出為 CSV/JSON
- 連結至實體詳情頁

### 9.3 使用者檔案檢視

- 使用者資訊卡片
- 活動時間軸
- 登入歷程
- 工作階段列表
- 角色變更歷程

### 9.4 實體歷程檢視

- 可從實體詳情頁存取
- 顯示所有時間的變更
- 變更差異檢視
- 連結至操作者

---

## 10. 權限

| 權限代碼 | 說明 |
|----------|------|
| admin.audit.view | 檢視稽核日誌 |
| admin.audit.view.activities | 檢視活動紀錄 |
| admin.audit.view.logins | 檢視登入紀錄 |
| admin.audit.view.sessions | 檢視工作階段 |
| admin.audit.export | 匯出稽核資料 |
| admin.audit.force_logout | 強制使用者登出 |
| admin.audit.alerts.view | 檢視安全警報 |
| admin.audit.alerts.resolve | 解決警報 |
| admin.audit.dashboard | 檢視稽核儀表板 |

預設指派：SYSTEM_ADMIN、PROGRAM_ADMIN

---

## 11. 實作備註

### 11.1 中間件整合

活動日誌作為 Axum 中間件實作：

```rust
.route_layer(middleware::from_fn_with_state(
    state.clone(), 
    activity_logging_middleware
))
```

### 11.2 非同步日誌

日誌以非同步方式寫入以避免阻塞請求：

```rust
tokio::spawn(async move {
    log_activity(db, event).await;
});
```

### 11.3 效能考量

- 分區表以提升查詢效率
- 常見查詢模式的索引
- 獨立登入事件表（高流量）
- 每日彙總供儀表板使用

---

## 12. 相關文件

- [資料庫綱要](../04_DATABASE_SCHEMA.md) - 完整資料表定義
- [權限與 RBAC](../06_PERMISSIONS_RBAC.md) - 角色指派
- [API 規格](../05_API_SPECIFICATION.md) - 端點詳情

---

*最後更新：2026-01-17*
