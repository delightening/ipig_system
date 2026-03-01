#!/usr/bin/env bash
# =============================================================================
# iPig System - Credential Rotation Reminder (半自動輪換)
# =============================================================================
# 每月提醒 DB/SMTP 憑證輪換，不執行輪換本身。
# 維運人員於維護窗口依 CREDENTIAL_ROTATION.md 手動執行輪換後，
# 需執行：echo "$(date +%Y-%m-%d)" > "$STATE_DIR/last_rotated_db"
#
# Cron example (每月 1 日 09:00 檢查，若逾期則發送告警):
# 0 9 1 * * /path/to/check_credential_rotation.sh || mail -s "iPig: 憑證輪換提醒" admin@example.com
#
# 環境變數:
#   STATE_DIR          - 狀態檔目錄 (預設: ./scripts/monitor/.credential_state)
#   ROTATION_DAYS      - 輪換週期天數 (預設: 30)
#   WARN_DAYS_BEFORE   - 到期前幾天開始警告 (預設: 7)
#   PROM_TEXTFILE_DIR  - 若設定，輸出 Prometheus 指標
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="${STATE_DIR:-${SCRIPT_DIR}/.credential_state}"
ROTATION_DAYS="${ROTATION_DAYS:-30}"
WARN_DAYS_BEFORE="${WARN_DAYS_BEFORE:-7}"

mkdir -p "$STATE_DIR"

# 取得今日日期 (YYYY-MM-DD)
today=$(date +%Y-%m-%d)

# 計算兩日期相差天數 (GNU date)
days_since() {
  local state_file=$1
  local d
  if [ -f "$state_file" ]; then
    d=$(cat "$state_file" 2>/dev/null | tr -d '\n' | head -1)
  fi
  if [ -z "$d" ]; then
    echo "999"  # 從未輪換，視為逾期
    return
  fi
  local diff
  if diff=$(($(date -d "$today" +%s 2>/dev/null) - $(date -d "$d" +%s 2>/dev/null))); then
    echo $(( diff / 86400 ))
  elif diff=$(($(date -j -f "%Y-%m-%d" "$today" +%s 2>/dev/null) - $(date -j -f "%Y-%m-%d" "$d" +%s 2>/dev/null))); then
    echo $(( diff / 86400 ))
  else
    echo "999"
  fi
}

check_credential() {
  local name=$1
  local state_file=$2
  local days
  days=$(days_since "$state_file")
  local status="ok"
  local msg=""
  if [ "$days" -ge "$ROTATION_DAYS" ]; then
    status="due"
    msg="[DUE] ${name} 憑證已 ${days} 天未輪換（週期 ${ROTATION_DAYS} 天），請依 docs/CREDENTIAL_ROTATION.md 執行輪換"
  elif [ "$days" -ge "$(( ROTATION_DAYS - WARN_DAYS_BEFORE ))" ]; then
    status="soon"
    msg="[WARN] ${name} 憑證將於 $(( ROTATION_DAYS - days )) 天內到期輪換（已 ${days} 天）"
  else
    msg="[OK] ${name} 憑證已輪換 ${days} 天前（週期 ${ROTATION_DAYS} 天）"
  fi
  echo "$status|$days|$msg"
}

exit_code=0
db_result=$(check_credential "DB" "${STATE_DIR}/last_rotated_db")
smtp_result=$(check_credential "SMTP" "${STATE_DIR}/last_rotated_smtp")

db_status="${db_result%%|*}"
db_days="${db_result#*|}"
db_days="${db_days%%|*}"
db_msg="${db_result#*|}"
db_msg="${db_msg#*|}"

smtp_status="${smtp_result%%|*}"
smtp_days="${smtp_result#*|}"
smtp_days="${smtp_days%%|*}"
smtp_msg="${smtp_result#*|}"
smtp_msg="${smtp_msg#*|}"

echo "$db_msg"
echo "$smtp_msg"

[ "$db_status" = "due" ] || [ "$smtp_status" = "due" ] && exit_code=1

# Prometheus textfile output
if [ -n "${PROM_TEXTFILE_DIR:-}" ]; then
  prom_file="${PROM_TEXTFILE_DIR}/ipig_credential_rotation.prom"
  mkdir -p "$PROM_TEXTFILE_DIR"
  # days_overdue: 正數=已逾期天數，負數=距到期還有幾天
  db_overdue=$(( db_days - ROTATION_DAYS ))
  smtp_overdue=$(( smtp_days - ROTATION_DAYS ))
  cat > "${prom_file}.$$" <<PROM
# HELP ipig_credential_rotation_days_since_last Days since last credential rotation.
# TYPE ipig_credential_rotation_days_since_last gauge
ipig_credential_rotation_days_since_last{credential="db"} ${db_days}
ipig_credential_rotation_days_since_last{credential="smtp"} ${smtp_days}
# HELP ipig_credential_rotation_due 1 if credential rotation is overdue, 0 otherwise.
# TYPE ipig_credential_rotation_due gauge
ipig_credential_rotation_due{credential="db"} $([ "$db_status" = "due" ] && echo 1 || echo 0)
ipig_credential_rotation_due{credential="smtp"} $([ "$smtp_status" = "due" ] && echo 1 || echo 0)
PROM
  mv "${prom_file}.$$" "$prom_file"
fi

exit $exit_code
