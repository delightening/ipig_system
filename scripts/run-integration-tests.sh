#!/usr/bin/env bash
# iPig 後端整合測試腳本 (Linux/macOS)
#
# 前置需求：
# - PostgreSQL 已安裝且可連線
# - 已設定 TEST_DATABASE_URL 或 DATABASE_URL 指向專用測試資料庫
# - 建議使用獨立測試 DB（如 ipig_db_test），避免與開發 DB 混用
#
# 若出現 VersionMismatch(1) 錯誤，表示測試 DB 的 migration 紀錄與程式碼不符。
# 解法：drop 並重建測試 DB，或執行 cargo run --bin fix_migration <version> 後再 sqlx migrate run

set -e
BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)/backend"

# 檢查環境變數
DB_URL="${TEST_DATABASE_URL:-$DATABASE_URL}"
if [ -z "$DB_URL" ]; then
    echo "錯誤：請設定 TEST_DATABASE_URL 或 DATABASE_URL 環境變數"
    echo "範例：export TEST_DATABASE_URL='postgres://user:pass@localhost:5432/ipig_db_test'"
    exit 1
fi

echo "使用資料庫：${DB_URL%%@*}@***"
echo ""
echo "執行 sqlx migrate run..."
cd "$BACKEND_DIR"
if ! sqlx migrate run; then
    echo "Migration 失敗。若出現 VersionMismatch，請 drop 測試 DB 後重建，或執行 fix_migration。"
    exit 1
fi

echo ""
echo "執行 cargo test（整合測試）..."
cargo test
