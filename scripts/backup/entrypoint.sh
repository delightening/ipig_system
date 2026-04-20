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

# E-3: 容器啟動時立即驗證 GPG 設定，而非等到凌晨 2 點備份才失敗
if [ "${BACKUP_REQUIRE_ENCRYPTION:-}" = "true" ]; then
    if [ -z "${BACKUP_GPG_RECIPIENT:-}" ]; then
        echo "ERROR: BACKUP_REQUIRE_ENCRYPTION=true 但 BACKUP_GPG_RECIPIENT 未設定。"
        echo "       請在 .env 設定收件者 Key ID 或 email，並匯入 GPG 公鑰後重新啟動。"
        exit 1
    fi
    if ! gpg --list-keys "${BACKUP_GPG_RECIPIENT}" > /dev/null 2>&1; then
        echo "ERROR: GPG keyring 找不到 '${BACKUP_GPG_RECIPIENT}'。"
        echo "       執行: docker compose exec db-backup gpg --import /path/to/public.asc"
        exit 1
    fi
    echo "✅ GPG 加密已設定，收件者: ${BACKUP_GPG_RECIPIENT}"
fi

# 建立 crontab：將所有環境變數傳入 cron job
env | grep -E '^(POSTGRES_|PGPASSWORD|PGHOST|PGPORT|BACKUP_|RSYNC_|POSTGRES_PASSWORD_FILE)' > /etc/backup.env

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
