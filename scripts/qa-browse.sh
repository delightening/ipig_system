#!/usr/bin/env bash
# =============================================================================
# qa-browse.sh — QA/Design-review helper for gstack browse on Windows
# =============================================================================
#
# Problem: On Windows, each `$B <command>` call spawns a new browser server,
# so cookies/login/tabs don't persist between calls.
#
# Solution: This script reads a JSON chain-command file and executes everything
# in a SINGLE `browse chain` call, preserving browser state throughout.
#
# Usage:
#   bash scripts/qa-browse.sh scripts/qa-chains/login.json
#   bash scripts/qa-browse.sh scripts/qa-chains/full-qa.json
#   bash scripts/qa-browse.sh scripts/qa-chains/design-review.json
#
# Environment variables (optional):
#   BROWSE_BIN    — path to browse binary (default: ~/.claude/skills/gstack/browse/dist/browse)
#   BASE_URL      — application base URL (default: http://localhost:5173)
#   SCREENSHOT_DIR — where to save screenshots (default: ./qa-screenshots)
# =============================================================================

set -euo pipefail

# --- Configuration -----------------------------------------------------------
BROWSE_BIN="${BROWSE_BIN:-$HOME/.claude/skills/gstack/browse/dist/browse}"
BASE_URL="${BASE_URL:-http://localhost:5173}"
SCREENSHOT_DIR="${SCREENSHOT_DIR:-./qa-screenshots}"
CHAIN_FILE="${1:-}"

# --- Validation --------------------------------------------------------------
if [[ -z "$CHAIN_FILE" ]]; then
  echo "ERROR: No chain file specified."
  echo "Usage: bash scripts/qa-browse.sh <chain-file.json>"
  echo ""
  echo "Available chain files:"
  ls scripts/qa-chains/*.json 2>/dev/null | sed 's/^/  /'
  exit 1
fi

if [[ ! -f "$CHAIN_FILE" ]]; then
  echo "ERROR: Chain file not found: $CHAIN_FILE"
  exit 1
fi

if [[ ! -f "$BROWSE_BIN" ]]; then
  echo "ERROR: Browse binary not found at: $BROWSE_BIN"
  echo "Set BROWSE_BIN environment variable to the correct path."
  exit 1
fi

# --- Prepare screenshot directory --------------------------------------------
mkdir -p "$SCREENSHOT_DIR"

# --- Variable substitution in chain file -------------------------------------
# Replace placeholders like {{BASE_URL}} and {{SCREENSHOT_DIR}} with actual values
CHAIN_CONTENT=$(cat "$CHAIN_FILE")
CHAIN_CONTENT="${CHAIN_CONTENT//\{\{BASE_URL\}\}/$BASE_URL}"
CHAIN_CONTENT="${CHAIN_CONTENT//\{\{SCREENSHOT_DIR\}\}/$SCREENSHOT_DIR}"

# Write resolved chain to a temp file
TEMP_CHAIN=$(mktemp /tmp/qa-chain-XXXXXX.json)
echo "$CHAIN_CONTENT" > "$TEMP_CHAIN"
trap 'rm -f "$TEMP_CHAIN"' EXIT

# --- Execute -----------------------------------------------------------------
echo "============================================"
echo " QA Browse Runner"
echo "============================================"
echo " Chain file : $CHAIN_FILE"
echo " Base URL   : $BASE_URL"
echo " Screenshots: $SCREENSHOT_DIR"
echo " Browse bin : $BROWSE_BIN"
echo "============================================"
echo ""

"$BROWSE_BIN" chain "$TEMP_CHAIN"

echo ""
echo "============================================"
echo " Done. Screenshots saved to: $SCREENSHOT_DIR"
echo "============================================"
