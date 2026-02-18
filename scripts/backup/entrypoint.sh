#!/bin/sh
# ============================================
# 備份容器 Entrypoint
# 設定 cron 排程並啟動
# ============================================

set -e

SCHEDULE="${BACKUP_SCHEDULE:-0 2 * * *}"

echo "📋 iPIG DB Backup Container"
echo "   排程: ${SCHEDULE}"
echo "   保留: ${BACKUP_RETENTION_DAYS:-30} 天"
echo "   異地: ${RSYNC_TARGET:-（未設定）}"
echo ""

# 建立 crontab：將所有環境變數傳入 cron job
env | grep -E '^(POSTGRES_|PGPASSWORD|PGHOST|PGPORT|BACKUP_|RSYNC_)' > /etc/backup.env

# 產生 crontab 設定
echo "${SCHEDULE} . /etc/backup.env && /usr/local/bin/pg_backup.sh >> /var/log/backup.log 2>&1" > /etc/crontabs/root

echo "✅ Cron 排程已設定，啟動 crond..."

# 執行一次立即備份（容器首次啟動時）
if [ "${BACKUP_ON_START:-false}" = "true" ]; then
    echo "📦 執行啟動備份..."
    . /etc/backup.env
    /usr/local/bin/pg_backup.sh
fi

# 啟動 cron（前景模式）
crond -f -l 2
