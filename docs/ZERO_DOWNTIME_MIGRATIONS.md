# 零停機資料庫遷移策略

本文件規範 iPig System 在生產環境中執行資料庫 schema 變更時，如何避免服務中斷。

---

## 基本原則

1. **遷移必須向後相容** — 新版 schema 必須同時支援新舊兩個版本的應用程式。
2. **拆分為多步驟** — 破壞性變更應拆為 2–3 次部署完成。
3. **先擴展、後收縮** — 先新增欄位/表 → 部署新程式碼 → 再移除舊欄位。
4. **使用 `CONCURRENTLY`** — `CREATE INDEX` 一律使用 `CONCURRENTLY` 避免鎖表。
5. **避免長時間鎖定** — 不使用 `ALTER TABLE ... ALTER COLUMN TYPE` 搭配大量資料重寫。

---

## 安全的操作 (可直接執行)

| 操作 | 是否安全 | 備註 |
|------|---------|------|
| `CREATE TABLE` | ✅ | 新表不影響現有查詢 |
| `ADD COLUMN ... DEFAULT NULL` | ✅ | PG 11+ 不重寫表 |
| `ADD COLUMN ... DEFAULT <value>` | ✅ | PG 11+ 同上（虛擬預設值） |
| `CREATE INDEX CONCURRENTLY` | ✅ | 不鎖表，但耗時較長 |
| `DROP INDEX CONCURRENTLY` | ✅ | |
| `ALTER TABLE ADD CONSTRAINT ... NOT VALID` | ✅ | 僅對新資料生效 |
| `VALIDATE CONSTRAINT` | ✅ | 對現有資料進行驗證（`SHARE UPDATE EXCLUSIVE` 鎖） |

## 需要多步驟的操作

### 重新命名欄位

```
步驟 1: ALTER TABLE ADD COLUMN new_name ...;
         UPDATE ... SET new_name = old_name WHERE new_name IS NULL; -- 批次
步驟 2: 部署同時讀取 new_name 和 old_name 的程式碼
步驟 3: 停止寫入 old_name
步驟 4: ALTER TABLE DROP COLUMN old_name;
```

### 變更欄位型別

```
步驟 1: 新增 new_column 為新型別
步驟 2: 設定 trigger 同步寫入 old_column → new_column
步驟 3: 背景批次轉換歷史資料
步驟 4: 部署讀取 new_column 的程式碼
步驟 5: 移除 trigger 與 old_column
```

### 新增 NOT NULL 約束

```
步驟 1: 修改程式碼確保所有新寫入都提供非 NULL 值
步驟 2: 背景批次填充歷史 NULL 資料
步驟 3: ALTER TABLE ADD CONSTRAINT ... CHECK (col IS NOT NULL) NOT VALID;
步驟 4: ALTER TABLE VALIDATE CONSTRAINT ...;
步驟 5: ALTER TABLE ALTER COLUMN col SET NOT NULL;
         ALTER TABLE DROP CONSTRAINT ...;
```

---

## 遷移腳本規範

### 命名規則

```
NNN_<description>.sql
```

- `NNN` = 三位數遞增編號（如 `018`、`019`）
- `<description>` = 底線分隔的簡短說明

### 腳本格式

```sql
-- NNN: 簡短說明
-- 預估執行時間: < X 秒
-- 鎖定級別: NONE / ACCESS SHARE / SHARE UPDATE EXCLUSIVE

-- 正向遷移
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_xxx ON table_name (...);

-- 注意: CONCURRENTLY 不可在 transaction 中執行
-- 若使用 sqlx migrate，須拆為獨立檔案或使用 raw SQL 執行
```

### 禁止事項

- ❌ `ALTER TABLE ... ALTER COLUMN ... TYPE` 在大表上（> 10 萬筆）
- ❌ `DROP COLUMN` 不經過棄用週期
- ❌ `CREATE INDEX` 不加 `CONCURRENTLY`（除非表非常小）
- ❌ 在遷移中執行長時間 `UPDATE`（超過 10 秒）
- ❌ 使用 `LOCK TABLE` 明確鎖定

---

## 部署流程

```
1. 執行遷移 (sqlx migrate run)
   ↓
2. 滾動更新 API Pod A → 新版本
   ↓
3. 健康檢查通過
   ↓
4. 滾動更新 API Pod B → 新版本
   ↓
5. 驗證功能正常
   ↓
6. （下次部署）清理步驟：移除舊欄位/觸發器
```

---

## 回滾策略

- 每個遷移腳本應附帶回滾指令（註解中）
- 新增欄位的回滾：`ALTER TABLE DROP COLUMN IF EXISTS`
- 新增索引的回滾：`DROP INDEX CONCURRENTLY IF EXISTS`
- 資料變更需要評估是否可回滾

---

## 監控

遷移執行期間需觀察：

- `pg_stat_activity`：檢查是否有長時間持有的鎖
- `pg_locks`：檢查等待中的鎖
- 應用程式錯誤率：Prometheus `http_requests_total{status="500"}`
- 回應延遲：P95/P99 是否異常上升
