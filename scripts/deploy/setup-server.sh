#!/usr/bin/env bash
set -euo pipefail

# ============================================
# iPig System: One-time production server setup
# ============================================
# Prerequisites:
#   - Docker 24.0+ and Docker Compose 2.20+
#   - Git repository cloned
#   - .env file configured
#
# Usage:
#   bash scripts/deploy/setup-server.sh
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "============================================"
echo "iPig System: Production Server Setup"
echo "============================================"
echo ""

# 1. Prompt for GHCR credentials
read -rp "GitHub username or org (GHCR_OWNER): " GHCR_OWNER
read -rsp "GitHub Personal Access Token (read:packages scope): " GHCR_TOKEN
echo ""

# 2. Login to GHCR
echo ""
echo "[1/3] Logging into GHCR..."
echo "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_OWNER" --password-stdin
echo "GHCR login successful."

# 3. Add GHCR variables to .env
echo ""
echo "[2/3] Updating .env with GHCR config..."
if ! grep -q "^GHCR_OWNER=" "$PROJECT_DIR/.env" 2>/dev/null; then
  cat >> "$PROJECT_DIR/.env" <<EOF

# =========================
# Container Registry (GHCR)
# =========================
GHCR_OWNER=$GHCR_OWNER
IMAGE_TAG=latest
EOF
  echo "  Added GHCR_OWNER to .env"
else
  echo "  GHCR_OWNER already in .env, skipping."
fi

# 2026-09-02：原步驟「Generate Watchtower API token」已移除。watchtower 服務本身
# 已從 docker-compose.prod.yml 移除（部署改為人工執行），該 token 不再有任何消費端，
# 產生它只會在 .env 留下一個沒有用途的祕密。

# 4. Pull images and start
echo ""
echo "[3/3] Pulling images and starting services..."
cd "$PROJECT_DIR"
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull api web
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build

echo ""
echo "============================================"
echo "Setup complete!"
echo ""
echo "Check status:"
echo "  docker compose -f docker-compose.yml -f docker-compose.prod.yml ps"
echo ""
echo "Health check:"
echo "  curl http://localhost:8000/api/health"
echo ""
echo "Deploy a new version (manual — no auto-update):"
echo "  export IMAGE_TAG=<target-sha>"
echo "  docker compose -f docker-compose.yml -f docker-compose.prod.yml pull api web"
echo "  docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d api web"
echo "============================================"
