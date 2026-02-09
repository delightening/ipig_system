# 出勤模組規格

> **版本**：1.0  
> **最後更新**：2026-01-17  
> **對象**：人資團隊、開發人員

---

## 1. 目的

出勤模組為內部員工提供完整的出勤與請假管理，包含：

- **出勤打卡** - 上下班簽到簽退
- **加班管理** - 加班紀錄與補休產生
- **請假管理** - 請假申請與審核流程
- **餘額追蹤** - 特休與補休餘額（含到期管理）
- **Google 行事曆同步** - 與共用行事曆雙向同步

---

## 2. 範圍

| 涵蓋範圍 | 不涵蓋範圍 |
|----------|------------|
| 僅限內部員工（`is_internal = true`）| 外部承攬人員 |
| 上下班打卡追蹤 | 生物辨識整合 |
| 加班與補休產生 | 薪資計算 |
| 請假申請與核准 | 複雜班表排程 |
| 特休與補休管理 | 專案工時追蹤 |
| Google 行事曆同步 | Microsoft/Outlook 同步 |

---

## 3. 商業規則

### 3.1 補休

| 規則 | 說明 |
|------|------|
| 產生方式 | 由核准的加班紀錄產生 |
| 倍率 | 平日：1.0x，假日：1.33x，國定假日：1.66x 或 2.0x |
| 有效期限 | 自加班日起 **1 年** |
| 使用順序 | FIFO（最舊先過期優先使用）|
| 最小單位 | 0.5 小時 |

### 3.2 特休

| 規則 | 說明 |
|------|------|
| 給假依據 | 依勞動基準法年資計算 |
| 給假時間 | 每年初 |
| 有效期限 | 自給假年度結束起 **2 年**（如：2025 年給假於 2027-12-31 到期）|
| 遞延規則 | 未使用天數可於有效期內遞延至次年 |
| 最小單位 | 0.5 天 |

### 3.3 特休年資對照表

| 服務年資 | 特休天數 |
|----------|----------|
| 6 個月 - 1 年 | 3 天（按比例）|
| 1 - 2 年 | 7 天 |
| 2 - 3 年 | 10 天 |
| 3 - 5 年 | 14 天 |
| 5 - 10 年 | 15 天 |
| 10 年以上 | 15 + 每年加 1 天（最多 30 天）|

### 3.4 請假類型

| 類型 | 代碼 | 核准層級 | 需檢附文件 |
|------|------|----------|------------|
| 特休假 | ANNUAL | L1（≤3 天），L2（>3 天）| 否 |
| 事假 | PERSONAL | L1（≤1 天），L2（>1 天）| 否 |
| 病假 | SICK | L1（≤3 天），HR（>3 天）| 醫師證明（>3 天）|
| 補休假 | COMPENSATORY | L1 | 否 |
| 婚假 | MARRIAGE | L2 + HR | 結婚證明 |
| 喪假 | BEREAVEMENT | L1 + HR | 死亡證明 |
| 產假 | MATERNITY | L2 + HR + GM | 醫療證明 |
| 陪產假 | PATERNITY | L2 + HR | 出生證明 |
| 生理假 | MENSTRUAL | L1 | 否 |
| 公假 | OFFICIAL | L2 | 公文 |
| 無薪假 | UNPAID | L2 + HR + GM | 需說明原因 |

---

## 4. 審核流程

### 4.1 核准層級

| 層級 | 角色 | 說明 |
|------|------|------|
| L1 | 直屬主管 | 第一級核准 |
| L2 | 部門主管 | 第二級核准 |
| HR | 執行秘書 | 行政核准 |
| GM | 系統管理員 | 特殊案件最終核准 |

### 4.2 流程圖

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  DRAFT  │───►│PENDING  │───►│PENDING  │───►│PENDING  │───►│APPROVED │
│  草稿   │    │  L1     │    │  L2     │    │  HR     │    │  已核准 │
└─────────┘    └────┬────┘    └────┬────┘    └────┬────┘    └─────────┘
     │              │              │              │
     │              ▼              ▼              ▼
     │         ┌─────────┐    ┌─────────┐    ┌─────────┐
     │         │REJECTED │    │REJECTED │    │REJECTED │
     │         │  駁回   │    │  駁回   │    │  駁回   │
     │         └─────────┘    └─────────┘    └─────────┘
     │
     ▼
