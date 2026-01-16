# Co-Editor 指派和權限修復實施總結

## ✅ 已完成的工作

### 1. **Migration 035 - 權限分配和 CO_EDITOR 支援**

📁 `backend/migrations/035_assign_pi_and_client_permissions.sql`

**完成內容**：
- ✅ 擴展 `protocol_role` enum 支援 `CO_EDITOR`
- ✅ 為 PI 角色分配完整的 AUP 協議權限
- ✅ 為 CLIENT 角色分配 AUP 協議查看權限
- ✅ 為 PI 角色分配豬隻管理權限
- ✅ 為 CLIENT 角色分配豬隻管理查看權限

### 2. **Rust 模型更新**

📁 `backend/src/models/protocol.rs`

**完成內容**：
- ✅ 新增 `CoEditor` 到 `ProtocolRole` enum
- ✅ 新增 `AssignCoEditorRequest` 結構

### 3. **Co-Editor 指派 API**

📁 `backend/src/services/protocol.rs`
📁 `backend/src/handlers/protocol.rs`
📁 `backend/src/routes.rs`

**完成內容**：
- ✅ 實現 `assign_co_editor` 服務方法
  - 驗證協議存在
  - 驗證用戶是 EXPERIMENT_STAFF 角色
  - 在 `user_protocols` 表中指派為 CO_EDITOR
- ✅ 實現 `assign_co_editor` handler
  - 需要 `aup.review.assign` 權限（IACUC_STAFF 可用）
- ✅ 新增路由：`POST /api/protocols/:id/co-editors`

### 4. **Co-Editor 權限檢查**

📁 `backend/src/handlers/protocol.rs`

**完成內容**：
- ✅ `get_protocol` handler：允許 co-editor 查看協議
- ✅ `update_protocol` handler：允許 co-editor 編輯協議
  - 檢查 `user_protocols` 表確認用戶是 PI、CLIENT 或 CO_EDITOR

### 5. **委託單位主管特殊權限**

📁 `backend/src/services/protocol.rs`

**完成內容**：
- ✅ 修改 `get_my_protocols` 方法
  - CLIENT 角色用戶如果有 `organization` 欄位
  - 可查看同組織下所有用戶的協議（委託單位主管權限）

---

## 📋 API 端點說明

### 指派 Co-Editor

**端點**：`POST /api/protocols/:id/co-editors`

**請求體**：
```json
{
  "protocol_id": "uuid",
  "user_id": "uuid"
}
```

**權限要求**：`aup.review.assign` (IACUC_STAFF 擁有)

**說明**：
- IACUC_STAFF 可以指派 EXPERIMENT_STAFF 為協議的 co-editor
- 被指派的用戶必須有 EXPERIMENT_STAFF 角色
- 指派後，該用戶可以查看和編輯該協議（當協議處於草稿或需修訂狀態時）

---

## 🚀 執行 Migration

Migration 035 會在 API 啟動時自動執行（`main.rs` 中的自動 migration 功能）。

**執行方式**：
1. **自動執行**（推薦）：
   ```bash
   # 啟動 API 服務，migrations 會自動執行
   cd backend
   cargo run
   ```

2. **手動執行**（如果需要）：
   ```bash
   # 使用 SQLx CLI
   cd backend
   sqlx migrate run --database-url "postgres://user:password@host:port/dbname"
   ```

---

## ⚠️ 注意事項

1. **Co-Editor 權限範圍**：
   - Co-editor 可以查看和編輯協議內容
   - Co-editor **不能**提交協議（只有 PI 可以）
   - Co-editor **不能**刪除協議（只有 PI 可以）

2. **委託單位主管權限**：
   - 目前實現是：所有有 CLIENT 角色且有 `organization` 欄位的用戶，都可以查看同組織下所有用戶的協議
   - 如果需要更精細的控制（例如只有特定用戶是主管），需要添加 `is_manager` 或類似的欄位

3. **權限檢查邏輯**：
   - Co-editor 的權限檢查是基於 `user_protocols` 表
   - 需要確保在查詢和編輯協議時，正確檢查 `user_protocols` 表中的角色

---

## 📝 後續建議

### 1. 前端實現
- [ ] 在協議管理頁面添加「指派 Co-Editor」功能
- [ ] 顯示協議的 co-editor 列表
- [ ] 允許移除 co-editor

### 2. 權限優化
- [ ] 如果需要更精細的主管控制，考慮添加 `is_manager` 欄位到 `users` 表
- [ ] 添加 API 端點列出協議的所有 co-editor

### 3. 通知功能
- [ ] 當被指派為 co-editor 時，發送通知給用戶
- [ ] 當協議被編輯時，通知 PI 和 co-editor

---

## 🎯 功能驗證

### 驗證 Co-Editor 指派

1. 以 IACUC_STAFF 角色登入
2. 使用 API 指派 EXPERIMENT_STAFF 為 co-editor：
   ```bash
   POST /api/protocols/{protocol_id}/co-editors
   {
     "protocol_id": "...",
     "user_id": "..."
   }
   ```
3. 以 EXPERIMENT_STAFF 登入
4. 驗證可以查看和編輯該協議

### 驗證委託單位主管權限

1. 創建兩個 CLIENT 用戶，同一個 `organization`
2. 創建協議，PI 是不同的用戶
3. 以 CLIENT 用戶登入
4. 驗證可以看到同組織下所有用戶的協議

---

## 📚 相關文件

- Migration 文件：`backend/migrations/035_assign_pi_and_client_permissions.sql`
- 權限檢查報告：`backend/PERMISSION_STATUS_REPORT.md`
- 路由定義：`backend/src/routes.rs`
- 服務層：`backend/src/services/protocol.rs`
- Handler 層：`backend/src/handlers/protocol.rs`
