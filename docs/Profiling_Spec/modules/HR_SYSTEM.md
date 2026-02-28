# 人事管理系統規格

> **模組**：出勤、請假、補休管理  
> **版本**：7.0  
> **最後更新**：2026-03-01

---

## 1. 系統目的

管理公司內部員工的人事相關作業：

- 出勤打卡（上班/下班）
- 請假申請與多級審核
- 特休假額度計算（週年制）
- 補休假累計與使用
- 加班時數管理
- Google 行事曆同步
- 考勤統計報表

> **適用範圍**：僅限內部員工（`is_internal = true`），不包含外部審查人員。管理員帳號排除於考勤統計。

---

## 2. 請假類別

| 代碼 | 類別 | 年度上限 |
|------|------|----------|
| ANNUAL | 特休假 | 依年資 |
| PERSONAL | 事假 | 14 天 |
| SICK | 病假 | 30 天 |
| COMPENSATORY | 補休假 | 依累計 |
| MARRIAGE | 婚假 | 8 天 |
| BEREAVEMENT | 喪假 | 依親等 |
| MATERNITY | 產假 | 56 天 |
| PATERNITY | 陪產假 | 7 天 |
| MENSTRUAL | 生理假 | 每月 1 天 |
| OFFICIAL | 公假 | 無上限 |
| UNPAID | 無薪假 | 無上限 |

---

## 3. 請假狀態流程

```
DRAFT → PENDING_L1 → PENDING_L2 → PENDING_HR → PENDING_GM → APPROVED
  ↓         ↓            ↓            ↓            ↓
CANCELLED  REJECTED    REJECTED    REJECTED    REJECTED
```

> **注意**：審核階段動態決定，依 `total_days` 及組織設定決定需幾級審核。

| 狀態 | 說明 |
|------|------|
| `DRAFT` | 草稿 |
| `PENDING_L1` | 待主管審核 |
| `PENDING_L2` | 待部門主管審核 |
| `PENDING_HR` | 待人資審核 |
| `PENDING_GM` | 待總經理審核 |
| `APPROVED` | 已核准 |
| `REJECTED` | 已駁回 |
| `CANCELLED` | 已取消 |
| `REVOKED` | 已銷假 |

---

## 4. 特休假計算規則

**週年制計算**（依勞動基準法）：

| 年資 | 特休天數 |
|------|----------|
| 6 個月～1 年 | 3 天 |
| 1～2 年 | 7 天 |
| 2～3 年 | 10 天 |
| 3～5 年 | 14 天 |
| 5～10 年 | 15 天 |
| 10 年以上 | 每年 +1 天 |

**到期規則**：到職週年日 + 2 年到期

**未休補償追蹤**：`compensation_status` 欄位追蹤是否已折算工資

---

## 5. 補休假計算規則

- **來源**：加班時數 1:1 轉換
- **使用順序**：FIFO（先到期先使用）
- **到期處理**：自動轉換為加班費

---

## 6. API 端點

### 6.1 出勤打卡

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/attendance` | 出勤紀錄查詢 |
| POST | `/hr/attendance/clock-in` | 上班打卡 |
| POST | `/hr/attendance/clock-out` | 下班打卡 |
| GET | `/hr/attendance/stats` | 出勤統計 |
| PUT | `/hr/attendance/:id` | 更正打卡 |

### 6.2 請假管理

| 方法 | 端點 | 說明 |
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

### 6.3 加班管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/overtime` | 加班列表 |
| POST | `/hr/overtime` | 新增加班 |
| GET | `/hr/overtime/:id` | 加班詳情 |
| PUT | `/hr/overtime/:id` | 更新加班 |
| DELETE | `/hr/overtime/:id` | 刪除草稿 |
| POST | `/hr/overtime/:id/submit` | 送審 |
| POST | `/hr/overtime/:id/approve` | 核准 |
| POST | `/hr/overtime/:id/reject` | 駁回 |

### 6.4 特休餘額

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/balances/annual` | 特休餘額 |
| GET | `/hr/balances/comp-time` | 補休餘額 |
| GET | `/hr/balances/summary` | 餘額摘要 |
| POST | `/hr/balances/annual-entitlements` | 新增特休配額 |
| POST | `/hr/balances/:id/adjust` | 調整餘額 |
| GET | `/hr/balances/expired-compensation` | 過期未補休 |

### 6.5 Google 行事曆

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/calendar/status` | 同步狀態 |
| GET | `/hr/calendar/config` | 取得設定 |
| PUT | `/hr/calendar/config` | 更新設定 |
| POST | `/hr/calendar/connect` | 連接日曆 |
| POST | `/hr/calendar/disconnect` | 中斷連接 |
| POST | `/hr/calendar/sync` | 觸發同步 |
| GET | `/hr/calendar/history` | 同步歷程 |
| GET | `/hr/calendar/events` | 日曆事件 |

### 6.6 Dashboard

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/dashboard/calendar` | 行事曆資料 |
| GET | `/hr/staff` | 代理人選單 |
| GET | `/hr/internal-users` | 內部使用者 |

---

## 7. 前端路由

| 路由 | 頁面 |
|------|------|
| `/hr` | HR Dashboard（行事曆視圖） |
| `/hr/leaves` | 請假管理 |
| `/hr/overtime` | 加班管理 |
| `/hr/balances` | 餘額查詢 |
| `/settings` | 通知設定（含 Google Calendar 連接） |

---

## 8. 相關文件

- [權限控制](../06_PERMISSIONS_RBAC.md) - HR 相關權限
- [通知系統](./NOTIFICATION_SYSTEM.md) - 審核通知
- [出勤模組](../08_ATTENDANCE_MODULE.md) - 出勤詳細規格
