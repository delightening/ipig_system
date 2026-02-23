#!/bin/bash
# ============================================
# iPIG 資料庫自動備份腳本
# 功能：pg_dump + gzip + 30天清理 + rsync 異地備份
# ============================================

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILENAME="ipig_db_${TIMESTAMP}.sql.gz"
LOG_PREFIX="[$(date '+%Y-%m-%d %H:%M:%S')]"

# 確保備份目錄存在
mkdir -p "${BACKUP_DIR}"

echo "${LOG_PREFIX} 開始備份..."

# 執行 pg_dump（PGPASSWORD 由環境變數提供）
if pg_dump -h "${PGHOST:-db}" -p "${PGPORT:-5432}" -U "${POSTGRES_USER:-postgres}" -d "${POSTGRES_DB:-ipig_db}" \
    --no-owner --no-privileges --clean --if-exists | gzip > "${BACKUP_DIR}/${FILENAME}"; then
    SIZE=$(du -h "${BACKUP_DIR}/${FILENAME}" | cut -f1)
    echo "${LOG_PREFIX} ✅ 備份完成: ${FILENAME} (${SIZE})"
else
    echo "${LOG_PREFIX} ❌ 備份失敗！"
    exit 1
fi

# GPG 加密（如設定 BACKUP_GPG_RECIPIENT）
if [ -n "${BACKUP_GPG_RECIPIENT:-}" ]; then
    echo "${LOG_PREFIX} 🔐 開始 GPG 加密..."
    if gpg --batch --yes --recipient "${BACKUP_GPG_RECIPIENT}" \
        --trust-model always --encrypt "${BACKUP_DIR}/${FILENAME}"; then
        rm -f "${BACKUP_DIR}/${FILENAME}"  # 移除未加密版
        FILENAME="${FILENAME}.gpg"
        echo "${LOG_PREFIX} ✅ GPG 加密完成: ${FILENAME}"
    else
        echo "${LOG_PREFIX} ⚠️ GPG 加密失敗（未加密備份已保留）"
    fi
fi

# 清理超過保留天數的舊備份
DELETED=$(find "${BACKUP_DIR}" -name "ipig_db_*.sql.gz" -mtime +${RETENTION_DAYS} -print -delete | wc -l)
if [ "${DELETED}" -gt 0 ]; then
    echo "${LOG_PREFIX} 🗑️ 已清理 ${DELETED} 個超過 ${RETENTION_DAYS} 天的備份"
fi

# rsync 異地備份（如設定 RSYNC_TARGET）
if [ -n "${RSYNC_TARGET:-}" ]; then
    echo "${LOG_PREFIX} 📤 開始異地備份至 ${RSYNC_TARGET}..."
    if rsync -az --timeout=60 "${BACKUP_DIR}/${FILENAME}" "${RSYNC_TARGET}/"; then
        echo "${LOG_PREFIX} ✅ 異地備份完成"
    else
        echo "${LOG_PREFIX} ⚠️ 異地備份失敗（本地備份已保留）"
    fi

    # /uploads 目錄異地備份
    if [ -d "/uploads" ]; then
        echo "${LOG_PREFIX} 📤 開始 /uploads 異地備份..."
        if rsync -az --timeout=120 /uploads/ "${RSYNC_TARGET}/uploads/"; then
            echo "${LOG_PREFIX} ✅ /uploads 異地備份完成"
        else
            echo "${LOG_PREFIX} ⚠️ /uploads 異地備份失敗"
        fi
    fi
fi

echo "${LOG_PREFIX} 備份流程結束"
