# AUP 審查系統規格

> **模組**：IACUC 動物使用計畫書提交與審查系統  
> **版本**：7.0  
> **最後更新**：2026-03-01

---

## 1. 系統目的與範圍

本系統用於管理 IACUC（Institutional Animal Care and Use Committee）動物試驗計畫書（Animal Use Protocol, AUP）的完整生命週期：

- 草稿撰寫（含共同編輯者）
- 提交審查
- 多層審查（Pre-Review → 獸醫審查 → 委員會審查）
- 修訂回覆（多輪往返）
- 核准/否決
- 暫停與結案
- **草稿軟刪除**（DELETED 狀態）
- **計畫變更申請（Amendment）**
- **手寫電子簽章審查核准**
- **PDF 匯出**（全 9 節完整呈現）

---

## 2. 使用者角色

| 角色 | 核心能力 |
|------|----------|
| PI | 建立/編輯草稿、提交計畫、回應審查意見、管理變更申請 |
| REVIEWER | 審查指派案件、新增審查意見 |
| VET | 獸醫審查節點、新增審查意見 |
| IACUC_CHAIR | 主導審查決策、核准或否決計畫、手寫簽章核准 |
| IACUC_STAFF | 指派審查人員、管理流程狀態 |
| SYSTEM_ADMIN | 全系統管理 |
| CLIENT | 委託人，查看委託計畫（唯讀） |

---

## 3. 核心資料模型

| 資料表 | 說明 |
|--------|------|
| `protocols` | 計畫書主檔，含狀態與 `working_content` (JSONB) |
| `protocol_versions` | 不可變版本快照 |
| `protocol_status_history` | 狀態轉移歷程 |
| `protocol_activities` | 計畫活動紀錄（誰/何時/何事） |
| `review_assignments` | 審查人員指派 |
| `review_comments` | 審查意見（含 `parent_id` 支援巢狀回覆） |
| `review_comment_drafts` | 審查意見草稿 |
| `protocol_attachments` | 計畫附件 |
| `amendments` | 變更申請（含 `content`、`previous_content` JSONB） |
| `amendment_versions` | 變更版本快照 |
| `system_settings` | 系統設定 |

---

## 4. 計畫書狀態機

### 4.1 狀態定義

| 狀態 | 說明 |
|------|------|
| `DRAFT` | 草稿 |
| `SUBMITTED` | 已提交 |
| `PRE_REVIEW` | 行政預審 |
| `VET_REVIEW` | 獸醫審查 |
| `UNDER_REVIEW` | 審查中（委員會） |
| `REVISION_REQUIRED` | 需修訂 |
| `RESUBMITTED` | 已重送 |
| `APPROVED` | 核准 |
| `APPROVED_WITH_CONDITIONS` | 附條件核准 |
| `DEFERRED` | 延後審議 |
| `REJECTED` | 否決 |
| `SUSPENDED` | 暫停 |
| `CLOSED` | 結案 |
| `DELETED` | 已刪除（軟刪除，UI 不顯示） |

### 4.2 狀態轉移

```
DRAFT → SUBMITTED (PI)
  ↓
SUBMITTED → PRE_REVIEW (IACUC_STAFF)
  ↓
PRE_REVIEW → VET_REVIEW (IACUC_STAFF)
  ↓
VET_REVIEW → UNDER_REVIEW (VET 完成審查)
  ↓
UNDER_REVIEW → REVISION_REQUIRED (REVIEWER, CHAIR)
             → APPROVED / REJECTED (CHAIR)
             → APPROVED_WITH_CONDITIONS (CHAIR)
             → DEFERRED (CHAIR)
  ↓
REVISION_REQUIRED → RESUBMITTED (PI)
  ↓
RESUBMITTED → PRE_REVIEW / UNDER_REVIEW

APPROVED → SUSPENDED / CLOSED
DRAFT → DELETED (PI, 軟刪除)
```

**多輪往返**：PI 回應修訂意見後重送，審查人員可再次要求修訂，形成多輪審查。

---

## 5. 變更申請 (Amendment)

計畫核准後的變更管理，分為**重大變更**與**次要變更**兩種。