┌─────────┐
│CANCELLED│
│  取消   │
└─────────┘
```

### 4.3 特殊情境

| 情境 | 處理方式 |
|------|----------|
| 緊急請假 | 請假期間/之後補提申請，標記為急件優先審核 |
| 事後補請 | 請假日期後補提申請，需說明原因 |
| 自我審核 | 主管不可核准自己的申請（向上呈請）|

---

## 5. 資料庫綱要

### 5.1 核心資料表

#### attendance_records（出勤紀錄）
| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 員工 |
| work_date | DATE | 工作日期 |
| clock_in_time | TIMESTAMPTZ | 簽到時間 |
| clock_out_time | TIMESTAMPTZ | 簽退時間 |
| regular_hours | NUMERIC(5,2) | 正常工時 |
| overtime_hours | NUMERIC(5,2) | 加班時數 |
| status | VARCHAR(20) | normal, late, early_leave, absent, leave, holiday |

#### overtime_records（加班紀錄）
| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 員工 |
| overtime_date | DATE | 加班日期 |
| start_time | TIMESTAMPTZ | 開始時間 |
| end_time | TIMESTAMPTZ | 結束時間 |
| hours | NUMERIC(5,2) | 加班時數 |
| overtime_type | VARCHAR(20) | weekday, weekend, holiday |
| multiplier | NUMERIC(3,2) | 補休倍率 |
| comp_time_hours | NUMERIC(5,2) | 產生的補休時數 |
| comp_time_expires_at | DATE | 到期日（1 年）|
| status | VARCHAR(20) | draft, pending, approved, rejected |

#### annual_leave_entitlements（特休額度）
| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 員工 |
| entitlement_year | INTEGER | 年度（如 2025）|
| entitled_days | NUMERIC(5,2) | 給假天數 |
| used_days | NUMERIC(5,2) | 已使用天數 |
| expires_at | DATE | 到期日（2 年）|
| is_expired | BOOLEAN | 完全過期標記 |

#### comp_time_balances（補休餘額）
| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 員工 |
| overtime_record_id | UUID | 來源加班紀錄 |
| original_hours | NUMERIC(5,2) | 取得時數 |
| used_hours | NUMERIC(5,2) | 已使用時數 |
| earned_date | DATE | 取得日期 |
| expires_at | DATE | 到期日（1 年）|
| is_expired | BOOLEAN | 完全過期標記 |

#### leave_requests（請假申請）
| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 申請人 |
| leave_type | leave_type ENUM | 請假類型 |
| start_date | DATE | 開始日期 |
| end_date | DATE | 結束日期 |
| total_days | NUMERIC(5,2) | 總天數 |
| reason | TEXT | 事由 |
| status | leave_status ENUM | 目前狀態 |
| current_approver_id | UUID | 下一位審核者 |

---

## 6. Google 行事曆同步

### 6.1 架構

- **方式**：共用行事曆搭配專屬 Gmail 帳號
- **同步頻率**：每日兩次（台灣時間 8 AM、6 PM）
- **方向**：主要為 iPig → Google，附帶衝突偵測
- **可見性**：所有具有行事曆存取權限的員工可檢視事件

### 6.2 事件格式

```json
{
  "summary": "[請假] 王小明 - 特休",
  "description": "iPig 請假編號: abc-123\n類型: 特休假\n狀態: 已核准",
  "start": { "date": "2026-01-20" },
  "end": { "date": "2026-01-21" },
  "extendedProperties": {
    "private": {
      "ipig_leave_id": "abc-123",
      "ipig_leave_type": "ANNUAL",
      "ipig_sync_version": "3"
    }
  }
}
```

### 6.3 同步規則

| iPig 事件 | Google 動作 |
|-----------|-------------|
| 請假核准 | 建立事件 |
| 請假更新 | 更新事件 |
| 請假取消/撤銷 | 刪除事件 |

### 6.4 衝突處理

| Google 變更 | iPig 回應 |
|-------------|-----------|
| 事件刪除 | 標記待審（不刪除請假）|
| 時間變更 | 標記待審 |
| 標題變更 | 忽略（非關鍵）|

衝突儲存於 `calendar_sync_conflicts`，需管理員解決：
- **保留 iPig 版本**：重新推送至 Google
- **接受 Google 變更**：更新 iPig（可能需重新核准）
- **忽略**：標記已解決，不採取動作

---

## 7. API 端點

### 7.1 出勤

```
GET    /api/hr/attendance              # 列表出勤紀錄
POST   /api/hr/attendance/clock-in     # 簽到
POST   /api/hr/attendance/clock-out    # 簽退
PUT    /api/hr/attendance/:id          # 手動修正
```

### 7.2 加班

```
GET    /api/hr/overtime                # 列表加班紀錄
POST   /api/hr/overtime                # 提交加班
PUT    /api/hr/overtime/:id            # 更新加班
DELETE /api/hr/overtime/:id            # 刪除（僅限草稿）
POST   /api/hr/overtime/:id/submit     # 送審
POST   /api/hr/overtime/:id/approve    # 核准
POST   /api/hr/overtime/:id/reject     # 駁回
```

### 7.3 請假

```
GET    /api/hr/leaves                  # 列表請假申請
POST   /api/hr/leaves                  # 建立申請
GET    /api/hr/leaves/:id              # 取得詳情
PUT    /api/hr/leaves/:id              # 更新（僅限草稿）
DELETE /api/hr/leaves/:id              # 刪除（僅限草稿）
POST   /api/hr/leaves/:id/submit       # 送審
POST   /api/hr/leaves/:id/approve      # 核准
POST   /api/hr/leaves/:id/reject       # 駁回
POST   /api/hr/leaves/:id/cancel       # 取消（開始前）
POST   /api/hr/leaves/:id/revoke       # 撤銷（開始後）
```

### 7.4 餘額

```
GET    /api/hr/balances/annual         # 特休餘額
GET    /api/hr/balances/comp-time      # 補休餘額
GET    /api/hr/balances/summary        # 綜合摘要
```

### 7.5 行事曆同步

```
GET    /api/hr/calendar/status         # 同步狀態
POST   /api/hr/calendar/connect        # 設定行事曆
POST   /api/hr/calendar/disconnect     # 移除設定
POST   /api/hr/calendar/sync           # 手動觸發同步
PUT    /api/hr/calendar/settings       # 更新設定
GET    /api/hr/calendar/conflicts      # 列表衝突
POST   /api/hr/calendar/conflicts/:id/resolve  # 解決衝突
```

---

## 8. UI 頁面

### 8.1 導覽

```
👥 人員管理
  ├── 出勤打卡
  ├── 加班申請
  ├── 請假申請
  ├── 假期餘額
  └── 行事曆設定 (管理員)
