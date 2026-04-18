#!/usr/bin/env bash
# 高效驗證：前端 + 後端並行，任一失敗即退出
# 用法：./scripts/verify-deps.sh
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

run_frontend() {
  cd "$ROOT_DIR/frontend"
  pnpm install --frozen-lockfile
  pnpm run build
  pnpm run test:run
}

run_backend() {
  cd "$ROOT_DIR/backend"
  cargo check --release
  cargo test
}

# 並行執行，任一失敗即退出
(run_frontend) &
FE_PID=$!
(run_backend) &
BE_PID=$!

FAILED=0
wait $FE_PID || FAILED=1
wait $BE_PID || FAILED=1
[ $FAILED -eq 0 ] || exit 1
echo "✅ 驗證通過"
