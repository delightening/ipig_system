# 各角色權限狀態檢查報告

## 需求摘要

根據用戶需求，以下為各角色應具備的權限：

### 1. IACUC_STAFF (執行秘書)
- ✅ **保存草稿時即可查看計畫內容**
- ✅ **可指派 EXPERIMENT_STAFF 為 co-editor**

### 2. PI (計畫主持人) 與 CLIENT (委託人)
應具備以下權限：
- ✅ 撰寫計畫、修改、提交
- ✅ 可見計畫內之實驗動物資料、紀錄
- ✅ 上傳附件、下載附件、刪除附件、查看附件
- ✅ 刪除計畫、建立計畫、查看自己的計畫、提交計畫、編輯計畫
- ✅ 查看審查意見
- ✅ 查看計畫書版本歷史、還原版本
- ✅ 匯出計畫內豬隻的手術紀錄、所有資料、病歷、試驗紀錄與觀察紀錄
- ✅ 查看計畫內豬隻病理報告
- ✅ 查看計畫內豬隻、匯出豬隻資料
- ✅ 查看計畫內豬隻的手術、疫苗、犧牲、體重、觀察等紀錄

### 3. CLIENT (委託人) 特殊權限
- ✅ **委託單位主管（委託人）可見其轄下所有人員之計畫與豬隻**

### 4. EXPERIMENT_STAFF (試驗工作人員)
- ✅ **可被指派為 co-editor**
- ✅ **協助計畫主持人、委託人撰寫計畫**

---

## 當前權限狀態檢查

### ✅ IACUC_STAFF (執行秘書)

#### AUP 協議相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| `aup.protocol.view_all` | ✅ | 查看所有計畫 |
| `aup.protocol.view_own` | ✅ | 查看自己的計畫 |
| `aup.protocol.create` | ✅ | 建立計畫 |
| `aup.protocol.edit` | ✅ | 編輯計畫 |
| `aup.protocol.submit` | ✅ | 提交計畫 |
| `aup.protocol.change_status` | ✅ | 變更狀態 |
| `aup.review.view` | ✅ | 查看審查意見 |
| `aup.review.assign` | ✅ | 指派審查人員 |
| `aup.attachment.upload` | ✅ | 上傳附件 |
| `aup.attachment.view` | ✅ | 查看附件 |
| `aup.attachment.download` | ✅ | 下載附件 |
| `aup.attachment.delete` | ✅ | 刪除附件 |
| `aup.version.view` | ✅ | 查看版本歷史 |
| `aup.version.restore` | ✅ | 還原版本 |

**結論**：IACUC_STAFF 具備完整的 AUP 權限 ✅

#### ⚠️ 缺失功能
- ❌ **無法指派 EXPERIMENT_STAFF 為 co-editor**
  - 原因：`protocol_role` enum 只有 `PI` 和 `CLIENT`，沒有 `CO_EDITOR`
  - 原因：沒有指派 co-editor 的 API 功能

---

### ❌ PI (計畫主持人)

#### AUP 協議相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| `aup.protocol.view_own` | ❌ | **未分配** |
| `aup.protocol.create` | ❌ | **未分配** |
| `aup.protocol.edit` | ❌ | **未分配** |
| `aup.protocol.delete` | ❌ | **未分配** |
| `aup.protocol.submit` | ❌ | **未分配** |
| `aup.review.view` | ❌ | **未分配** |
| `aup.attachment.upload` | ❌ | **未分配** |
| `aup.attachment.view` | ❌ | **未分配** |
| `aup.attachment.download` | ❌ | **未分配** |
| `aup.attachment.delete` | ❌ | **未分配** |
| `aup.version.view` | ❌ | **未分配** |
| `aup.version.restore` | ❌ | **未分配** |

#### 豬隻管理相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| `pig.pig.view_project` | ❌ | **未分配** |
| `pig.pig.export` | ❌ | **未分配** |
| `pig.record.view` | ❌ | **未分配** |
| `pig.record.observation` | ❌ | **未分配** |
| `pig.record.surgery` | ❌ | **未分配** |
| `pig.record.weight` | ❌ | **未分配** |
| `pig.record.vaccine` | ❌ | **未分配** |
| `pig.record.sacrifice` | ❌ | **未分配** |
| `pig.export.medical` | ✅ | 已分配 |
| `pig.export.observation` | ✅ | 已分配 |
| `pig.export.surgery` | ✅ | 已分配 |
| `pig.export.experiment` | ❌ | **未分配** |
| `pig.pathology.view` | ❌ | **未分配** |

