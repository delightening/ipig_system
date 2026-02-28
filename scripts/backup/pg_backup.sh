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

echo "[$(date -Iseconds)] Starting backup of ${DB_NAME}..."

# Create compressed backup
PGPASSWORD="${DB_PASSWORD:-}" pg_dump \
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

# Generate SHA256 checksum
sha256sum "$BACKUP_FILE" > "${BACKUP_FILE}.sha256"
echo "Checksum: $(cat "${BACKUP_FILE}.sha256")"

# Cleanup old backups（P1-R4-11：含 .sql.gz 與 .sql.gz.gpg）
DELETED=0
DELETED=$((DELETED + $(find "$BACKUP_DIR" -name "ipig_*.sql.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)))
DELETED=$((DELETED + $(find "$BACKUP_DIR" -name "ipig_*.sql.gz.gpg" -mtime +${RETENTION_DAYS} -delete -print | wc -l)))
find "$BACKUP_DIR" -name "ipig_*.sha256" -mtime +${RETENTION_DAYS} -delete

FILESIZE=$(du -h "$BACKUP_FILE" | cut -f1)
echo "[$(date -Iseconds)] Backup complete: $BACKUP_FILE ($FILESIZE)"
echo "  Retention: ${RETENTION_DAYS} days, cleaned up ${DELETED} old backups"
