#!/bin/sh
# 從環境變數產生 Alertmanager 設定檔
# 用法:  source .env 2>/dev/null; ./scripts/generate-alertmanager-config.sh
# 或:    export ALERTMANAGER_WEBHOOK_URL=...; ./scripts/generate-alertmanager-config.sh

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEMPLATE="${SCRIPT_DIR}/../monitoring/alertmanager/alertmanager.example.yml"
OUTPUT="${SCRIPT_DIR}/../monitoring/alertmanager/alertmanager.yml"

if [ ! -f "$TEMPLATE" ]; then
  echo "Error: Template not found: $TEMPLATE" >&2
  exit 1
fi

# 若無 envsubst 則提示
if ! command -v envsubst >/dev/null 2>&1; then
  echo "Error: envsubst not found. Install gettext (apt install gettext / brew install gettext)" >&2
  exit 1
fi

# 僅替換我們定義的變數
export ALERTMANAGER_WEBHOOK_URL="${ALERTMANAGER_WEBHOOK_URL:-}"
export ALERT_EMAIL_TO="${ALERT_EMAIL_TO:-}"
export ALERT_EMAIL_FROM="${ALERT_EMAIL_FROM:-}"
export ALERT_SMTP_HOST="${ALERT_SMTP_HOST:-}"
export ALERT_SMTP_USER="${ALERT_SMTP_USER:-}"
export ALERT_SMTP_PASSWORD="${ALERT_SMTP_PASSWORD:-}"

envsubst '$ALERTMANAGER_WEBHOOK_URL $ALERT_EMAIL_TO $ALERT_EMAIL_FROM $ALERT_SMTP_HOST $ALERT_SMTP_USER $ALERT_SMTP_PASSWORD' \
  < "$TEMPLATE" > "$OUTPUT"

echo "Generated: $OUTPUT"
