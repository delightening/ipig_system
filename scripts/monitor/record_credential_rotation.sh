#!/usr/bin/env bash
# =============================================================================
# iPig System - Record Credential Rotation (輪換後執行)
# =============================================================================
# 維運人員完成 DB 或 SMTP 憑證輪換後，執行此腳本更新「上次輪換日期」，
# 供 check_credential_rotation.sh 計算下次到期日。
#
# 用法:
#   ./record_credential_rotation.sh db     # 記錄 DB 輪換
#   ./record_credential_rotation.sh smtp   # 記錄 SMTP 輪換
#   ./record_credential_rotation.sh all    # 記錄兩者
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="${STATE_DIR:-${SCRIPT_DIR}/.credential_state}"
today=$(date +%Y-%m-%d)

mkdir -p "$STATE_DIR"

record() {
  local name=$1
  local file=$2
  echo "$today" > "$file"
  echo "已記錄 ${name} 憑證輪換日期: $today"
}

case "${1:-}" in
  db)
    record "DB" "${STATE_DIR}/last_rotated_db"
    ;;
  smtp)
    record "SMTP" "${STATE_DIR}/last_rotated_smtp"
    ;;
  all)
    record "DB" "${STATE_DIR}/last_rotated_db"
    record "SMTP" "${STATE_DIR}/last_rotated_smtp"
    ;;
  *)
    echo "用法: $0 {db|smtp|all}"
    echo "  db   - 記錄 DB 憑證輪換"
    echo "  smtp - 記錄 SMTP 憑證輪換"
    echo "  all  - 記錄兩者"
    exit 1
    ;;
esac
