# 安全與稽核

> **版本**：7.0  
> **最後更新**：2026-03-01  
> **對象**：系統管理員、安全人員、開發人員

---

## 1. 概覽

iPig 系統基於 GLP（Good Laboratory Practice）合規要求，建置完善的安全與稽核機制：

```
┌──────────────────────────────────────────────────────────────────────┐
│                        安全與稽核架構                                  │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │ Rate Limiter│  │  Real IP    │  │   CORS      │                 │
│  │ (中間件)     │  │ (中間件)     │  │ (Tower-HTTP)│                 │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │
│         │                 │                 │                        │
│         ▼                 ▼                 ▼                        │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Activity Logger (中間件)                   │    │
│  │         記錄所有受保護端點的操作至 user_activity_logs          │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ Login       │  │ Session     │  │ Security    │                │
│  │ Tracker     │  │ Manager     │  │ Alerts      │                │
│  └─────────────┘  └─────────────┘  └─────────────┘                │
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ GeoIP       │  │ Audit Logs  │  │ Partition   │                │
│  │ (MaxMind)   │  │ (JSONB)     │  │ Maintenance │                │
│  └─────────────┘  └─────────────┘  └─────────────┘                │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 2. 中間件層

### 2.1 Rate Limiter

**檔案**：`middleware/rate_limiter.rs`

| 類型 | 限制 | 適用範圍 |
|------|------|----------|
| 認證 | 100/min | login、forgot-password、reset-password、refresh |
| 寫入 | 120/min | POST/PUT/PATCH/DELETE |
| 上傳 | 30/min | 檔案上傳路由 |
| 一般 API | 600/min | 其餘端點 |
| 回應 | HTTP 429 Too Many Requests | Retry-After header |

**實作**：DashMap 記憶體儲存，依 IP 計數。

### 2.2 Real IP 擷取

**檔案**：`middleware/real_ip.rs` (2KB)

從 HTTP 標頭依序解析真實 IP：
1. `X-Forwarded-For`（取第一個非私有 IP）
2. `X-Real-IP`
3. `CF-Connecting-IP`（Cloudflare）
4. 連接端 IP（fallback）

### 2.3 Activity Logger

**檔案**：`middleware/activity_logger.rs` (7KB)

自動記錄所有受保護端點的操作：

| 記錄欄位 | 說明 |
|----------|------|
| user_id | 操作者 |
| action | HTTP 方法 + 路徑 |
| entity_type | 實體類型（自動解析）|
| entity_id | 實體 ID |
| details | 請求/回應摘要 (JSONB) |
| ip_address | 真實 IP (INET) |
| user_agent | 瀏覽器 UA |
| session_id | 工作階段 ID |
| created_at | 時間戳 |

---

## 3. 登入安全

### 3.1 Login Tracker

**檔案**：`services/login_tracker.rs` (15KB)

| 功能 | 說明 |
|------|------|
| 登入事件記錄 | 每次登入嘗試記錄 IP、GeoIP、成功/失敗 |
| 帳號鎖定 | 連續 N 次失敗後鎖定帳號 |
| 異常偵測 | 偵測異常登入行為 |

### 3.2 GeoIP 定位

**檔案**：`services/geoip.rs` (3KB)

使用 MaxMind GeoLite2-City 資料庫：

```
IP 地址 → GeoIP 查詢 → { country, city, latitude, longitude }
```

| 使用場景 | 說明 |
|----------|------|
| 登入紀錄 | 記錄登入地點 |
| 工作階段 | 顯示使用者地理位置 |
| 安全警報 | 異地登入偵測 |

### 3.3 登入事件表 (login_events)

```sql
CREATE TABLE login_events (
    id UUID PRIMARY KEY,
    user_id UUID,
    email VARCHAR(255),
    success BOOLEAN,
    ip_address INET,
    user_agent TEXT,
    geoip JSONB,       -- { country, city, lat, lng }
    failure_reason TEXT,
    created_at TIMESTAMPTZ
);
```

---

## 4. 工作階段管理

### 4.1 Session Manager

**檔案**：`services/session_manager.rs` (6KB)

| 功能 | 說明 |
|------|------|
| 建立工作階段 | 登入成功時建立 |
| 活動追蹤 | Heartbeat 更新 last_activity_at |
| 強制登出 | 管理員可強制終止他人 Session |
| 自動清理 | 排程清理過期 Session |

### 4.2 前端 Heartbeat

**檔案**：`hooks/useHeartbeat.ts`

前端透過 `POST /auth/heartbeat` 定期回報活動狀態，觸發條件：
- 滑鼠移動
- 鍵盤操作
- 點擊操作

### 4.3 工作階段表 (user_sessions)

```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    token_jti UUID,      -- JWT ID 關聯
    ip_address INET,
    user_agent TEXT,
    geoip JSONB,
    last_activity_at TIMESTAMPTZ,
    expired_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ
);
```

---

## 5. 安全警報

### 5.1 Security Alerts

**表名**：`security_alerts`

系統自動偵測異常行為並產生警報：

| 警報類型 | 說明 |
|----------|------|
| 多次登入失敗 | 同一帳號連續失敗 |
| 異地登入 | 從非常用地點登入 |
| 帳號鎖定 | 帳號被鎖定 |
| 可疑操作 | 異常操作模式 |

### 5.2 管理 API

| 端點 | 說明 |
|------|------|
| `GET /admin/audit/alerts` | 安全警報列表 |
| `POST /admin/audit/alerts/:id/resolve` | 解決警報 |

---

## 6. 稽核追蹤

### 6.1 雙層稽核

| 層級 | 表名 | 說明 |
|------|------|------|
| 簡易稽核 | audit_logs | 實體 CRUD 操作紀錄 |
| 完整活動 | user_activity_logs | 所有 API 操作紀錄（分割表）|

### 6.2 簡易稽核 (audit_logs)

記錄實體級別的變更：

| 欄位 | 說明 |
|------|------|
| actor_user_id | 操作者 |
| action | 動作（create, update, delete）|
| entity_type | 實體類型 |
| entity_id | 實體 ID |
| before_data | 變更前資料 (JSONB) |
| after_data | 變更後資料 (JSONB) |
| ip_address | IP |

### 6.3 完整活動 (user_activity_logs)

按月分割的完整操作日誌：

- **分割策略**：RANGE BY `created_at`，每月一個分割區
- **自動維護**：`partition_maintenance.rs` 排程管理，保留 6 個月
- **支援匯出**：CSV/Excel 格式

### 6.4 稽核 API

| 端點 | 說明 |
|------|------|
| `GET /admin/audit/activities` | 活動記錄列表（分頁、篩選）|
| `GET /admin/audit/activities/export` | 匯出活動記錄 |
| `GET /admin/audit/activities/user/:user_id` | 使用者活動時間軸 |
| `GET /admin/audit/activities/entity/:type/:id` | 實體歷程 |
| `GET /admin/audit/logins` | 登入事件列表 |
| `GET /admin/audit/sessions` | 工作階段列表 |
| `POST /admin/audit/sessions/:id/logout` | 強制登出 |
| `GET /admin/audit/dashboard` | 安全儀表板 |

---

## 7. 安全儀表板

管理員可透過 `/admin/audit` 頁面查看：

- **活動統計**：今日/本週/本月操作量
- **登入統計**：成功/失敗比例
- **活躍工作階段**：線上使用者列表
- **安全警報**：未解決的警報
- **使用者時間軸**：個別使用者操作歷程
- **實體歷程**：特定資源的異動紀錄

---

## 8. GLP 合規

### 8.1 電子簽章

支援兩種簽章方式：

| 方式 | 欄位 | 說明 |
|------|------|------|
| 密碼驗證 | `signature_data` (JSONB) | 傳統密碼確認簽署 |
| 手寫簽名 | `handwriting_svg` + `stroke_data` | 觸控手寫簽名（`signature_pad`）|

**簽章場景**：
- 犧牲紀錄 (`/signatures/sacrifice/:id`)
- 觀察紀錄 (`/signatures/observation/:id`)
- 安樂死紀錄 (`/signatures/euthanasia/:id`)
- 轉讓紀錄 (`/signatures/transfer/:id`)
- 計畫審查 (`/signatures/protocol/:id`)

簽章資料儲存於 `electronic_signatures` 表，簽章後紀錄自動鎖定。

### 8.2 紀錄標註

- 任何紀錄類型均可新增標註（`record_annotations`）
- 不可逆操作追蹤

### 8.3 版本控制

- 觀察、手術、體重、疫苗、犧牲、病理、血檢紀錄均有版本歷程
- `record_versions` 表保存每次修改的 JSON 快照

---

## 9. 與其他文件的關聯

- [AUDIT_LOGGING.md](./guides/AUDIT_LOGGING.md) - GLP 合規操作指南（詳細操作步驟）
- [06_PERMISSIONS_RBAC.md](./06_PERMISSIONS_RBAC.md) - 角色權限定義
- [01_ARCHITECTURE_OVERVIEW.md](./01_ARCHITECTURE_OVERVIEW.md) - 系統架構

---

*下一章：[出勤模組](./08_ATTENDANCE_MODULE.md)*
