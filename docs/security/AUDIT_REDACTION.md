# Audit Redaction 對照表

> **用途**：GLP 21 CFR Part 11 §11.10「Secure audit trail」合規文件。列出所有可能進入 audit log 的 entity，以及 `AuditRedact::redacted_fields()` 是否有覆寫敏感欄位。
>
> **最後檢視**：2026-04-24（R26 epic 收尾期間）

## 1. 敏感欄位 redact 清單（明確 redact 的 entity）

| Entity | 位置 | Redacted Fields |
|--------|------|-----------------|
| `User` | `models/user.rs:41` | `password_hash`, `totp_secret_encrypted`, `totp_backup_codes` |
| `AiApiKey` | `models/ai.rs:29` | `key_hash` |

## 2. Default empty impl（無敏感欄位，全欄位明碼可存）

經 code review 確認以下 entity 無敏感欄位，`redacted_fields()` 採 default 空陣列：

### 動物管理
- `Animal`, `AnimalSource`, `AnimalObservation`, `AnimalSurgery`, `AnimalWeight`
- `AnimalVaccination`, `AnimalSacrifice`, `AnimalSuddenDeath`
- `AnimalTransfer`, `TransferVetEvaluation`, `AnimalPathologyReport`
- `CareRecord`, `VetAdviceRecord`, `VetPatrolReport`, `AnimalVetAdvice`

### ERP / 儲位
- `Product`, `ProductCategory`
- `Partner`（業務資料：名稱、電話、email，非機密）
- `StorageLocationInventoryItem`（庫存數量、批號、效期）

### 設備管理
- `Equipment`, `EquipmentMaintenanceRecord`

### 文件 / 單據
- `Document`, `DocumentLine`, `DocumentAuditSnapshot`

### 人資
- `AttendanceRecord`, `OvertimeRecord`
- `LeaveRequest`, `LeaveApproval`
- `AnnualLeaveEntitlement`, `CompTimeBalance`

### 權限 / 角色
- `Role`, `RolePermissionSnapshot`, `RoleAssignmentSnapshot`

## 3. Never in audit diff（不會進入 audit trail）

以下 entity 含敏感欄位但從未傳入 `DataDiff::compute`，因此不需 impl `AuditRedact`：

| Entity | 敏感欄位 | 為何不 audit |
|--------|----------|-------------|
| `UserSession` | `refresh_token_id`（UUID，非真正 token） | Session 狀態由 `USER_LOGIN/LOGOUT` 事件記錄，不走 DataDiff |

## 4. FullPlan DoD-7 列舉但實際不存在的 entity

以下在 `R26_FullPlan.md` DoD-7 列出，但 codebase 中**從未定義**（可能為早期規劃後改用不同架構實作）：

- `TwoFactorSecret` — 2FA 邏輯嵌入 `User.totp_secret_encrypted`
- `JwtBlacklist` — 只是 middleware cache，無 DB entity
- `OAuthCredential` — OAuth 整合改用 service account 檔案，無 DB entity
- `McpKey` — 實際名稱為 `AiApiKey`（已 redact `key_hash`）

## 5. CI 守衛（防未來遺漏）

`.github/workflows/ci.yml` 的 `audit-redaction-guard` job：
- 掃描 `models/**/*.rs` 中含敏感欄位 pattern（`password_hash`, `_secret`, `_token`, `api_key`, `backup_codes`）的 struct
- 若該 struct 傳入 `DataDiff::compute`（grep 驗證）且 `AuditRedact::redacted_fields()` 返回空陣列，CI fail
- 目的：強制新增敏感 entity 時必須明確覆寫 `redacted_fields()`

## 6. 維護記錄

- **2026-04-24**：首次撰寫。R26 epic critical review 確認所有 entity（80 處 `DataDiff::compute` 呼叫點）已有 `AuditRedact` impl。原 review 擔憂的 Equipment / Document / Partner / Role / Animal 實際都已 impl。
- **未來新增 entity 時**：
  1. 若 entity 含密碼/token/secret/key → 明確 `impl AuditRedact for X { fn redacted_fields() -> &'static [&'static str] { &["field_name"] } }`
  2. 若 entity 純業務資料 → 空 impl + 在本文檔 §2 登記
  3. 若 entity 不進入 audit → 在本文檔 §3 說明
