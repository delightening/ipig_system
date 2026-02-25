#!/usr/bin/env bash
set -euo pipefail

# ============================================
# iPig System: Rollback to a specific image tag
# ============================================
# Usage:
#   bash scripts/deploy/rollback.sh <commit-sha>
#
# This will:
#   1. Stop Watchtower (prevent auto-update)
#   2. Pull the specified image versions
#   3. Restart api + web with pinned versions
#   4. Run health checks
#   5. Watchtower stays stopped until you resume
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

# 1. Stop Watchtower
echo "[1/5] Stopping Watchtower..."
$COMPOSE stop watchtower 2>/dev/null || true

# 2. Set image tag
echo "[2/5] Setting IMAGE_TAG=$TARGET_TAG..."
export IMAGE_TAG="$TARGET_TAG"

# 3. Pull specific version
echo "[3/5] Pulling images..."
$COMPOSE pull api web

# 4. Restart services
echo "[4/5] Restarting services..."
$COMPOSE up -d --no-build api web

# 5. Health check
echo "[5/5] Running health checks..."
if bash "$SCRIPT_DIR/healthcheck.sh" 60 12; then
  echo ""
  echo "============================================"
  echo "Rollback to $TARGET_TAG successful!"
  echo ""
  echo "Watchtower is STOPPED to prevent auto-update."
  echo "To resume auto-updates:"
  echo "  export IMAGE_TAG=latest"
  echo "  $COMPOSE up -d watchtower"
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