**結論**：PI 角色**缺少大部分 AUP 權限** ❌

---

### ❌ CLIENT (委託人)

#### AUP 協議相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| `aup.protocol.view_own` | ❌ | **未分配** |
| `aup.protocol.create` | ❌ | **未分配**（依需求應不能建立） |
| `aup.protocol.edit` | ❌ | **未分配** |
| `aup.protocol.delete` | ❌ | **未分配**（依需求應不能刪除） |
| `aup.protocol.submit` | ❌ | **未分配** |
| `aup.review.view` | ❌ | **未分配** |
| `aup.attachment.upload` | ❌ | **未分配** |
| `aup.attachment.view` | ❌ | **未分配** |
| `aup.attachment.download` | ❌ | **未分配** |
| `aup.attachment.delete` | ❌ | **未分配** |
| `aup.version.view` | ❌ | **未分配** |
| `aup.version.restore` | ❌ | **未分配** |

#### 豬隻管理相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| `pig.pig.view_project` | ❌ | **未分配** |
| `pig.pig.export` | ❌ | **未分配** |
| `pig.record.view` | ❌ | **未分配** |
| `pig.record.observation` | ❌ | **未分配** |
| `pig.record.surgery` | ❌ | **未分配** |
| `pig.record.weight` | ❌ | **未分配** |
| `pig.record.vaccine` | ❌ | **未分配** |
| `pig.record.sacrifice` | ❌ | **未分配** |
| `pig.export.medical` | ✅ | 已分配 |
| `pig.export.observation` | ✅ | 已分配 |
| `pig.export.surgery` | ✅ | 已分配 |
| `pig.export.experiment` | ❌ | **未分配** |
| `pig.pathology.view` | ❌ | **未分配** |

**結論**：CLIENT 角色**缺少大部分 AUP 權限** ❌

#### ⚠️ 特殊需求缺失
- ❌ **委託單位主管可見轄下所有人員之計畫與豬隻**
  - 原因：需要基於 `organization` 欄位的權限控制
  - 原因：需要特殊的資料範圍查詢邏輯

---

### ❌ EXPERIMENT_STAFF (試驗工作人員)

#### AUP 協議相關權限
| 權限 | 狀態 | 說明 |
|-----|:---:|------|
| AUP 相關權限 | ❌ | **完全沒有 AUP 權限** |

#### ⚠️ 缺失功能
- ❌ **無法被指派為 co-editor**
  - 原因：`protocol_role` enum 只有 `PI` 和 `CLIENT`，沒有 `CO_EDITOR`
  - 原因：沒有指派 co-editor 的 API 功能
- ❌ **無法協助撰寫計畫**
  - 原因：沒有協議編輯權限
  - 原因：無法查看協議內容

---

## 資料庫結構問題

### ❌ `protocol_role` enum 不支援 CO_EDITOR

**當前定義** (`backend/migrations/002_extend_schema.sql`):
```sql
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT');
```

**問題**：沒有 `CO_EDITOR` 選項

**影響**：
- 無法在 `user_protocols` 表中指派 EXPERIMENT_STAFF 為 co-editor
- 無法識別誰是某個協議的 co-editor

---

## 需要修復的問題清單

### 🔴 高優先級（必須修復）

1. **為 PI 角色分配 AUP 權限**
   - [ ] `aup.protocol.view_own`
   - [ ] `aup.protocol.create`
   - [ ] `aup.protocol.edit`
   - [ ] `aup.protocol.delete`
   - [ ] `aup.protocol.submit`
   - [ ] `aup.review.view`
   - [ ] `aup.attachment.upload`
   - [ ] `aup.attachment.view`
   - [ ] `aup.attachment.download`
   - [ ] `aup.attachment.delete`
   - [ ] `aup.version.view`
   - [ ] `aup.version.restore`

2. **為 PI 角色分配豬隻管理權限**
   - [ ] `pig.pig.view_project`
   - [ ] `pig.pig.export`
   - [ ] `pig.record.view`
   - [ ] `pig.record.observation`
   - [ ] `pig.record.surgery`
   - [ ] `pig.record.weight`
   - [ ] `pig.record.vaccine`
   - [ ] `pig.record.sacrifice`
   - [ ] `pig.export.experiment`
   - [ ] `pig.pathology.view`

