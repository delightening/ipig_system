#!/bin/sh
# Alertmanager configuration entrypoint
# Generates alertmanager.yml from environment variables.
# Falls back to static config if ALERTMANAGER_WEBHOOK_URL is not set.

set -e

CONFIG_FILE="/tmp/alertmanager.yml"
STATIC_CONFIG="/etc/alertmanager/alertmanager.yml"

# H7: YAML-safe escape — wraps value in YAML single quotes and escapes embedded
# single quotes as '' per YAML spec §7.3.3. Prevents YAML injection when passwords
# or URLs contain special characters (: # " ' { } [ ]).
yaml_escape() {
    printf "'%s'" "$(printf '%s' "$1" | sed "s/'/''/g")"
}

# If no webhook URL configured, use static config (receivers disabled)
if [ -z "$ALERTMANAGER_WEBHOOK_URL" ]; then
  exec /bin/alertmanager --config.file="$STATIC_CONFIG"
fi

# Pre-escape all values that will be embedded in YAML
WEBHOOK_URL_SAFE=$(yaml_escape "$ALERTMANAGER_WEBHOOK_URL")

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
      - url: ${WEBHOOK_URL_SAFE}
        send_resolved: true

  - name: "critical"
    webhook_configs:
      - url: ${WEBHOOK_URL_SAFE}
        send_resolved: true
EOF

# Add email config for critical receiver if SMTP is configured
if [ -n "$ALERT_EMAIL_TO" ] && [ -n "$ALERT_SMTP_HOST" ]; then
  # H7: Escape all SMTP config values to prevent YAML injection
  EMAIL_TO_SAFE=$(yaml_escape "$ALERT_EMAIL_TO")
  EMAIL_FROM_SAFE=$(yaml_escape "${ALERT_EMAIL_FROM:-alertmanager@ipig.system}")
  SMTP_HOST_SAFE=$(yaml_escape "$ALERT_SMTP_HOST")
  SMTP_USER_SAFE=$(yaml_escape "$ALERT_SMTP_USER")
  SMTP_PASS_SAFE=$(yaml_escape "$ALERT_SMTP_PASSWORD")

  cat >> "$CONFIG_FILE" <<EOF
    email_configs:
      - to: ${EMAIL_TO_SAFE}
        from: ${EMAIL_FROM_SAFE}
        smarthost: ${SMTP_HOST_SAFE}
        auth_username: ${SMTP_USER_SAFE}
        auth_password: ${SMTP_PASS_SAFE}
        require_tls: true
EOF
fi

exec /bin/alertmanager --config.file="$CONFIG_FILE"
