# 舊資料庫 Dump 還原指南

> **用途**：從 `old_ipig.dump` 還原舊系統的資料庫到新系統
> **適用場景**：資料遷移、系統升級、災難復原

---

## 快速開始

### 前置條件

1. **Docker 環境運行中**
   ```powershell
   docker compose up -d
   ```

2. **`.env` 檔案已設定**
   - 至少需要：`POSTGRES_USER`、`POSTGRES_PASSWORD`、`POSTGRES_DB`
   - 建議從 `backend/env.sample` 複製並修改

3. **`old_ipig.dump` 檔案存在**
   - 預設位置：專案根目錄的 `old_ipig.dump`
   - 或使用 `-DumpPath` 參數指定路徑

### 基本用法

```powershell
# 標準還原（會提示備份和確認）
.\scripts\restore_old_dump.ps1

# 指定 dump 檔案路徑
.\scripts\restore_old_dump.ps1 -DumpPath "C:\backups\old_ipig.dump"

# 僅還原資料（不包含結構，需要資料庫結構已存在）
.\scripts\restore_old_dump.ps1 -DataOnly

# 跳過備份步驟
.\scripts\restore_old_dump.ps1 -SkipBackup

# 跳過 migration 同步處理
.\scripts\restore_old_dump.ps1 -SkipMigrationSync
```

---

## 腳本功能說明

### 1. 環境檢查
- ✅ 檢查 Docker 是否運行
- ✅ 檢查資料庫容器是否存在
- ✅ 檢查 dump 檔案是否存在

### 2. Dump 檔案驗證
- ✅ 檢查 dump 檔案格式（PostgreSQL custom format）
- ✅ 檢查是否包含 `_sqlx_migrations` 表
- ✅ 檢查是否包含資料

### 3. 資料庫備份（可選）
- ✅ 自動檢測現有資料庫
- ✅ 提示是否備份現有資料
- ✅ 備份檔案命名：`backup_before_restore_YYYYMMDD_HHMMSS.dump`

### 4. 安全還原
- ✅ 完整還原模式：`--clean --if-exists`（刪除現有物件後還原）
- ✅ 僅資料模式：`--data-only`（只還原資料，不改變結構）
- ✅ 自動處理所有權限設定（`--no-owner --no-acl`）

### 5. Migration 追蹤同步
- ✅ 自動檢測 `_sqlx_migrations` 表
- ✅ 比對 migration 記錄數與檔案數
- ✅ 提供三種處理選項：
  1. **保留 dump 中的記錄**（預設）
  2. **清空記錄**（讓系統重新執行 migrations）
  3. **手動同步**（使用 `sync_migrations.ps1 -Method FixChecksums`）

### 6. 驗證與清理
- ✅ 驗證還原後的資料表數量
- ✅ 驗證使用者數量
- ✅ 自動清理容器內的暫存檔案

---

## 重要注意事項

### ⚠️ 資料覆蓋警告

**完整還原模式會刪除現有資料庫的所有物件！**

- 使用 `--clean` 參數會執行 `DROP` 操作
- 建議在還原前先備份現有資料庫
- 腳本會自動提示是否備份

### 🔄 Migration 追蹤問題

如果 dump 包含 `_sqlx_migrations` 表，可能會遇到以下情況：

#### 情況 1：Migration 記錄與檔案不匹配

**現象**：系統啟動時出現 `VersionMismatch` 或 checksum 錯誤

**解決方案**：
```powershell
# 選項 A：清空 migration 記錄，讓系統重新執行
.\scripts\restore_old_dump.ps1
# 在提示時選擇選項 2

# 選項 B：手動同步 checksums
.\scripts\sync_migrations.ps1 -Method FixChecksums
```

#### 情況 2：Dump 中的 migration 版本與當前不一致

**現象**：Dump 來自舊版本，migration 檔案已更新

**解決方案**：
1. 先還原 dump（保留 migration 記錄）
2. 檢查哪些 migrations 需要更新
3. 使用 `sqlx migrate resolve` 或 `sync_migrations.ps1 -Method FixChecksums` 同步

