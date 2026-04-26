#!/bin/sh
# Frontend nginx entrypoint
#
# Responsibilities (R27-1 + R27-2，從 Dockerfile 一行 CMD 拆出來方便維護):
#   1) 建立 /tmp/nginx temp 目錄（/tmp 是 tmpfs，容器啟動時為空，需 runtime 建立）。
#   2) 視 /etc/nginx/conf.d/default.conf 可寫性決定是否 envsubst：
#      - 可寫（一般 prod 部署）：驗證 API_BACKEND_URL 已設且非空，
#        再以 envsubst 將模板渲染成 default.conf。
#      - 唯讀（docker-compose.test.yml 掛 nginx-ci.conf）：跳過 envsubst，
#        直接使用掛載的配置。
#   3) 啟動 nginx。

set -eu

mkdir -p /tmp/nginx

CONF_PATH="/etc/nginx/conf.d/default.conf"
TEMPLATE_PATH="/etc/nginx/templates/default.conf.template"

if [ -w "$CONF_PATH" ] || [ ! -e "$CONF_PATH" ]; then
    # R27-2: envsubst 路徑必須有 API_BACKEND_URL，否則生成的 default.conf
    # 會留 `proxy_pass http://;` 這種無效配置，nginx 啟動 fail 但訊息不直觀。
    # 提早 fail-fast 給明確錯誤訊息。
    # 去除前後空白避免 `" "` 純空白繞過驗證（CodeRabbit PR #217）
    _api_backend_url_trimmed=$(printf '%s' "${API_BACKEND_URL:-}" | tr -d '[:space:]')
    if [ -z "$_api_backend_url_trimmed" ]; then
        echo "❌ FATAL: API_BACKEND_URL 環境變數未設或為空（含純空白）" >&2
        echo "   prod 部署需提供 backend 的 reverse-proxy 目標，例如：" >&2
        echo "     -e API_BACKEND_URL=http://backend:3000" >&2
        echo "   CI 測試模式請改掛唯讀 default.conf 跳過 envsubst。" >&2
        exit 1
    fi
    envsubst '${API_BACKEND_URL}' < "$TEMPLATE_PATH" > "$CONF_PATH"
else
    echo "ℹ️  $CONF_PATH is read-only, using pre-mounted config (skipping envsubst)"
fi

exec nginx -g 'daemon off;'