3. **為 CLIENT 角色分配 AUP 權限**（僅查看與附件）
   - [ ] `aup.protocol.view_own`
   - [ ] `aup.review.view`
   - [ ] `aup.attachment.upload`
   - [ ] `aup.attachment.view`
   - [ ] `aup.attachment.download`
   - [ ] `aup.attachment.delete`
   - [ ] `aup.version.view`

4. **為 CLIENT 角色分配豬隻管理權限**（僅查看）
   - [ ] `pig.pig.view_project`
   - [ ] `pig.pig.export`
   - [ ] `pig.record.view`
   - [ ] `pig.export.experiment`
   - [ ] `pig.pathology.view`

5. **實現 Co-Editor 機制**
   - [ ] 擴展 `protocol_role` enum 支援 `CO_EDITOR`
   - [ ] 創建 migration 添加 `CO_EDITOR` 到 enum
   - [ ] 實現指派 co-editor 的 API
   - [ ] 為 EXPERIMENT_STAFF 分配協議編輯權限（當被指派為 co-editor 時）

### 🟡 中優先級（應該實現）

6. **委託單位主管特殊權限**
   - [ ] 實現基於 `organization` 的資料範圍查詢
   - [ ] 委託單位主管可查看轄下所有人員之計畫
   - [ ] 委託單位主管可查看轄下所有人員之豬隻

### 🟢 低優先級（可以後續優化）

7. **權限檢查邏輯優化**
   - [ ] 檢查草稿狀態下的權限控制
   - [ ] IACUC_STAFF 在保存草稿時即可查看（目前已有 `view_all`）

---

## 已完成的修復

✅ **Migration 035 已創建**: `035_assign_pi_and_client_permissions.sql`

### 修復內容：

1. ✅ **擴展 protocol_role enum 支援 CO_EDITOR**
   - 已新增 `CO_EDITOR` 值到 `protocol_role` enum
   - 已更新 Rust 模型 `ProtocolRole` enum

2. ✅ **為 PI 角色分配 AUP 協議權限**
   - `aup.protocol.view_own`, `create`, `edit`, `delete`, `submit`
   - `aup.review.view`
   - `aup.attachment.*` (upload, view, download, delete)
   - `aup.version.view`, `restore`

3. ✅ **為 CLIENT 角色分配 AUP 協議權限**（僅查看和附件）
   - `aup.protocol.view_own`
   - `aup.review.view`
   - `aup.attachment.*` (upload, view, download, delete)
   - `aup.version.view`

4. ✅ **為 PI 角色分配豬隻管理權限**
   - `pig.pig.view_project`, `export`
   - `pig.record.view`, `observation`, `surgery`, `weight`, `vaccine`, `sacrifice`
   - `pig.export.experiment`
   - `pig.pathology.view`

5. ✅ **為 CLIENT 角色分配豬隻管理權限**（僅查看）
   - `pig.pig.view_project`, `export`
   - `pig.record.view`
   - `pig.export.experiment`
   - `pig.pathology.view`

### 待實現的功能（應用程式邏輯層）：

⚠️ **以下功能需要在應用程式邏輯層實現**：

1. **Co-Editor 指派機制**
   - 創建 API 端點供 IACUC_STAFF 指派 EXPERIMENT_STAFF 為 co-editor
   - 更新 `user_protocols` 表，設定 `role_in_protocol = 'CO_EDITOR'`
   - 權限檢查邏輯：當 EXPERIMENT_STAFF 被指派為 co-editor 時，應允許編輯協議

2. **委託單位主管特殊權限**
   - 實現基於 `users.organization` 欄位的資料範圍查詢
   - CLIENT 角色中，如果用戶是主管，可查看同組織下所有用戶的計畫
   - 需要在前端或後端邏輯中識別「主管」身份（可能需要額外欄位）

---

## 參考資料

- 權限定義：`backend/migrations/011_reorganize_permissions.sql`
- 角色定義：`backend/migrations/002_extend_schema.sql`
- IACUC_STAFF 權限：`backend/migrations/010_assign_iacuc_staff_permissions.sql`
- 豬隻權限：`backend/migrations/004_pig_frontend_requirements.sql`
