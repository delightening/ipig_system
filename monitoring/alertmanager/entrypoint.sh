#!/bin/sh
# 使用 sed 將環境變數替換至設定檔（busybox 相容，無需 envsubst）
# Alertmanager 不支援 ${VAR}，故於啟動前預處理
set -e
if [ -f /etc/alertmanager/alertmanager.example.yml ]; then
  sed -e "s|\${ALERTMANAGER_WEBHOOK_URL}|${ALERTMANAGER_WEBHOOK_URL}|g" \
      -e "s|\${ALERT_EMAIL_TO}|${ALERT_EMAIL_TO}|g" \
      -e "s|\${ALERT_EMAIL_FROM}|${ALERT_EMAIL_FROM}|g" \
      -e "s|\${ALERT_SMTP_HOST}|${ALERT_SMTP_HOST}|g" \
      -e "s|\${ALERT_SMTP_USER}|${ALERT_SMTP_USER}|g" \
      -e "s|\${ALERT_SMTP_PASSWORD}|${ALERT_SMTP_PASSWORD}|g" \
      /etc/alertmanager/alertmanager.example.yml > /tmp/alertmanager.yml
  exec /bin/alertmanager --config.file=/tmp/alertmanager.yml "$@"
else
  exec /bin/alertmanager --config.file=/etc/alertmanager/alertmanager.yml "$@"
fi
