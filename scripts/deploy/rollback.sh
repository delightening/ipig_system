#!/usr/bin/env bash
set -euo pipefail

# ============================================
# iPig System: Rollback to a specific image tag
# ============================================
# Usage:
#   bash scripts/deploy/rollback.sh <commit-sha>
#
# This will:
#   1. Pull the specified image versions
#   2. Restart api + web with pinned versions
#   3. Run health checks
#
# 2026-09-02：原步驟 1「Stop Watchtower」已移除——watchtower 服務本身已從
# docker-compose.prod.yml 移除，部署改為人工執行，不再需要「先擋住自動更新」
# 這一步，回滾後也不會有任何東西把版本推回去。
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMPOSE="docker compose -f $PROJECT_DIR/docker-compose.yml -f $PROJECT_DIR/docker-compose.prod.yml"

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <commit-sha>"
  echo ""
  echo "List recent local images:"
  docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.CreatedAt}}" | grep ipig || true
  exit 1
fi

TARGET_TAG="$1"

echo "============================================"
echo "iPig System: Rollback to $TARGET_TAG"
echo "============================================"
echo ""

# 1. Set image tag
echo "[1/4] Setting IMAGE_TAG=$TARGET_TAG..."
export IMAGE_TAG="$TARGET_TAG"

# 2. Pull specific version
# ⚠️ outbox-worker 必須一起回滾。watchtower 原本自動更新的是 api / web /
# outbox-worker 三個（三者都標 watchtower.enable=true），移除它改人工之後，
# 這裡少一個就會留下 split-version：api/web 回到舊版、outbox-worker 還在新版。
echo "[2/4] Pulling images..."
$COMPOSE pull api web outbox-worker

# 3. Restart services
# 指定服務名時 --no-build 是安全的——這三個在 prod overlay 都有 image 覆寫。
# （不指定服務時不可加 --no-build，print-pdf 沒有 GHCR 映像會失敗。）
echo "[3/4] Restarting services..."
$COMPOSE up -d --no-build api web outbox-worker

# 4. Health check
echo "[4/4] Running health checks..."
if bash "$SCRIPT_DIR/healthcheck.sh" 60 12; then
  echo ""
  echo "============================================"
  echo "Rollback to $TARGET_TAG successful!"
  echo ""
  echo "Deployment is manual (watchtower removed 2026-09-02)."
  echo "Nothing will move this version on its own."
  echo ""
  echo "To go back to a newer build, pin its sha and re-run:"
  echo "  export IMAGE_TAG=<target-sha>"
  echo "  $COMPOSE pull api web && $COMPOSE up -d --no-build api web"
  echo "============================================"
else
  echo ""
  echo "============================================"
  echo "ROLLBACK HEALTH CHECK FAILED!"
  echo ""
  echo "Check logs:"
  echo "  docker logs ipig-api"
  echo "  docker logs ipig-web"
  echo "============================================"
  exit 1
fi
