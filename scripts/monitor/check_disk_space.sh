#!/usr/bin/env bash
# =============================================================================
# iPig System - Disk Space Monitor
# =============================================================================
# Cron example (daily at 8am):
# 0 8 * * * /path/to/check_disk_space.sh | mail -s "iPig Disk Report" admin@example.com
#
# Cron example with Prometheus textfile collector:
# 0 * * * * PROM_TEXTFILE_DIR=/var/lib/prometheus/node-exporter /path/to/check_disk_space.sh
# =============================================================================

set -euo pipefail

UPLOAD_DIR="${UPLOAD_DIR:-/uploads}"
DISK_WARN_UPLOADS_GB="${DISK_WARN_UPLOADS_GB:-5}"
DISK_WARN_PERCENT="${DISK_WARN_PERCENT:-85}"

exit_code=0

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

bytes_to_gb() {
  awk "BEGIN { printf \"%.1f\", $1 / 1073741824 }"
}

# ---------------------------------------------------------------------------
# Check uploads directory size
# ---------------------------------------------------------------------------

if [ -d "$UPLOAD_DIR" ]; then
  upload_bytes=$(du -sb "$UPLOAD_DIR" 2>/dev/null | awk '{print $1}')
else
  upload_bytes=0
fi

upload_gb=$(bytes_to_gb "$upload_bytes")
threshold_bytes=$(awk "BEGIN { printf \"%.0f\", $DISK_WARN_UPLOADS_GB * 1073741824 }")

if awk "BEGIN { exit !($upload_bytes > $threshold_bytes) }"; then
  echo "[WARN] Uploads directory: ${upload_gb} GB exceeds threshold ${DISK_WARN_UPLOADS_GB} GB"
  exit_code=1
else
  echo "[OK] Uploads directory: ${upload_gb} GB (threshold: ${DISK_WARN_UPLOADS_GB} GB)"
fi

# ---------------------------------------------------------------------------
# Check overall disk usage
# ---------------------------------------------------------------------------

if [ -d "$UPLOAD_DIR" ]; then
  disk_usage_percent=$(df "$UPLOAD_DIR" 2>/dev/null | awk 'NR==2 { sub(/%/, "", $5); print $5 }')
else
  disk_usage_percent=$(df / 2>/dev/null | awk 'NR==2 { sub(/%/, "", $5); print $5 }')
fi

if [ -z "$disk_usage_percent" ]; then
  echo "[WARN] Could not determine disk usage"
  exit_code=1
elif [ "$disk_usage_percent" -gt "$DISK_WARN_PERCENT" ]; then
  echo "[WARN] Disk usage: ${disk_usage_percent}% exceeds threshold ${DISK_WARN_PERCENT}%"
  exit_code=1
else
  echo "[OK] Disk usage: ${disk_usage_percent}% (threshold: ${DISK_WARN_PERCENT}%)"
fi

# ---------------------------------------------------------------------------
# Prometheus textfile collector output
# ---------------------------------------------------------------------------

if [ -n "${PROM_TEXTFILE_DIR:-}" ]; then
  prom_file="${PROM_TEXTFILE_DIR}/ipig_disk.prom"
  mkdir -p "$PROM_TEXTFILE_DIR"

  cat > "${prom_file}.$$" <<PROM
# HELP ipig_uploads_bytes Total size of the uploads directory in bytes.
# TYPE ipig_uploads_bytes gauge
ipig_uploads_bytes ${upload_bytes}
# HELP ipig_uploads_threshold_bytes Warning threshold for uploads directory in bytes.
# TYPE ipig_uploads_threshold_bytes gauge
ipig_uploads_threshold_bytes ${threshold_bytes}
# HELP ipig_disk_usage_percent Disk usage percentage of the uploads partition.
# TYPE ipig_disk_usage_percent gauge
ipig_disk_usage_percent ${disk_usage_percent:-0}
# HELP ipig_disk_threshold_percent Warning threshold for disk usage percentage.
# TYPE ipig_disk_threshold_percent gauge
ipig_disk_threshold_percent ${DISK_WARN_PERCENT}
PROM

  mv "${prom_file}.$$" "$prom_file"
fi

exit $exit_code
