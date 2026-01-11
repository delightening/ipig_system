# 修復遷移檔案 009 的問題

## 問題說明

遷移檔案 `009_reorganize_permissions.sql` 已經在資料庫中執行過，但後來被修改了。SQLx 檢測到檔案內容變更，因此報錯：
```
Error: migration 9 was previously applied but has been modified
```

## 解決方案

### 方法 1：手動刪除遷移記錄（推薦）

連接到資料庫並刪除 009 的遷移記錄：

**使用 Docker：**
```bash
docker exec -it ipig-db psql -U postgres -d ipig_db -c "DELETE FROM _sqlx_migrations WHERE version = 9;"
```

**使用 psql 直接連接：**
```bash
psql -U postgres -d ipig_db -c "DELETE FROM _sqlx_migrations WHERE version = 9;"
```

**使用 SQLx CLI：**
```bash
sqlx migrate revert --database-url <DATABASE_URL>
```

### 方法 2：重置所有遷移（僅開發環境）

⚠️ **警告：這會刪除所有資料！僅在開發環境使用！**

```bash
# 刪除所有遷移記錄
docker exec -it ipig-db psql -U postgres -d ipig_db -c "DELETE FROM _sqlx_migrations;"

# 重新執行所有遷移
# 應用程式啟動時會自動執行
```

## 執行步驟

1. **停止應用程式**（如果正在運行）

2. **刪除 009 的遷移記錄**：
   ```bash
   docker exec -it ipig-db psql -U postgres -d ipig_db -c "DELETE FROM _sqlx_migrations WHERE version = 9;"
   ```

3. **重新啟動應用程式**：
   - 新的遷移檔案 `011_reorganize_permissions.sql` 將會被執行
   - 遷移檔案 `010_assign_iacuc_staff_permissions.sql` 也會被執行

## 注意事項

- 009 檔案已經被刪除，不會再造成衝突
- 011 檔案包含了 009 的所有內容，會重新組織權限
- 010 檔案會為 IACUC_STAFF 角色分配必要的權限
- 執行完修復後，秘書角色就能看到所有計畫書了
