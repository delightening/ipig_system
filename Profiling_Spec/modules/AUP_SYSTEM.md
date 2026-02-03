# AUP 審查系統規格

> **模組**：IACUC 動物使用計畫書提交與審查系統  
> **最後更新**：2026-02-03

---

## 1. 系統目的與範圍

本系統用於管理 IACUC（Institutional Animal Care and Use Committee）動物試驗計畫書（Animal Use Protocol, AUP）的完整生命週期：

- 草稿撰寫
- 提交審查
- 修訂回覆
- 核准/否決
- 暫停與結案

---

## 2. 使用者角色

| 角色 | 核心能力 |
|------|----------|
| PI | 建立與編輯草稿、提交計畫、回應審查意見 |
| REVIEWER / VET | 審查指派案件、新增審查意見 |
| CHAIR | 主導審查決策、核准或否決計畫 |
| IACUC_STAFF | 指派審查人員、管理流程狀態 |
| SYSTEM_ADMIN | 全系統管理 |

---

## 3. 核心資料模型

| 資料表 | 說明 |
|--------|------|
| `protocols` | 計畫書主檔，含狀態與草稿內容 |
| `protocol_versions` | 不可變版本快照 |
| `protocol_status_history` | 狀態轉移歷程 |
| `review_assignments` | 審查人員指派 |
| `review_comments` | 審查意見 |
| `attachments` | 附件檔案 |

---

## 4. 計畫書狀態機

### 4.1 狀態定義

| 狀態 | 說明 |
|------|------|
| `DRAFT` | 草稿 |
| `SUBMITTED` | 已提交 |
| `PRE_REVIEW` | 行政預審 |
| `UNDER_REVIEW` | 審查中 |
| `REVISION_REQUIRED` | 需修訂 |
| `RESUBMITTED` | 已重送 |
| `APPROVED` | 核准 |
| `APPROVED_WITH_CONDITIONS` | 附條件核准 |
| `DEFERRED` | 延後審議 |
| `REJECTED` | 否決 |
| `SUSPENDED` | 暫停 |
| `CLOSED` | 結案 |

### 4.2 狀態轉移

```
DRAFT → SUBMITTED (PI)
  ↓
SUBMITTED → PRE_REVIEW (IACUC_STAFF)
  ↓
PRE_REVIEW → UNDER_REVIEW (IACUC_STAFF, CHAIR)
  ↓
UNDER_REVIEW → REVISION_REQUIRED (REVIEWER, CHAIR)
             → APPROVED / REJECTED (CHAIR)
             → DEFERRED (CHAIR)
  ↓
REVISION_REQUIRED → RESUBMITTED (PI)
  ↓
RESUBMITTED → PRE_REVIEW / UNDER_REVIEW

APPROVED → SUSPENDED / CLOSED
```

---

## 5. API 端點

### 5.1 計畫書管理

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/protocols` | 建立計畫書 |
| GET | `/protocols` | 列表查詢 |
| GET | `/protocols/{id}` | 取得單一計畫書 |
| PATCH | `/protocols/{id}` | 更新內容 |
| POST | `/protocols/{id}/submit` | 提交審查 |
| POST | `/protocols/{id}/status` | 變更狀態 |

### 5.2 版本管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/protocols/{id}/versions` | 取得版本列表 |
| GET | `/protocols/{id}/versions/{vid}` | 取得特定版本 |
| GET | `/protocols/{id}/status-history` | 取得狀態歷程 |

### 5.3 審查流程

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/reviews/assignments` | 指派審查人員 |
| GET | `/reviews/assignments` | 查詢指派列表 |
| POST | `/reviews/comments` | 新增審查意見 |
| GET | `/reviews/comments` | 查詢意見列表 |
| POST | `/reviews/comments/{id}/resolve` | 解決意見 |

### 5.4 附件管理

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/attachments` | 上傳附件 |
| GET | `/attachments` | 查詢附件 |
| GET | `/attachments/{id}/download` | 下載附件 |

---

## 6. AUP 表單結構

計畫書草稿儲存於 `protocols.working_content` 欄位（JSONB），包含 9 個章節：

1. **基本資料** - GLP、PI、計畫名稱、期間
2. **3Rs 原則說明** - Replacement, Reduction, Refinement
3. **試驗物質與對照組**
4. **試驗流程與麻醉止痛**
5. **參考文獻**
6. **手術計畫**
7. **動物資訊** - 種類、性別、數量、年齡、體重、來源
8. **人員名單與職責**
9. **流程圖與附件**

---

## 7. 前端路由

| 路由 | 頁面 |
|------|------|
| `/protocols` | 計畫書列表 |
| `/protocols/new` | 建立計畫書 |
| `/protocols/:id` | 檢視/編輯計畫書 |
| `/my-projects` | 我的計劃（PI/CLIENT） |

---

## 8. 相關文件

- [權限控制](../06_PERMISSIONS_RBAC.md) - 角色權限詳細說明
- [稽核日誌](../guides/AUDIT_LOGGING.md) - 變更追蹤
- [通知系統](./NOTIFICATION_SYSTEM.md) - 審查通知
