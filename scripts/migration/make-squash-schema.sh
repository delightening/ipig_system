#!/usr/bin/env bash
# 從一顆「已套用全部 migration 的乾淨 DB」產生 squash 用的 002_schema.sql。
#
# 為什麼是從 DB dump 而不是手動合併 152 支檔案：那些檔案裡有大量 ALTER／DROP 重建／
# 一次性資料修正互相疊加，人工重寫極易漏步驟。直接拿「跑完全部 migration 的結果」
# 當新起點，保證與實際 schema 100% 一致。
#
# ⚠️ 來源 DB 的選擇很重要：
#   必須是「照 repo 現有 migration 從零跑出來」的 DB，而不是長期運行的 prod。
#   prod 可能有手動修改造成的漂移（drift），那些漂移不在任何 migration 檔裡，
#   dump 進來會讓新的 001/002 產生「migration 檔生不出來的 schema」。
#
# 用法：
#   SOURCE_CONTAINER=ipig-np-db ./scripts/migration/make-squash-schema.sh
#   # 或直接給連線字串
#   SOURCE_DATABASE_URL=postgres://... ./scripts/migration/make-squash-schema.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$SCRIPT_DIR/../../backend/migrations/002_schema.sql"

dump() {
  if [ -n "${SOURCE_CONTAINER:-}" ]; then
    MSYS_NO_PATHCONV=1 docker exec "$SOURCE_CONTAINER" \
      pg_dump -U "${PGUSER:-postgres}" -d "${PGDATABASE:-ipig_db}" \
      --schema-only --no-owner --exclude-table=_sqlx_migrations
  elif [ -n "${SOURCE_DATABASE_URL:-}" ]; then
    pg_dump "$SOURCE_DATABASE_URL" --schema-only --no-owner --exclude-table=_sqlx_migrations
  else
    echo "ERROR: 需設定 SOURCE_CONTAINER 或 SOURCE_DATABASE_URL" >&2
    exit 1
  fi
}

echo "=== 產生 schema dump ==="
RAW="$(dump)"

echo "=== 清理三處會讓 sqlx 執行失敗的內容 ==="
# 1. `\restrict` / `\unrestrict`：pg_dump 16 新增的 **psql 專用 meta-command**，
#    不是合法 SQL。sqlx 的 migration runner 直接送給 Postgres 執行 → 語法錯誤。
# 2. `SELECT pg_catalog.set_config('search_path', '', false);`：把連線的 search_path
#    清空。sqlx 查自己的 `_sqlx_migrations` 時沒加 schema 前綴，會在 migration 執行到
#    一半時突然找不到自己的記帳表。dump 內容本身全部 schema-qualified，移掉無副作用。
# 3. `CREATE SCHEMA public;`：全新 DB 已內建 public schema，會 already exists。
#    （不加 --schema 篩選時 pg_dump 通常不會輸出這行，但保險起見一併處理。）
CLEANED="$(printf '%s\n' "$RAW" \
  | grep -v '^\\restrict ' \
  | grep -v '^\\unrestrict ' \
  | grep -v "^SELECT pg_catalog.set_config('search_path'" \
  | sed 's/^CREATE SCHEMA public;$/CREATE SCHEMA IF NOT EXISTS public;/')"

printf '%s\n' "$CLEANED" > "$OUT"

echo "=== 驗證清理結果 ==="
fail=0
if grep -q '^\\restrict\|^\\unrestrict' "$OUT"; then echo "  ❌ 仍有 psql meta-command"; fail=1; fi
if grep -q "set_config('search_path'" "$OUT"; then echo "  ❌ 仍有 search_path 重設"; fail=1; fi
if grep -q '_sqlx_migrations' "$OUT"; then echo "  ❌ 仍含 _sqlx_migrations"; fail=1; fi
if ! grep -q 'CREATE EXTENSION' "$OUT"; then echo "  ⚠️ 沒有 CREATE EXTENSION（若來源確實沒用 extension 則正常）"; fi
[ "$fail" -eq 0 ] && echo "  ✅ 清理完成"

echo ""
echo "產出：$OUT（$(wc -l < "$OUT") 行）"
echo "下一步：套到全新丟棄 DB 做 schema diff 驗證，須零真實差異。"
