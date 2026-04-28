# Migration Down SQL

> R30-26: 補強 SQLx 不原生支援 DOWN migration 的缺口。

## 機制

SQLx CLI 不支援 `up.sql` / `down.sql` 配對檔案，因此本專案採取**約定式**：

- `backend/migrations/NNN_xxx.sql` — UP（由 `sqlx::migrate!` 執行）
- `backend/migrations/down/NNN_xxx.sql` — DOWN（**手動**執行，僅 staging / dev rollback 用）

**Production 採 forward-only migration**：上線後僅以新 migration 修正前一個 migration 的問題，不執行 down。Down SQL 僅供 staging IQ-PQ 演練、或 dev 環境快速 reset 局部結構。

## 撰寫規範

每個**新建立**的 migration（041 起）必須附 down SQL：

1. 路徑：`backend/migrations/down/NNN_xxx.sql`，與 up 同檔名 prefix。
2. 內容：恰好反向 up 的 schema 變更（`DROP` 對應 `CREATE`、`ALTER ... DROP COLUMN` 對應 `ADD COLUMN`）。
3. **資料遺失警告**：若 down 會破壞性丟資料（例如 DROP COLUMN 含使用者寫入的欄位），於檔頭加 SQL 註解 `-- WARNING: data loss on down`。
4. 不可逆 migration（例如 type cast、enum 值變更後已被引用）— 寫一行註解 `-- IRREVERSIBLE: <理由>`，down 留空。
5. Idempotency 偏好：`DROP TABLE IF EXISTS`、`DROP COLUMN IF EXISTS`，避免重複執行炸掉。

## 執行

```bash
# 假設要回退 migration 045 044 043 三個版本：
psql "$DATABASE_URL" -f backend/migrations/down/045_xxx.sql
psql "$DATABASE_URL" -f backend/migrations/down/044_xxx.sql
psql "$DATABASE_URL" -f backend/migrations/down/043_xxx.sql

# 同步 _sqlx_migrations 表：
psql "$DATABASE_URL" -c "DELETE FROM _sqlx_migrations WHERE version >= 43;"
```

## CI 守衛

`.github/workflows/ci.yml` 的 `migration-down-guard` job 會檢查：
- PR diff 含 `backend/migrations/NNN_xxx.sql`（version ≥ 041）→ 必須同時含 `backend/migrations/down/NNN_xxx.sql`
- 既有 001-040 migration 在 ignore list 內，不回溯補（列為 R30-26b backlog）

## 既有 migration 的 down SQL

001-040 的反向 SQL 部分已收錄在 [docs/db/DB_ROLLBACK.md](../../../docs/db/DB_ROLLBACK.md)。後續補齊由 R30-26b 追蹤。