### 📋 還原後檢查清單

1. **檢查資料表**
   ```powershell
   docker exec ipig-db psql -U postgres -d ipig_db -c "\dt"
   ```

2. **檢查資料量**
   ```powershell
   docker exec ipig-db psql -U postgres -d ipig_db -c "SELECT COUNT(*) FROM users;"
   ```

3. **檢查 migration 狀態**
   ```powershell
   docker exec ipig-db psql -U postgres -d ipig_db -c "SELECT * FROM _sqlx_migrations ORDER BY version;"
   ```

4. **啟動系統並檢查日誌**
   ```powershell
   docker compose up -d
   docker compose logs -f api
   ```

---

## 常見問題

### Q1: 還原後系統啟動失敗，出現 migration 錯誤

**A**: 這是因為 migration checksum 不匹配。解決方法：
```powershell
# 方法 1：清空 migration 記錄（推薦，如果資料庫結構已完整）
docker exec ipig-db psql -U postgres -d ipig_db -c "TRUNCATE TABLE _sqlx_migrations;"
docker compose restart api

# 方法 2：同步 checksums
.\scripts\sync_migrations.ps1 -Method FixChecksums
```

### Q2: 只想還原資料，不想改變結構

**A**: 使用 `-DataOnly` 參數：
```powershell
# 先確保資料庫結構已存在（執行 migrations）
docker compose up -d
# 等待 migrations 完成後，只還原資料
.\scripts\restore_old_dump.ps1 -DataOnly
```

### Q3: Dump 檔案很大，還原很慢

**A**: 這是正常的。可以：
- 監控還原進度：`docker compose logs -f db`
- 檢查容器資源：`docker stats ipig-db`
- 如果中斷，可以重新執行腳本（會先清理再還原）

### Q4: 還原後某些功能不正常

**A**: 可能原因：
1. **結構差異**：舊 dump 的結構與新系統不一致
   - 解決：先執行 migrations 建立新結構，再只還原資料
2. **資料格式變更**：某些欄位格式已改變
   - 解決：需要手動資料遷移腳本
3. **外鍵約束**：還原順序問題
   - 解決：重新執行腳本，`pg_restore` 會自動處理

---

## 進階用法

### 組合參數

```powershell
# 完整還原，跳過備份和 migration 同步
.\scripts\restore_old_dump.ps1 -SkipBackup -SkipMigrationSync

# 僅資料還原，跳過備份
.\scripts\restore_old_dump.ps1 -DataOnly -SkipBackup
```

### 手動還原（不使用腳本）

如果需要更多控制，可以手動執行：

```powershell
# 1. 複製 dump 到容器
docker cp old_ipig.dump ipig-db:/tmp/old_ipig.dump

# 2. 檢查 dump 內容
docker exec ipig-db pg_restore --list /tmp/old_ipig.dump

# 3. 還原（完整）
docker exec ipig-db pg_restore -U postgres -d ipig_db --clean --if-exists --no-owner --no-acl /tmp/old_ipig.dump

# 或僅資料
docker exec ipig-db pg_restore -U postgres -d ipig_db --data-only --no-owner --no-acl /tmp/old_ipig.dump

# 4. 清理
docker exec ipig-db rm -f /tmp/old_ipig.dump
```

---

## 相關文件

- [資料庫綱要文件](Profiling_Spec/04_DATABASE_SCHEMA.md)
- [Migration 管理說明](INTEGRATION_TESTS.md#常見錯誤-versionmismatch1)
- [備份還原流程](backup/BACKUP.md)

---

## 腳本參數說明

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `-DumpPath` | Dump 檔案路徑 | `old_ipig.dump` |
| `-SkipBackup` | 跳過備份現有資料庫 | `false` |
| `-SkipMigrationSync` | 跳過 migration 同步處理 | `false` |
| `-DataOnly` | 僅還原資料（不包含結構） | `false` |

---

**最後更新**：2026-03-01
