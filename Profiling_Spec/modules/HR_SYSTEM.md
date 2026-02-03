# 人事管理系統規格

> **模組**：出勤、請假、補休管理  
> **最後更新**：2026-02-03

---

## 1. 系統目的

管理公司內部員工的人事相關作業：

- 請假申請與審核
- 特休假額度計算
- 補休假累計與使用
- 加班時數管理
- Google 行事曆同步

> **適用範圍**：僅限內部員工（`is_internal = true`），不包含外部審查人員

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

---

## 5. 補休假計算規則

- **來源**：加班時數 1:1 轉換
- **使用順序**：FIFO（先到期先使用）
- **到期處理**：自動轉換為加班費

---

## 6. API 端點

### 6.1 請假申請

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/hr/leaves` | 建立請假申請 |
| GET | `/hr/leaves` | 查詢列表 |
| GET | `/hr/leaves/{id}` | 查詢單一申請 |
| PUT | `/hr/leaves/{id}` | 更新申請 |
| DELETE | `/hr/leaves/{id}` | 刪除申請 |
| POST | `/hr/leaves/{id}/submit` | 送審 |
| POST | `/hr/leaves/{id}/withdraw` | 撤回 |
| POST | `/hr/leaves/{id}/cancel` | 取消 |
| POST | `/hr/leaves/{id}/revoke` | 銷假 |

### 6.2 請假審核

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/leaves/pending` | 待審核列表 |
| POST | `/hr/leaves/{id}/approve` | 核准 |
| POST | `/hr/leaves/{id}/reject` | 駁回 |

### 6.3 額度查詢

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/hr/leaves/balances` | 個人額度總覽 |
| GET | `/hr/leaves/balances/annual` | 特休餘額 |
| GET | `/hr/leaves/balances/compensatory` | 補休餘額 |

---

## 7. 核心資料模型

### leave_requests 表

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| user_id | UUID | 申請人 |
| leave_type | ENUM | 請假類別 |
| status | ENUM | 狀態 |
| start_date | DATE | 開始日期 |
| end_date | DATE | 結束日期 |
| hours | DECIMAL | 請假時數 |
| reason | TEXT | 請假原因 |

### annual_leave_balances 表

| 欄位 | 類型 | 說明 |
|------|------|------|
| user_id | UUID | FK |
| year | INTEGER | 年度 |
| granted | DECIMAL | 應有天數 |
| used | DECIMAL | 已使用 |
| expires_at | DATE | 到期日 |

---

## 8. 前端路由

| 路由 | 頁面 |
|------|------|
| `/hr/leaves` | 請假申請列表 |
| `/hr/leaves/new` | 新增請假 |
| `/hr/leaves/{id}` | 請假詳情 |
| `/hr/leaves/pending` | 待審核列表 |
| `/hr/leaves/balances` | 額度查詢 |
| `/hr/leaves/calendar` | 請假行事曆 |

---

## 9. 相關文件

- [權限控制](../06_PERMISSIONS_RBAC.md) - HR 相關權限
- [通知系統](./NOTIFICATION_SYSTEM.md) - 審核通知
