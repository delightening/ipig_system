# 除錯 422 錯誤指南

## 問題描述

GET 請求 `/api/pigs/2/observations` 返回 422 (Unprocessable Entity) 錯誤。

## 檢查步驟

### 1. 檢查瀏覽器控制台錯誤資訊

在瀏覽器中：
1. 打開開發者工具 (F12)
2. 切換到 "Console" 標籤
3. 刷新頁面或觸發請求
4. 查找錯誤資訊，應該會顯示：
   - `Failed to load observations:`
   - `Error response:` (完整的錯誤回應體)
   - `Error status:` (HTTP 狀態碼)
   - `Full error:` (完整的錯誤物件)

錯誤回應格式應該類似：
```json
{
  "error": {
    "message": "錯誤資訊",
    "code": 422
  }
}
```

**請複製完整的錯誤資訊，特別是 `error.message` 欄位。**

### 2. 檢查資料庫遷移狀態

運行遷移檢查腳本：

```powershell
cd backend
.\check_migrations.ps1
```

腳本會：
- 顯示所有已應用的遷移
- 檢查遷移 004 是否已應用
- 檢查 `pig_observations` 表是否有 `deleted_at` 和 `copied_from_id` 欄位

如果遷移 004 未應用或欄位缺失，需要：
1. 停止後端服務
2. 如果遷移記錄存在但欄位缺失，刪除遷移記錄：
   ```sql
   DELETE FROM _sqlx_migrations WHERE version = 4;
   ```
3. 重新啟動後端服務（會自動運行遷移）

### 3. 檢查伺服器日誌

查看後端伺服器的日誌輸出，查找：
- `[Database] Running migrations...` - 遷移開始
- `[Database] ✓ Migrations completed successfully` - 遷移成功
- `[Database] ❌ FAILED` - 遷移失敗（如果有）
- `Database error` - 資料庫錯誤（如果有）
- `Business rule violation` - 業務規則錯誤（422）

**如果看到遷移錯誤，請複製完整的錯誤資訊。**

### 4. 手動檢查資料庫表結構

如果可以使用 PostgreSQL 客戶端：

```sql
-- 檢查遷移記錄
SELECT version, name, applied_at 
FROM _sqlx_migrations 
ORDER BY version;

-- 檢查 pig_observations 表結構
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'pig_observations'
ORDER BY ordinal_position;

-- 檢查是否缺少關鍵欄位
SELECT 
  CASE WHEN EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'pig_observations' AND column_name = 'deleted_at'
  ) THEN '存在' ELSE '缺失' END as deleted_at_status,
  CASE WHEN EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'pig_observations' AND column_name = 'copied_from_id'
  ) THEN '存在' ELSE '缺失' END as copied_from_id_status;
```

### 5. 測試 API 端點

使用 curl 或 Postman 測試 API：

```bash
# 取得存取權杖（需要先登入）
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"your_email","password":"your_password"}'

# 使用權杖測試 API
curl -X GET http://localhost:8080/api/pigs/2/observations \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

## 可能的原因

1. **資料庫遷移未運行**
   - 遷移 004 添加了 `deleted_at` 和 `copied_from_id` 欄位
   - 如果這些欄位不存在，查詢會失敗

2. **資料庫連線問題**
   - 檢查 DATABASE_URL 配置
   - 檢查資料庫服務是否運行

3. **業務規則錯誤**
   - 422 錯誤在程式碼中對應 `AppError::BusinessRule`
   - 但 `list_pig_observations` 函數沒有使用 BusinessRule
   - 可能是錯誤被轉換了

4. **資料序列化問題**
   - 資料庫查詢成功，但序列化為 JSON 時失敗
   - 檢查 `PigObservation` 結構體與資料庫欄位是否匹配

## 下一步

根據檢查結果：

1. **如果遷移 004 未應用**：重新運行遷移
2. **如果欄位缺失**：手動添加欄位或重新運行遷移
3. **如果看到具體的錯誤消息**：根據錯誤消息進行相應的修復
4. **如果仍然無法解決**：請提供：
   - 瀏覽器控制台的完整錯誤資訊
   - 伺服器日誌的相關部分
   - 遷移狀態檢查的結果
