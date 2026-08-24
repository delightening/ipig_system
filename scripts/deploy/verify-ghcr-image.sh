#!/usr/bin/env bash
set -euo pipefail

# ============================================
# iPig System: Verify a GHCR-built image, read-only
# ============================================
# Pulls the image CD already built & pushed for a given commit, and checks
# it actually is what it claims to be -- WITHOUT touching any running
# container, prod compose state, or the deploy lock.
#
# This exists because prod currently runs `docker compose build api` (local
# build) even though `docker-compose.prod.yml` + Watchtower + rollback.sh +
# setup-server.sh already assume the GHCR path is live. Nobody had actually
# pulled from GHCR on this machine before this script -- see PR description
# for the discrepancy this surfaced.
#
# What it checks:
#   1. The image pulls (registry reachable, tag exists, auth works).
#   2. The pulled image's baked-in GIT_SHA matches the commit you asked to
#      verify. GIT_SHA is a Rust `option_env!()` COMPILE-TIME constant (see
#      handlers/version.rs), not a runtime env var -- the final image is
#      distroless (no shell, fresh `FROM`, nothing carries over from the
#      builder stage), so this is checked by `docker create` + `docker cp`
#      the binary out and grepping it. The container is never started.
#   3. Image size vs the current local `ipig-api:latest`, so a wildly
#      different size (e.g. debug build slipped in) is visible before
#      anyone considers switching prod over.
#
# What it does NOT do (out of scope for this PR -- see PR description):
#   - Does not run the image against a real DB or verify the app actually
#     boots. That needs either a scratch Postgres + full secret set, or a
#     staging environment; neither exists on this machine today.
#   - Does not touch docker-compose.prod.yml, Watchtower, or any running
#     container. Prod keeps building locally until a human decides
#     otherwise.
#
# Usage:
#   bash scripts/deploy/verify-ghcr-image.sh [commit-sha] [service]
#
# commit-sha must be the 7-character short sha docker/metadata-action tags
# images with (`type=sha` default) -- that's the only tag CD actually
# pushes. A longer sha is truncated to 7 with a notice; anything else
# (too short, non-hex) is rejected before we waste a pull attempt.
#
# Examples:
#   bash scripts/deploy/verify-ghcr-image.sh                # HEAD, api
#   bash scripts/deploy/verify-ghcr-image.sh 3fb9922         # specific sha
#   bash scripts/deploy/verify-ghcr-image.sh 3fb9922 web     # other service
#
# Requires GHCR_OWNER in .env (or exported) and a prior
#   docker login ghcr.io
# (see scripts/deploy/setup-server.sh for the one-time login flow --
# this script deliberately does not ask for or handle credentials itself).
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

SERVICE="${2:-api}"
IMAGE_NAME="ipig-${SERVICE}"

# Binary path inside the final (distroless) image, and whether GIT_SHA is
# expected to be findable in it. web / db-backup don't consume GIT_SHA at
# all (cd.yml matrix.version_args is unset for them) -- there's nothing
# meaningful to grep for, so step 2 is skipped for those.
case "$SERVICE" in
  api) BIN_PATH_IN_IMAGE="/app/erp-backend" ;;
  outbox-worker) BIN_PATH_IN_IMAGE="/app/outbox_worker" ;;
  *) BIN_PATH_IN_IMAGE="" ;;
esac

if [ -n "${1:-}" ]; then
  case "$1" in
    [0-9a-fA-F]???????*)
      # 8+ hex chars: CD only ever tags with the first 7, truncate to match.
      TARGET_SHA="$(printf '%s' "$1" | cut -c1-7)"
      echo "NOTE: truncating '$1' to '$TARGET_SHA' (CD tags images with 7-char short shas only)."
      ;;
    [0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F])
      TARGET_SHA="$1"
      ;;
    *)
      echo "Invalid commit-sha '$1': expected 7 hex characters (CD's tag format)." >&2
      exit 2
      ;;
  esac
else
  TARGET_SHA="$(git -C "$PROJECT_DIR" rev-parse --short=7 HEAD)"
fi

if [ -f "$PROJECT_DIR/.env" ]; then
  # shellcheck disable=SC1091
  GHCR_OWNER="$(grep -E '^GHCR_OWNER=' "$PROJECT_DIR/.env" 2>/dev/null | cut -d= -f2- || true)"
fi
GHCR_OWNER="${GHCR_OWNER:-${GHCR_OWNER_OVERRIDE:-}}"

if [ -z "$GHCR_OWNER" ]; then
  echo "GHCR_OWNER not set in .env and not exported. Set it or run:" >&2
  echo "  GHCR_OWNER_OVERRIDE=<owner> bash $0 $*" >&2
  exit 2
fi

REMOTE_IMAGE="ghcr.io/${GHCR_OWNER}/${IMAGE_NAME}:${TARGET_SHA}"

# Local image naming is inconsistent across services: api / outbox-worker
# declare an explicit `image:` in docker-compose.yml (so a local build tags
# as ipig-<service>), but web / db-backup / print-pdf don't, so compose
# falls back to its default `<project-dir-name>-<service>` -- on this
# checkout that's ipig_system-<service>. Try both, first match wins.
if docker image inspect "ipig-${SERVICE}:latest" >/dev/null 2>&1; then
  LOCAL_IMAGE="ipig-${SERVICE}:latest"
