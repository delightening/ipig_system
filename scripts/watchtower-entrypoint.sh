#!/bin/sh
# High 1/2: 從 Docker Secrets 讀取 Watchtower API token 與 SMTP 密碼，避免明文環境變數
set -e
if [ -f /run/secrets/watchtower_api_token ]; then
    export WATCHTOWER_HTTP_API_TOKEN=$(cat /run/secrets/watchtower_api_token)
fi
if [ -f /run/secrets/watchtower_smtp_password ]; then
    export WATCHTOWER_NOTIFICATION_EMAIL_SERVER_PASSWORD=$(cat /run/secrets/watchtower_smtp_password)
fi
exec /watchtower "$@"
