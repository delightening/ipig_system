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
echo "[1/4] Logging into GHCR..."
echo "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_OWNER" --password-stdin
echo "GHCR login successful."

# 3. Add GHCR variables to .env
echo ""
echo "[2/4] Updating .env with GHCR config..."
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

# 4. Generate Watchtower API token
echo ""
echo "[3/4] Generating Watchtower API token..."
if ! grep -q "^WATCHTOWER_API_TOKEN=" "$PROJECT_DIR/.env" 2>/dev/null; then
  WATCHTOWER_TOKEN=$(openssl rand -hex 32)
  cat >> "$PROJECT_DIR/.env" <<EOF

# =========================
# Watchtower
# =========================
WATCHTOWER_API_TOKEN=$WATCHTOWER_TOKEN
DEPLOY_NOTIFY_EMAIL=
EOF
  echo "  Watchtower API token generated and added to .env"
  echo "  Token: $WATCHTOWER_TOKEN"
  echo "  (save this for manual trigger use)"
else
  echo "  WATCHTOWER_API_TOKEN already in .env, skipping."
fi

# 5. Pull images and start
echo ""
echo "[4/4] Pulling images and starting services..."
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
echo "Manual trigger update:"
echo "  curl -H 'Authorization: Bearer <WATCHTOWER_API_TOKEN>' http://localhost:8090/v1/update"
echo "============================================"
