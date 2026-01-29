# HR Attendance Module - Business Rules

> **Version**: 2.0  
> **Last Updated**: 2026-01-29  
> **Audience**: HR Team, Developers

> [!NOTE]
> 此文件專注於業務規則。完整的 API 端點請參考 [05_API_SPECIFICATION.md](./05_API_SPECIFICATION.md)，
> 資料表定義請參考 [04_DATABASE_SCHEMA.md](./04_DATABASE_SCHEMA.md)。

---

## 1. 模組範圍

| 包含 | 不包含 |
|------|--------|
| 內部員工 (`is_internal = true`) | 外部承包商 |
| 打卡/簽退追蹤 | 生物辨識整合 |
| 加班與補休產生 | 薪資計算 |
| 請假申請與審批 | 複雜排班 |
| 特休與補休餘額 | 專案時間追蹤 |
| Google Calendar 同步 | Microsoft/Outlook 同步 |

---

## 2. 補休規則 (Comp Time)

| 規則 | 說明 |
|------|------|
| 產生方式 | 核准的加班紀錄自動產生 |
| 倍率 | 平日 1.0x / 假日 1.33x / 國定假日 1.66x~2.0x |
| 有效期限 | 加班日起 **1 年** |
| 使用順序 | FIFO（先到期者先使用）|
| 最小單位 | 0.5 小時 |

---

## 3. 特休規則 (Annual Leave)

| 年資 | 特休天數 |
|------|----------|
| 6個月 - 1年 | 3 天 (按比例) |
| 1 - 2 年 | 7 天 |
| 2 - 3 年 | 10 天 |
| 3 - 5 年 | 14 天 |
| 5 - 10 年 | 15 天 |
| 10+ 年 | 15 + 每年1天 (上限30) |

**有效期限**: 發放年度結束後 **2 年**

---

## 4. 假別類型

| 假別 | 代碼 | 審批層級 | 附件要求 |
|------|------|----------|----------|
| 特休假 | ANNUAL | L1 (≤3天), L2 (>3天) | 無 |
| 事假 | PERSONAL | L1 (≤1天), L2 (>1天) | 無 |
| 病假 | SICK | L1 (≤3天), HR (>3天) | 醫生證明 (>3天) |
| 補休假 | COMPENSATORY | L1 | 無 |
| 婚假 | MARRIAGE | L2 + HR | 結婚證書 |
| 喪假 | BEREAVEMENT | L1 + HR | 死亡證明 |
| 產假 | MATERNITY | L2 + HR + GM | 醫療證明 |
| 陪產假 | PATERNITY | L2 + HR | 出生證明 |
| 生理假 | MENSTRUAL | L1 | 無 |
| 公假 | OFFICIAL | L2 | 公文 |
| 無薪假 | UNPAID | L2 + HR + GM | 理由說明 |

---

## 5. 審批流程

```
DRAFT → PENDING_L1 → PENDING_L2 → PENDING_HR → APPROVED
                ↓           ↓           ↓
            REJECTED    REJECTED    REJECTED

DRAFT → CANCELLED (申請人取消)
APPROVED → REVOKED (請假開始後撤銷)
```

| 層級 | 角色 | 說明 |
|------|------|------|
| L1 | Direct Manager | 直屬主管 |
| L2 | Department Head | 部門主管 |
| HR | IACUC Staff (執行秘書) | 行政審核 |
| GM | Admin | 特殊假別最終審核 |

---

## 6. Google Calendar 同步

- **方式**: 共用行事曆 + 專用 Gmail 帳號
- **頻率**: 每日兩次 (08:00, 18:00 台灣時間)
- **方向**: 主要為 iPig → Google，有衝突偵測

### 事件格式
```
標題: [請假] 王小明 - 特休
說明: iPig Leave ID: abc-123
      Type: 特休假
      Status: 已核准
```

### 衝突處理
- Google 刪除事件 → 標記待審查 (不刪除 iPig 請假)
- Google 修改時間 → 標記待審查
- Google 修改標題 → 忽略

---

## 7. 通知機制

| 事件 | 收件人 | 管道 |
|------|--------|------|
| 請假提交 | 審批人 | Email + App |
| 請假核准/拒絕 | 申請人 | Email + App |
| 餘額即將到期 (30天) | 員工 | Email |
| 餘額即將到期 (7天) | 員工 | Email + App |
| Calendar 衝突 | HR Admin | App |

---

## 8. 背景任務

| 任務 | 排程 | 說明 |
|------|------|------|
| Calendar Sync (AM) | 08:00 daily | 推送/拉取變更 |
| Calendar Sync (PM) | 18:00 daily | 推送/拉取變更 |
| Expiration Check | 00:00 daily | 標記過期餘額 |
| Expiry Warnings | 00:00 daily | 發送到期通知 |

---

*Related: [API Specification](./05_API_SPECIFICATION.md) | [Database Schema](./04_DATABASE_SCHEMA.md) | [Audit Logging](./07_AUDIT_LOGGING.md)*
