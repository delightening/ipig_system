#!/bin/bash
set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/ipig_${TIMESTAMP}.sql.gz"
DB_HOST="${DB_HOST:-db}"
DB_USER="${DB_USER:-postgres}"
DB_NAME="${DB_NAME:-erp_db}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"

mkdir -p "$BACKUP_DIR"

# High 5: 優先從 Docker Secret 檔讀取密碼，避免 PGPASSWORD 暴露於 process listing
if [ -n "${POSTGRES_PASSWORD_FILE:-}" ] && [ -f "$POSTGRES_PASSWORD_FILE" ]; then
  export PGPASSWORD=$(cat "$POSTGRES_PASSWORD_FILE")
elif [ -n "${DB_PASSWORD:-}" ]; then
  export PGPASSWORD="${DB_PASSWORD}"
fi

echo "[$(date -Iseconds)] Starting backup of ${DB_NAME}..."

# Create compressed backup
pg_dump \
  -h "$DB_HOST" \
  -U "$DB_USER" \
  -Fc \
  "$DB_NAME" > "${BACKUP_FILE%.gz}"

gzip "${BACKUP_FILE%.gz}"

# Verify integrity
echo "Verifying backup integrity..."
gunzip -t "$BACKUP_FILE" || {
  echo "ERROR: Backup file is corrupt: $BACKUP_FILE"
  exit 1
}

# Verify pg_restore can read the backup
gunzip -c "$BACKUP_FILE" | pg_restore --list > /dev/null 2>&1 || {
  echo "ERROR: pg_restore cannot read backup: $BACKUP_FILE"
  exit 1
}

# High 7: 若設定 BACKUP_GPG_RECIPIENT 則加密備份；生產可強制要求加密
if [ -n "${BACKUP_REQUIRE_ENCRYPTION:-}" ] && [ "${BACKUP_REQUIRE_ENCRYPTION}" = "true" ]; then
  if [ -z "${BACKUP_GPG_RECIPIENT:-}" ]; then
    echo "ERROR: Production backup requires BACKUP_GPG_RECIPIENT to be set."
    exit 1
  fi
fi

FINAL_FILE="$BACKUP_FILE"
if [ -n "${BACKUP_GPG_RECIPIENT:-}" ]; then
  echo "Encrypting backup with GPG for recipient: $BACKUP_GPG_RECIPIENT"
  gpg --batch --yes --encrypt --recipient "$BACKUP_GPG_RECIPIENT" -o "${BACKUP_FILE}.gpg" "$BACKUP_FILE" || {
    echo "ERROR: GPG encryption failed"
    exit 1
  }
  rm -f "$BACKUP_FILE"
  FINAL_FILE="${BACKUP_FILE}.gpg"
fi

# Generate SHA256 checksum for final file
sha256sum "$FINAL_FILE" > "${FINAL_FILE}.sha256"
echo "Checksum: $(cat "${FINAL_FILE}.sha256")"

# Cleanup old backups（P1-R4-11：含 .sql.gz 與 .sql.gz.gpg）
DELETED=0
DELETED=$((DELETED + $(find "$BACKUP_DIR" -name "ipig_*.sql.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)))
DELETED=$((DELETED + $(find "$BACKUP_DIR" -name "ipig_*.sql.gz.gpg" -mtime +${RETENTION_DAYS} -delete -print | wc -l)))
find "$BACKUP_DIR" -name "ipig_*.sha256" -mtime +${RETENTION_DAYS} -delete

FILESIZE=$(du -h "$FINAL_FILE" | cut -f1)
echo "[$(date -Iseconds)] Backup complete: $FINAL_FILE ($FILESIZE)"
echo "  Retention: ${RETENTION_DAYS} days, cleaned up ${DELETED} old backups"
