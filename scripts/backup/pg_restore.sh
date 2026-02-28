#!/bin/bash
# iPig 資料庫還原腳本（P1-R4-10）
# 用法: ./pg_restore.sh [備份檔案路徑]
# 範例: ./pg_restore.sh /backups/ipig_20260228_020000.sql.gz

set -euo pipefail

BACKUP_FILE="${1:-}"
DB_HOST="${DB_HOST:-db}"
DB_USER="${DB_USER:-postgres}"
DB_NAME="${DB_NAME:-erp_db}"
DB_PASSWORD="${DB_PASSWORD:-}"

export PGPASSWORD="$DB_PASSWORD"

usage() {
    echo "用法: $0 <備份檔案路徑>"
    echo "  備份檔案應為 .sql.gz 或 .sql.gz.gpg 格式"
    echo ""
    echo "環境變數:"
    echo "  DB_HOST     資料庫主機（預設: db）"
    echo "  DB_USER     資料庫使用者（預設: postgres）"
    echo "  DB_NAME     資料庫名稱（預設: erp_db）"
    echo "  DB_PASSWORD 資料庫密碼"
    exit 1
}

if [[ -z "$BACKUP_FILE" ]]; then
    echo "錯誤: 請指定備份檔案路徑"
    usage
fi

if [[ ! -f "$BACKUP_FILE" ]]; then
    echo "錯誤: 備份檔案不存在: $BACKUP_FILE"
    exit 1
fi

# 判斷是否為 GPG 加密
INPUT_FILE="$BACKUP_FILE"
DECRYPTED=""
if [[ "$BACKUP_FILE" == *.gpg ]]; then
    DECRYPTED=$(mktemp)
    trap "rm -f $DECRYPTED" EXIT
    echo "解密備份檔案..."
    gpg --decrypt "$BACKUP_FILE" > "$DECRYPTED" 2>/dev/null || {
        echo "錯誤: GPG 解密失敗，請確認金鑰已設定"
        exit 1
    }
    INPUT_FILE="$DECRYPTED"
fi

# 解壓縮（若為 .gz）
RESTORE_INPUT="$INPUT_FILE"
TEMP_GZ=""
if [[ "$INPUT_FILE" == *.gz ]]; then
    echo "解壓縮備份..."
    RESTORE_INPUT=$(mktemp)
    TEMP_GZ="$RESTORE_INPUT"
    trap "rm -f $TEMP_GZ $DECRYPTED 2>/dev/null" EXIT
    gunzip -c "$INPUT_FILE" > "$RESTORE_INPUT"
fi

# 驗證備份可讀（pg_dump -Fc 格式需用 pg_restore --list）
echo "驗證備份完整性..."
if pg_restore --list "$RESTORE_INPUT" > /dev/null 2>&1; then
    echo "備份格式正確（custom format）"
else
    # 可能是 plain SQL
    if head -c 5 "$RESTORE_INPUT" | grep -q "CREATE\|DROP\|--"; then
        echo "備份格式為 plain SQL，使用 psql 還原"
        psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" < "$RESTORE_INPUT"
        echo "[$(date -Iseconds)] 還原完成"
        exit 0
    else
        echo "錯誤: 無法識別備份格式"
        exit 1
    fi
fi

# 確認還原目標
echo "即將還原至: $DB_HOST / $DB_NAME"
echo "警告: 此操作將覆蓋現有資料！"
read -p "輸入 'yes' 確認繼續: " confirm
if [[ "$confirm" != "yes" ]]; then
    echo "已取消"
    exit 0
fi

# 執行還原（custom format）
echo "[$(date -Iseconds)] 開始還原..."
pg_restore -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
    --clean \
    --if-exists \
    --no-owner \
    --no-acl \
    "$RESTORE_INPUT" 2>/dev/null || true

# pg_restore 在 --clean 時可能回傳非零（因物件不存在），故用 || true
# 驗證還原結果
echo "驗證還原結果..."
TABLES=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" 2>/dev/null || echo "0")
echo "[$(date -Iseconds)] 還原完成，public schema 中共 $TABLES 個資料表"
