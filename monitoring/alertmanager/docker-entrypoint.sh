#!/bin/sh
# Alertmanager configuration entrypoint
# Generates alertmanager.yml from environment variables.
# Falls back to static config if ALERTMANAGER_WEBHOOK_URL is not set.

set -e

CONFIG_FILE="/tmp/alertmanager.yml"
STATIC_CONFIG="/etc/alertmanager/alertmanager.yml"

# If no webhook URL configured, use static config (receivers disabled)
if [ -z "$ALERTMANAGER_WEBHOOK_URL" ]; then
  exec /bin/alertmanager --config.file="$STATIC_CONFIG"
fi

# Generate config from environment variables
cat > "$CONFIG_FILE" <<EOF
global:
  resolve_timeout: 5m

route:
  group_by: ["alertname", "severity"]
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: "default"

  routes:
    - match:
        severity: critical
      receiver: "critical"
      repeat_interval: 1h

receivers:
  - name: "default"
    webhook_configs:
      - url: "${ALERTMANAGER_WEBHOOK_URL}"
        send_resolved: true

  - name: "critical"
    webhook_configs:
      - url: "${ALERTMANAGER_WEBHOOK_URL}"
        send_resolved: true
EOF

# Add email config for critical receiver if SMTP is configured
if [ -n "$ALERT_EMAIL_TO" ] && [ -n "$ALERT_SMTP_HOST" ]; then
  cat >> "$CONFIG_FILE" <<EOF
    email_configs:
      - to: "${ALERT_EMAIL_TO}"
        from: "${ALERT_EMAIL_FROM:-alertmanager@ipig.system}"
        smarthost: "${ALERT_SMTP_HOST}"
        auth_username: "${ALERT_SMTP_USER}"
        auth_password: "${ALERT_SMTP_PASSWORD}"
        require_tls: true
EOF
fi

exec /bin/alertmanager --config.file="$CONFIG_FILE"