### 5.1 流程

```
PI 提出變更 → 草稿 → 提交 → (IACUC審查) → 核准/駁回
```

### 5.2 API

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/amendments` | 變更列表 |
| POST | `/amendments` | 建立變更 |
| GET | `/amendments/:id` | 變更詳情 |
| PUT | `/amendments/:id` | 更新變更 |
| POST | `/amendments/:id/submit` | 提交變更 |
| POST | `/amendments/:id/approve` | 核准變更 |
| POST | `/amendments/:id/reject` | 駁回變更 |
| GET | `/amendments/:id/versions` | 版本列表 |

---

## 6. 手寫電子簽章（計劃審查）

IACUC 委員可透過手寫簽名方式核准計畫：

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/signatures/protocol/:id` | 手寫簽署計劃審查 |
| GET | `/signatures/protocol/:id/status` | 取得簽章狀態 |

前端 `SectionSignature.tsx` 支援手寫簽名與上傳兩種模式並存。

---

## 7. API 端點

### 7.1 計畫書管理

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/protocols` | 建立計畫書 |
| GET | `/protocols` | 列表查詢（支援排序） |
| GET | `/protocols/:id` | 取得單一計畫書 |
| PATCH | `/protocols/:id` | 更新內容 |
| POST | `/protocols/:id/submit` | 提交審查 |
| POST | `/protocols/:id/status` | 變更狀態 |
| DELETE | `/protocols/:id` | 刪除草稿（軟刪除） |

### 7.2 版本管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/protocols/:id/versions` | 版本列表 |
| GET | `/protocols/:id/versions/:vid` | 特定版本內容 |
| GET | `/protocols/:id/status-history` | 狀態歷程 |
| GET | `/protocols/:id/activities` | 活動日誌 |

### 7.3 審查流程

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/reviews/assignments` | 指派審查人員 |
| GET | `/reviews/assignments` | 查詢指派列表 |
| POST | `/reviews/comments` | 新增審查意見 |
| GET | `/reviews/comments` | 查詢意見列表 |
| POST | `/reviews/comments/:id/resolve` | 解決意見 |
| POST | `/reviews/comments/reply` | 回覆意見 |

### 7.4 附件管理

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/attachments` | 上傳附件 |
| GET | `/attachments` | 查詢附件 |
| GET | `/attachments/:id/download` | 下載附件 |
| DELETE | `/attachments/:id` | 刪除附件 |

### 7.5 我的計劃（外部使用者）

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/my-projects` | PI/CLIENT 的計劃列表 |
| GET | `/my-projects/:id` | 計劃詳情 |

### 7.6 PDF 匯出

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/protocols/:id/export/pdf` | 匯出 AUP 計畫書 PDF |

---

## 8. AUP 表單結構

計畫書草稿儲存於 `protocols.working_content` 欄位（JSONB），包含 9 個章節：

1. **基本資料** - GLP、PI、計畫名稱、期間
2. **3Rs 原則說明** - Replacement, Reduction, Refinement
3. **試驗物質與對照組**
4. **試驗流程與麻醉止痛**
5. **參考文獻**
6. **手術計畫**
7. **動物資訊** - 種類、性別、數量、年齡、體重、來源
8. **人員名單與職責**（含個人資歷解析）
9. **流程圖與附件**

---

## 9. 前端路由

| 路由 | 頁面 |
|------|------|
| `/protocols` | 計畫書列表 |
| `/protocols/new` | 建立計畫書 |
| `/protocols/:id` | 檢視/編輯計畫書（多 Tab：申請表、審查、附件、歷程） |
| `/my-projects` | 我的計劃（PI/CLIENT） |
| `/my-projects/:id` | 計劃詳情 |

---

## 10. 相關文件

- [權限控制](../06_PERMISSIONS_RBAC.md) - 角色權限詳細說明
- [稽核日誌](../guides/AUDIT_LOGGING.md) - 變更追蹤
- [通知系統](./NOTIFICATION_SYSTEM.md) - 審查通知
- [動物管理](./ANIMAL_MANAGEMENT.md) - 計劃與動物的關聯
