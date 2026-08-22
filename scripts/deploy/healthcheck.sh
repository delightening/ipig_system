#!/usr/bin/env bash
set -euo pipefail

# ============================================
# iPig System: Post-deployment health check
# ============================================
# Verifies API and Web services are healthy.
# Exit codes: 0 = all healthy, 1 = unhealthy
#
# Usage:
#   bash scripts/deploy/healthcheck.sh [MAX_WAIT_SECONDS] [RETRIES]
# ============================================

MAX_WAIT=${1:-60}
RETRIES=${2:-12}
INTERVAL=$((MAX_WAIT / RETRIES))
if [ "$INTERVAL" -lt 1 ]; then INTERVAL=1; fi

API_URL="${API_HEALTH_URL:-http://localhost:8000/api/health}"
WEB_URL="${WEB_HEALTH_URL:-http://localhost:8080/}"

check_api() {
  local response
  response=$(curl -sf --max-time 3 "$API_URL" 2>/dev/null) || return 1
  echo "$response" | grep -q '"healthy"' || return 1
  return 0
}

check_web() {
  curl -sf --max-time 3 "$WEB_URL" >/dev/null 2>&1 || return 1
  return 0
}

echo "[Health Check] Waiting for services..."
echo "  API: $API_URL"
echo "  Web: $WEB_URL"
echo "  Max wait: ${MAX_WAIT}s (${RETRIES} retries, ${INTERVAL}s interval)"
echo ""

api_healthy=false
web_healthy=false

for i in $(seq 1 "$RETRIES"); do
  printf "  Attempt %d/%d: " "$i" "$RETRIES"

  if ! $api_healthy && check_api; then
    api_healthy=true
  fi
  if $api_healthy; then printf "API=OK "; else printf "API=WAIT "; fi

  if ! $web_healthy && check_web; then
    web_healthy=true
  fi
  if $web_healthy; then printf "Web=OK"; else printf "Web=WAIT"; fi

  echo ""

  if $api_healthy && $web_healthy; then
    echo ""
    echo "[Health Check] All services healthy!"
    version=$(curl -sf --max-time 3 "$API_URL" 2>/dev/null | grep -o '"version":"[^"]*"' | cut -d'"' -f4) || version="unknown"
    echo "  API version: $version"
    exit 0
  fi

  sleep "$INTERVAL"
done

echo ""
echo "[Health Check] FAILED - services not healthy within ${MAX_WAIT}s"
if ! $api_healthy; then
  echo "  API: UNHEALTHY"
  curl -sf --max-time 3 "$API_URL" 2>&1 || echo "  (no response)"
fi
if ! $web_healthy; then
  echo "  Web: UNHEALTHY"
fi
exit 1
