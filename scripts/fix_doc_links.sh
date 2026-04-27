#!/usr/bin/env bash
# Link-fix pass for docs/ reorganization
# Run AFTER reorganize_docs.sh
set -euo pipefail

# ── Helper: in-place sed (cross-platform bash) ───────────────────────
# GNU sed: -i''; BSD/macOS: -i ''
SED() { sed -i "$@"; }

# ── Active files only (not archive — those are frozen) ────────────────
ACTIVE=(
  docs/README.md
  docs/TODO.md
  docs/PROGRESS.md
  docs/spec/README.md
  docs/spec/architecture/*.md
  docs/spec/modules/*.md
  docs/spec/guides/*.md
  docs/dev/*.md
  docs/dev/e2e/*.md
  docs/ops/*.md
  docs/runbooks/*.md
  docs/db/*.md
  docs/security/*.md
  docs/deploy/*.md
  docs/deploy/nas-migration/*.md
  docs/user/*.md
  README.md
  CLAUDE.md
  DESIGN.md
)

echo "🔧 Pass 1: directory-name renames (same depth, just renamed)"

for f in "${ACTIVE[@]}"; do
  [ -f "$f" ] || continue
  SED \
    -e 's|Profiling_Spec/archive/|archive/legacy/spec_v2/|g' \
    -e 's|Profiling_Spec/guides/|spec/guides/|g' \
    -e 's|Profiling_Spec/modules/|spec/modules/|g' \
    -e 's|Profiling_Spec/|spec/|g' \
    -e 's|security-compliance/|security/|g' \
    -e 's|database/|db/|g' \
    -e 's|operations/|ops/|g' \
    -e 's|nas-migration/|deploy/nas-migration/|g' \
    "$f"
done

echo "✅ Pass 1 done"

echo "🔧 Pass 2: file-specific depth fixes for files that MOVED from docs/ root"

# ── docs/spec/README.md ───────────────────────────────────────────────
# Was: docs/Profiling_Spec/README.md  → now docs/spec/README.md (same depth)
# Internal links: ./01_*.md → ./architecture/01_*.md etc.
f="docs/spec/README.md"
SED \
  -e 's|\./01_|\./architecture/01_|g' \
  -e 's|\./02_|\./architecture/02_|g' \
  -e 's|\./03_|\./architecture/03_|g' \
  -e 's|\./04_|\./architecture/04_|g' \
  -e 's|\./05_|\./architecture/05_|g' \
  -e 's|\./06_|\./architecture/06_|g' \
  -e 's|\./07_|\./architecture/07_|g' \
  -e 's|\./08_|\./modules/08_|g' \
  -e 's|\./09_|\./architecture/09_|g' \
  -e 's|\./database_erd\.md|./architecture/database_erd.md|g' \
  -e 's|\.\./QUICK_START\.md|../user/QUICK_START.md|g' \
  -e 's|\.\./DEPLOYMENT\.md|../deploy/DEPLOYMENT.md|g' \
  -e 's|\.\./ARCHITECTURE\.md|./architecture/ARCHITECTURE.md|g' \
  "$f"
echo "  fixed: $f"

# ── docs/ops/OPERATIONS.md ───────────────────────────────────────────
# Was: docs/operations/OPERATIONS.md → now docs/ops/OPERATIONS.md (same depth)
f="docs/ops/OPERATIONS.md"
SED \
  -e 's|\.\./DEPLOYMENT\.md|../deploy/DEPLOYMENT.md|g' \
  -e 's|\.\./ARCHITECTURE\.md|../spec/architecture/ARCHITECTURE.md|g' \
  -e 's|\.\./db/DB_ROLLBACK\.md|../db/DB_ROLLBACK.md|g' \
  -e 's|\.\./database/DB_ROLLBACK\.md|../db/DB_ROLLBACK.md|g' \
  "$f"
echo "  fixed: $f"

# ── docs/deploy/DEPLOYMENT.md ────────────────────────────────────────
# Was: docs/DEPLOYMENT.md (depth 1) → now docs/deploy/DEPLOYMENT.md (depth 2)
# Added 1 level, so all same-level refs get ../ prefix, parent refs get one more ../
f="docs/deploy/DEPLOYMENT.md"
SED \
  -e 's|\.\./README\.md|../../README.md|g' \
  -e 's|QUICK_START\.md|../user/QUICK_START.md|g' \
  -e 's|USER_GUIDE\.md|../user/USER_GUIDE.md|g' \
  -e 's|ARCHITECTURE\.md|../spec/architecture/ARCHITECTURE.md|g' \
  -e 's|ops/OPERATIONS\.md|OPERATIONS.md|g' \
  "$f"
echo "  fixed: $f"

# ── docs/user/QUICK_START.md ─────────────────────────────────────────
# Was: docs/QUICK_START.md (depth 1) → now docs/user/QUICK_START.md (depth 2)
f="docs/user/QUICK_START.md"
SED \
  -e 's|\.\./README\.md|../../README.md|g' \
  -e 's|DEPLOYMENT\.md|../deploy/DEPLOYMENT.md|g' \
  -e 's|USER_GUIDE\.md|USER_GUIDE.md|g' \
  -e 's|ops/WINDOWS_BUILD\.md|../ops/WINDOWS_BUILD.md|g' \
  -e 's|ops/ENV_AND_DB\.md|../ops/ENV_AND_DB.md|g' \
  -e 's|dev/e2e/FLOW\.md|../dev/e2e/FLOW.md|g' \
  -e 's|dev/e2e/README\.md|../dev/e2e/README.md|g' \
  -e 's|e2e/FLOW\.md|../dev/e2e/FLOW.md|g' \
  -e 's|e2e/README\.md|../dev/e2e/README.md|g' \
  "$f"
echo "  fixed: $f"

# ── docs/security/THREAT_MODEL.md ────────────────────────────────────
# Was: docs/THREAT_MODEL.md (depth 1) → now docs/security/THREAT_MODEL.md (depth 2)
# This file had no external links (checked above)

# ── docs/spec/architecture/ARCHITECTURE.md ───────────────────────────
# Was: docs/ARCHITECTURE.md (depth 1) → now 3 levels deep
# Checked: no links in this file

# ── docs/ops/VPS_CHEATSHEET.md ───────────────────────────────────────
# Was: docs/VPS_CHEATSHEET.md (depth 1) → now docs/ops/ (depth 2)
# Checked: no links in this file

# ── docs/ops/OBSERVABILITY_PLAN.md ───────────────────────────────────
# Was: docs/OBSERVABILITY_PLAN.md (depth 1) → now docs/ops/ (depth 2)
# No external links

# ── docs/spec/modules/AI_API_GUIDE.md ────────────────────────────────
# Was: docs/walkthrough_ai_api.md (depth 1) → now docs/spec/modules/ (depth 3)
# Check and fix
f="docs/spec/modules/AI_API_GUIDE.md"
if [ -f "$f" ]; then
  SED \
    -e 's|\.\./README\.md|../../../README.md|g' \
    -e 's|\.\./QUICK_START\.md|../../user/QUICK_START.md|g' \
    "$f"
  echo "  fixed: $f"
fi

# ── docs/spec/modules/MCP_REVIEW_SERVER.md ───────────────────────────
# Was: docs/MCP_Review_Server.md (depth 1) → now docs/spec/modules/ (depth 3)
f="docs/spec/modules/MCP_REVIEW_SERVER.md"
if [ -f "$f" ]; then
  SED \
    -e 's|\.\./README\.md|../../../README.md|g' \
    "$f"
  echo "  fixed: $f"
fi

echo "✅ Pass 2 done"

echo "🔧 Pass 3: fix root README.md paths (docs/ subdir refs changed)"
f="README.md"
SED \
  -e 's|docs/security-compliance/security\.md|docs/security/security.md|g' \
  -e 's|docs/Profiling_Spec/|docs/spec/|g' \
  -e 's|docs/operations/OPERATIONS\.md|docs/ops/OPERATIONS.md|g' \
  -e 's|docs/operations/COMPOSE\.md|docs/ops/COMPOSE.md|g' \
  -e 's|docs/QUICK_START\.md|docs/user/QUICK_START.md|g' \
  -e 's|docs/USER_GUIDE\.md|docs/user/USER_GUIDE.md|g' \
  -e 's|docs/DEPLOYMENT\.md|docs/deploy/DEPLOYMENT.md|g' \
  -e 's|docs/ARCHITECTURE\.md|docs/spec/architecture/ARCHITECTURE.md|g' \
  -e 's|docs/TODO\.md|docs/TODO.md|g' \
  "$f"
echo "  fixed: $f"

echo "✅ Pass 3 done"

echo ""
echo "🎉 All link-fix passes complete."