elif docker image inspect "ipig_system-${SERVICE}:latest" >/dev/null 2>&1; then
  LOCAL_IMAGE="ipig_system-${SERVICE}:latest"
else
  LOCAL_IMAGE="ipig-${SERVICE}:latest"
fi

echo "============================================"
echo "iPig System: Verify GHCR image (read-only)"
echo "============================================"
echo "Target commit : $TARGET_SHA"
echo "Service       : $SERVICE"
echo "Remote image  : $REMOTE_IMAGE"
echo "Local image   : $LOCAL_IMAGE (comparison baseline, not modified)"
echo ""

echo "[1/3] Pulling $REMOTE_IMAGE ..."
if ! docker pull "$REMOTE_IMAGE"; then
  echo ""
  echo "PULL FAILED. Common causes:"
  echo "  - Not logged in: docker login ghcr.io -u <user> --password-stdin"
  echo "  - CD hasn't built this sha yet (check .github/workflows/cd.yml run for $TARGET_SHA)"
  echo "  - Wrong GHCR_OWNER (currently: $GHCR_OWNER)"
  exit 1
fi

echo ""
echo "[2/3] Checking baked-in GIT_SHA ..."
# GIT_SHA is read via Rust's `option_env!("GIT_SHA")` in handlers/version.rs --
# a COMPILE-TIME macro, baked into the binary's rodata by rustc. It is NOT a
# runtime env var: backend/Dockerfile sets `ENV GIT_SHA=...` only in the
# `builder` stage (to make it visible to `cargo build`), and the final image
# is `gcr.io/distroless/cc-debian12` -- a fresh `FROM`, so nothing from
# earlier stages carries over, and it has no shell to inspect anyway
# (`docker run --entrypoint sh` fails outright on distroless).
# So checking this means extracting the binary and grepping it -- never
# running it, never touching a shell inside the image.
# SHA_STATE distinguishes "we checked and it matches" from "there was
# nothing to check" -- collapsing both into one boolean is exactly what
# let the final summary claim "GIT_SHA matches" for web/db-backup, where
# no check ever ran. States: matched | skipped | mismatch | extract_failed.
SHA_STATE="mismatch"
if [ -z "$BIN_PATH_IN_IMAGE" ]; then
  echo "  SKIPPED: '$SERVICE' doesn't consume GIT_SHA at build time (cd.yml matrix.version_args"
  echo "  is only set for api / outbox-worker) -- nothing baked in to check."
  SHA_STATE="skipped"
else
  CONTAINER_ID="$(docker create "$REMOTE_IMAGE")"
  if BIN_PATH="$(mktemp)" && docker cp "${CONTAINER_ID}:${BIN_PATH_IN_IMAGE}" "$BIN_PATH" 2>/dev/null; then
    if grep -qa "$TARGET_SHA" "$BIN_PATH"; then
      echo "  OK: binary contains the string '$TARGET_SHA' (GIT_SHA baked in as expected)."
      SHA_STATE="matched"
    else
      echo "  MISMATCH: binary does not contain '$TARGET_SHA'."
      echo "  Either GIT_SHA wasn't passed at build time, or this image was built from a different commit."
      echo "  Do not treat this image as verified for that commit."
    fi
    rm -f "$BIN_PATH"
  else
    echo "  WARNING: could not extract $BIN_PATH_IN_IMAGE from the image."
    SHA_STATE="extract_failed"
  fi
  docker rm "$CONTAINER_ID" >/dev/null
fi

echo ""
echo "[3/3] Size comparison vs local build ..."
REMOTE_SIZE="$(docker image inspect "$REMOTE_IMAGE" --format '{{.Size}}')"
if LOCAL_SIZE="$(docker image inspect "$LOCAL_IMAGE" --format '{{.Size}}' 2>/dev/null)"; then
  REMOTE_MB=$((REMOTE_SIZE / 1024 / 1024))
  LOCAL_MB=$((LOCAL_SIZE / 1024 / 1024))
  echo "  Remote (GHCR): ${REMOTE_MB}MB"
  echo "  Local  (this machine, $LOCAL_IMAGE): ${LOCAL_MB}MB"
  DIFF=$((REMOTE_MB > LOCAL_MB ? REMOTE_MB - LOCAL_MB : LOCAL_MB - REMOTE_MB))
  if [ "$DIFF" -gt 50 ]; then
    echo "  NOTE: >50MB difference -- worth understanding why before relying on either build path."
  fi
else
  echo "  No local '$LOCAL_IMAGE' to compare against (skip)."
fi

echo ""
echo "============================================"
case "$SHA_STATE" in
  matched)
    echo "Result: GHCR image for $TARGET_SHA verified (pulls, GIT_SHA matches)."
    ;;
  skipped)
    echo "Result: GHCR image for $TARGET_SHA pulls successfully. GIT_SHA check N/A for '$SERVICE'"
    echo "(not baked in for this service -- see step 2 above)."
    ;;
  *)
    echo "Result: GHCR image pulled but GIT_SHA could not be confirmed. See warnings above."
    ;;
esac
echo "NOT deployed -- prod is untouched. This only confirms the image is real and correct."
echo "============================================"

[ "$SHA_STATE" = "matched" ] || [ "$SHA_STATE" = "skipped" ]