```

### 8.2 員工頁面

| 頁面 | 說明 |
|------|------|
| 出勤 | 簽到簽退、週摘要 |
| 加班 | 提交加班、檢視歷程 |
| 請假 | 申請請假、檢視歷程 |
| 餘額 | 檢視特休與補休 |

### 8.3 管理員頁面

| 頁面 | 說明 |
|------|------|
| 團隊行事曆 | 完整團隊請假行事曆 |
| 餘額總覽 | 所有使用者餘額 |
| 待核准項目 | 待核准申請列表 |
| 同步狀態 | 行事曆同步監控 |
| 衝突管理 | 解決同步衝突 |

---

## 9. 權限

| 代碼 | 說明 |
|------|------|
| hr.attendance.view.own | 檢視個人出勤 |
| hr.attendance.clock | 簽到簽退 |
| hr.attendance.view.all | 檢視所有出勤 |
| hr.attendance.correct | 修正紀錄 |
| hr.overtime.view.own | 檢視個人加班 |
| hr.overtime.create | 提交加班 |
| hr.overtime.approve | 核准加班 |
| hr.leave.view.own | 檢視個人請假 |
| hr.leave.create | 申請請假 |
| hr.leave.approve.l1 | L1 核准 |
| hr.leave.approve.l2 | L2 核准 |
| hr.leave.approve.hr | HR 核准 |
| hr.leave.approve.gm | GM 核准 |
| hr.balance.view.own | 檢視個人餘額 |
| hr.balance.view.all | 檢視所有餘額 |
| hr.balance.manage | 調整餘額 |
| hr.calendar.config | 設定同步 |
| hr.calendar.sync | 觸發同步 |
| hr.calendar.conflicts | 解決衝突 |

---

## 10. 通知

| 事件 | 收件人 | 管道 |
|------|--------|------|
| 請假送審 | 審核者 | Email、站內 |
| 請假核准 | 申請人 | Email、站內 |
| 請假駁回 | 申請人 | Email、站內 |
| 餘額即期（30 天）| 使用者 | Email |
| 餘額即期（7 天）| 使用者 | Email、站內 |
| 偵測到同步衝突 | HR 管理員 | 站內 |

---

## 11. 背景工作

| 工作 | 排程 | 說明 |
|------|------|------|
| 行事曆同步（AM）| 每日 08:00 | 推送/拉取變更 |
| 行事曆同步（PM）| 每日 18:00 | 推送/拉取變更 |
| 到期檢查 | 每日 00:00 | 標記已過期餘額 |
| 到期提醒 | 每日 00:00 | 發送到期通知 |
| 每日統計彙總 | 每日 01:00 | 計算彙總資料 |

---

## 12. 相關文件

- [權限與 RBAC](./06_PERMISSIONS_RBAC.md) - 角色指派
- [API 規格](./05_API_SPECIFICATION.md) - 完整端點詳情
- [擴展性](./09_EXTENSIBILITY.md) - 未來擴展規劃

---

*最後更新：2026-01-17*
