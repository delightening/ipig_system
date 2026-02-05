# Database Migrations

此目錄包含整合後的資料庫 migration 檔案。

## 檔案結構

| 檔案 | 大小 | 說明 |
|-----|------|------|
| `001_schema.sql` | ~122KB | 所有表結構、索引、觸發器、視圖 |
| `002_seed_data.sql` | ~33KB | ERP 基礎資料、使用者帳號、種子資料 |
| `003_permissions.sql` | ~16KB | 角色權限分配 |

## 執行順序

```bash
# 重置資料庫後，依序執行：
Get-Content migrations/001_schema.sql | docker exec -i ipig-db psql -U postgres -d ipig_db
Get-Content migrations/002_seed_data.sql | docker exec -i ipig-db psql -U postgres -d ipig_db
Get-Content migrations/003_permissions.sql | docker exec -i ipig-db psql -U postgres -d ipig_db
```

## 原始檔案

原始 33+ 個 migration 檔案保留在 `archived_migrations/` 供歷史參考。

## 注意事項

- 這些是**開發環境專用**的整合 migration
- 生產環境應使用原始的增量 migration
- 執行前需確保資料庫為空或可重置
