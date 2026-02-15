# 出勤模組

> **版本**：2.0  
> **最後更新**：2026-02-16  
> **對象**：開發人員、HR 管理員

---

## 1. 概覽

iPig 出勤模組整合打卡、請假、加班及 Google 行事曆同步功能。

---

## 2. 出勤打卡（Clock In/Out）

### 2.1 流程

```
使用者打卡 → 記錄時間 → 計算工時 → 更新出勤紀錄
```

### 2.2 核心邏輯

| 功能 | 說明 |
|------|------|
| 上班打卡 | 記錄 `clock_in` 時間 |
| 下班打卡 | 記錄 `clock_out` 時間，計算 `work_hours` |
| 更正打卡 | 管理員可修正歷史打卡紀錄 |
| 出勤統計 | 彙整指定期間出勤數據 |

### 2.3 API 端點

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/attendance` | 出勤紀錄查詢 |
| POST | `/hr/attendance/clock-in` | 上班打卡 |
| POST | `/hr/attendance/clock-out` | 下班打卡 |
| GET | `/hr/attendance/stats` | 出勤統計 |
| PUT | `/hr/attendance/:id` | 更正打卡 |

---

## 3. 請假管理

### 3.1 假別定義

| 假別 | 代碼 | 說明 |
|------|------|------|
| 特休 | ANNUAL | 依年資計算天數 |
| 事假 | PERSONAL | 事假 |
| 病假 | SICK | 病假 |
| 補休 | COMPENSATORY | 加班補償 |
| 婚假 | MARRIAGE | 婚假 |
| 喪假 | BEREAVEMENT | 喪假 |
| 產假 | MATERNITY | 女性產假 |
| 陪產假 | PATERNITY | 男性陪產假 |
| 生理假 | MENSTRUAL | 生理假 |
| 公假 | OFFICIAL | 公假 |
| 無薪假 | UNPAID | 無薪假 |

### 3.2 多級審核流程

```
申請者 → DRAFT → PENDING_L1(直屬主管)
                     │
                     ├── 核准 → PENDING_L2(二級主管)
                     │              │
                     │              ├── 核准 → PENDING_HR(HR)
                     │              │              │
                     │              │              ├── 核准 → PENDING_GM(總經理)
                     │              │              │              │
                     │              │              │              └── 核准 → APPROVED
                     │              │              │
                     │              │              └── 駁回 → REJECTED
                     │              │
                     │              └── 駁回 → REJECTED
                     │
                     └── 駁回 → REJECTED
```

> **注意**：審核階段動態決定，依 `total_days` 及組織設定決定需幾級審核。

### 3.3 API 端點

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/leaves` | 請假列表 |
| POST | `/hr/leaves` | 新增請假 |
| GET | `/hr/leaves/:id` | 請假詳情 |
| PUT | `/hr/leaves/:id` | 更新請假 |
| DELETE | `/hr/leaves/:id` | 刪除草稿 |
| POST | `/hr/leaves/:id/submit` | 送審 |
| POST | `/hr/leaves/:id/approve` | 核准 |
| POST | `/hr/leaves/:id/reject` | 駁回 |
| POST | `/hr/leaves/:id/cancel` | 撤銷 |

---

## 4. 加班管理

### 4.1 流程

```
申請加班 → DRAFT → 送審 → 核准/駁回
```

### 4.2 API 端點

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/overtime` | 加班列表 |
| POST | `/hr/overtime` | 新增加班 |
| GET | `/hr/overtime/:id` | 加班詳情 |
| PUT | `/hr/overtime/:id` | 更新加班 |
| DELETE | `/hr/overtime/:id` | 刪除草稿 |
| POST | `/hr/overtime/:id/submit` | 送審 |
| POST | `/hr/overtime/:id/approve` | 核准 |
| POST | `/hr/overtime/:id/reject` | 駁回 |

---

## 5. 特休餘額管理

### 5.1 功能

| 功能 | 說明 |
|------|------|
| 特休配額 | 依年資自動計算或手動分配 |
| 餘額查詢 | 查看各假別剩餘天數 |
| 補休餘額 | 加班轉換的補休天數 |
| 餘額調整 | 管理員可手動調整 |
| 到期提醒 | 特休即將到期通知 |

### 5.2 API 端點

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/balances/annual` | 特休餘額 |
| GET | `/hr/balances/comp-time` | 補休餘額 |
| GET | `/hr/balances/summary` | 餘額摘要 |
| POST | `/hr/balances/annual-entitlements` | 新增特休配額 |
| POST | `/hr/balances/:id/adjust` | 調整餘額 |
| GET | `/hr/balances/expired-compensation` | 過期未補休 |

---

## 6. Google 行事曆同步

### 6.1 架構

```
iPig 系統 ←→ Google Calendar API ←→ Google 行事曆
             (Service Account)
```

### 6.2 功能

| 功能 | 說明 |
|------|------|
| 連接日曆 | 使用 Service Account 連接指定日曆 |
| 同步請假 | 核准的請假自動同步至日曆 |
| 同步加班 | 核准的加班自動同步至日曆 |
| 衝突處理 | 偵測並解決同步衝突 |
| 同步歷程 | 記錄所有同步操作 |

### 6.3 API 端點

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
| POST | `/hr/calendar/conflicts/:id/resolve` | 解決衝突 |
| GET | `/hr/calendar/events` | 日曆事件 |

---

## 7. HR Dashboard

### 7.1 功能

| 功能 | 說明 |
|------|------|
| 行事曆視圖 | 顯示請假、加班月曆 |
| 代理人查詢 | 查詢可用代理人 |
| 人員列表 | 內部使用者清單 |

### 7.2 API 端點

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/hr/dashboard/calendar` | 行事曆資料 |
| GET | `/hr/staff` | 代理人選單 |
| GET | `/hr/internal-users` | 內部使用者 |

---

## 8. 資料模型

### 8.1 出勤紀錄 (attendance_records)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 使用者 |
| date | DATE | 出勤日期 |
| clock_in | TIMESTAMPTZ | 上班時間 |
| clock_out | TIMESTAMPTZ | 下班時間 |
| work_hours | NUMERIC(5,2) | 工作時數 |

### 8.2 請假申請 (leave_requests)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 申請者 |
| proxy_user_id | UUID | 代理人 |
| leave_type | leave_type | 假別 |
| start_date | DATE | 起始日 |
| end_date | DATE | 結束日 |
| total_days | NUMERIC(5,2) | 天數 |
| status | leave_status | 狀態 |
| current_approver_id | UUID | 下一審核者 |
| approved_days | NUMERIC(5,2) | 核准天數 |
| approval_history | JSONB | 審核歷程 |

### 8.3 加班紀錄 (overtime_records)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 申請者 |
| overtime_date | DATE | 加班日期 |
| start_time | TIME | 起始時間 |
| end_time | TIME | 結束時間 |
| total_hours | NUMERIC(5,2) | 總時數 |
| compensation_type | VARCHAR | 補償方式（加班費/補休）|
| status | VARCHAR | 審核狀態 |

---

*下一章：[擴展性](./09_EXTENSIBILITY.md)*
