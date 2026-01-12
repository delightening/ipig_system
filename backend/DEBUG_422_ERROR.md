# 调试 422 错误指南

## 问题描述

GET 请求 `/api/pigs/2/observations` 返回 422 (Unprocessable Entity) 错误。

## 检查步骤

### 1. 检查浏览器控制台错误信息

在浏览器中：
1. 打开开发者工具 (F12)
2. 切换到 "Console" 标签
3. 刷新页面或触发请求
4. 查找错误信息，应该会显示：
   - `Failed to load observations:`
   - `Error response:` (完整的错误响应体)
   - `Error status:` (HTTP 状态码)
   - `Full error:` (完整的错误对象)

错误响应格式应该类似：
```json
{
  "error": {
    "message": "错误信息",
    "code": 422
  }
}
```

**请复制完整的错误信息，特别是 `error.message` 字段。**

### 2. 检查数据库迁移状态

运行迁移检查脚本：

```powershell
cd backend
.\check_migrations.ps1
```

脚本会：
- 显示所有已应用的迁移
- 检查迁移 004 是否已应用
- 检查 `pig_observations` 表是否有 `deleted_at` 和 `copied_from_id` 字段

如果迁移 004 未应用或字段缺失，需要：
1. 停止后端服务
2. 如果迁移记录存在但字段缺失，删除迁移记录：
   ```sql
   DELETE FROM _sqlx_migrations WHERE version = 4;
   ```
3. 重新启动后端服务（会自动运行迁移）

### 3. 检查服务器日志

查看后端服务器的日志输出，查找：
- `[Database] Running migrations...` - 迁移开始
- `[Database] ✓ Migrations completed successfully` - 迁移成功
- `[Database] ❌ FAILED` - 迁移失败（如果有）
- `Database error` - 数据库错误（如果有）
- `Business rule violation` - 业务规则错误（422）

**如果看到迁移错误，请复制完整的错误信息。**

### 4. 手动检查数据库表结构

如果可以使用 PostgreSQL 客户端：

```sql
-- 检查迁移记录
SELECT version, name, applied_at 
FROM _sqlx_migrations 
ORDER BY version;

-- 检查 pig_observations 表结构
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'pig_observations'
ORDER BY ordinal_position;

-- 检查是否缺少关键字段
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

### 5. 测试 API 端点

使用 curl 或 Postman 测试 API：

```bash
# 获取访问令牌（需要先登录）
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"your_email","password":"your_password"}'

# 使用令牌测试 API
curl -X GET http://localhost:8080/api/pigs/2/observations \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

## 可能的原因

1. **数据库迁移未运行**
   - 迁移 004 添加了 `deleted_at` 和 `copied_from_id` 字段
   - 如果这些字段不存在，查询会失败

2. **数据库连接问题**
   - 检查 DATABASE_URL 配置
   - 检查数据库服务是否运行

3. **业务规则错误**
   - 422 错误在代码中对应 `AppError::BusinessRule`
   - 但 `list_pig_observations` 函数没有使用 BusinessRule
   - 可能是错误被转换了

4. **数据序列化问题**
   - 数据库查询成功，但序列化为 JSON 时失败
   - 检查 `PigObservation` 结构体与数据库字段是否匹配

## 下一步

根据检查结果：

1. **如果迁移 004 未应用**：重新运行迁移
2. **如果字段缺失**：手动添加字段或重新运行迁移
3. **如果看到具体的错误消息**：根据错误消息进行相应的修复
4. **如果仍然无法解决**：请提供：
   - 浏览器控制台的完整错误信息
   - 服务器日志的相关部分
   - 迁移状态检查的结果
