#!/bin/sh
# R28-6：docker-entrypoint.sh 邊界 case 自動化測試
#
# 涵蓋：
#   1) shell 語法檢查（sh -n）
#   2) Docker build 成功
#   3) 無 API_BACKEND_URL → exit 1 + FATAL 訊息
#   4) 純空白 API_BACKEND_URL → exit 1 + FATAL 訊息（trim）
#   5) 有效 API_BACKEND_URL → nginx 正常啟動 + envsubst 渲染
#
# 使用：在 CI 跑 `cd frontend && ./test-entrypoint.sh`。
# 需要 docker daemon 可用。

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

IMAGE_TAG="ipig-frontend-r28-6-test:latest"
CONTAINER_NAME="ipig-r28-6-test"

cleanup() {
    docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
    docker rmi "$IMAGE_TAG" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "=== Test 1: shell 語法檢查 ==="
sh -n docker-entrypoint.sh
echo "PASS syntax OK"
echo

echo "=== Test 2: docker build ==="
docker build -t "$IMAGE_TAG" . > /dev/null
echo "PASS build OK"
echo

echo "=== Test 3: 無 API_BACKEND_URL -> exit 1 ==="
output=$(docker run --rm --name "$CONTAINER_NAME" "$IMAGE_TAG" 2>&1) && exit_code=$? || exit_code=$?
if [ "$exit_code" != 1 ]; then
    echo "FAIL: 預期 exit 1, 實得 $exit_code"
    echo "$output"
    exit 1
fi
if ! echo "$output" | grep -q "FATAL: API_BACKEND_URL"; then
    echo "FAIL: 預期錯誤訊息含 'FATAL: API_BACKEND_URL', 實得:"
    echo "$output"
    exit 1
fi
echo "PASS correctly fails with FATAL message"
echo

echo "=== Test 4: 純空白 API_BACKEND_URL -> exit 1 (trim) ==="
output=$(docker run --rm --name "$CONTAINER_NAME" -e API_BACKEND_URL="   " "$IMAGE_TAG" 2>&1) && exit_code=$? || exit_code=$?
if [ "$exit_code" != 1 ]; then
    echo "FAIL: 預期純空白 exit 1, 實得 $exit_code"
    echo "$output"
    exit 1
fi
if ! echo "$output" | grep -q "FATAL: API_BACKEND_URL"; then
    echo "FAIL: 預期 trim 後仍 fail-fast, 實得:"
    echo "$output"
    exit 1
fi
echo "PASS trim works (whitespace rejected)"
echo

echo "=== Test 5: 有效 API_BACKEND_URL -> nginx 啟動 ==="
# 不 bind port 避免與其他容器衝突；只驗證 envsubst 渲染與 nginx process 存在。
# --add-host backend:127.0.0.1 讓 nginx 啟動時能解析 upstream hostname
# （否則 nginx 啟動會 emerg "host not found in upstream"）
docker run -d --name "$CONTAINER_NAME" \
    -e API_BACKEND_URL="http://backend:3000" \
    --add-host=backend:127.0.0.1 \
    "$IMAGE_TAG" > /dev/null
sleep 5
# 容器是否存活
if [ "$(docker inspect -f '{{.State.Running}}' "$CONTAINER_NAME" 2>/dev/null)" != "true" ]; then
    echo "FAIL: 容器未持續執行, logs:"
    docker logs "$CONTAINER_NAME" 2>&1 | head -30
    exit 1
fi
# 驗證 default.conf 已被 envsubst 渲染（不再含未解析變數）
# MSYS_NO_PATHCONV=1：避免 Git Bash on Windows 把 /etc/... 翻成 C:/Users/.../etc/...
rendered=$(MSYS_NO_PATHCONV=1 docker exec "$CONTAINER_NAME" cat /etc/nginx/conf.d/default.conf 2>/dev/null) || {
    echo "FAIL: docker exec 讀取 default.conf 失敗（容器內 nginx user 可能無權限）"
    docker logs "$CONTAINER_NAME" 2>&1 | head -20
    exit 1
}
if echo "$rendered" | grep -q '${API_BACKEND_URL}'; then
    echo "FAIL: envsubst 未渲染, default.conf 仍含 \${API_BACKEND_URL}"
    exit 1
fi
if ! echo "$rendered" | grep -q "http://backend:3000"; then
    echo "FAIL: 渲染後 default.conf 不含預期 backend URL"
    echo "$rendered" | head -20
    exit 1
fi
echo "PASS nginx started + envsubst rendered correctly"
echo

echo "=== ALL 5 tests PASSED ==="
