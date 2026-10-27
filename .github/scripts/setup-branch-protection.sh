#!/usr/bin/env bash
# 為 main 建立「required status checks」ruleset，並開啟 repo 的 auto-merge 功能。
# 這是 Dependabot 自動合併的前置條件（見 .github/DEPENDABOT_AUTOMERGE.md）：
# 沒有 required checks，`gh pr merge --auto` 不是立刻合併就是報錯。
#
# 執行前提：gh CLI 以 repo 擁有者（delightening）登入——設定 ruleset 需要 admin 權限。
#
# 用法：
#   bash .github/scripts/setup-branch-protection.sh                  # dry-run：只印出將送出的內容
#   bash .github/scripts/setup-branch-protection.sh --apply          # 開 auto-merge + 建立/更新 ruleset（kill switch 仍關）
#   bash .github/scripts/setup-branch-protection.sh --apply --enable # 另外把 DEPENDABOT_AUTOMERGE 設為 true（正式啟用）
#
# 本腳本冪等：ruleset 以名稱辨識，已存在就 PUT 更新，不會重複建立。

set -euo pipefail

REPO="${REPO:-delightening/ipig_system}"
OWNER="${REPO%%/*}"
RULESET_NAME="main-required-checks"

APPLY=false
ENABLE=false
for arg in "$@"; do
  case "$arg" in
    --apply) APPLY=true ;;
    --enable) ENABLE=true ;;
    *) echo "未知參數：$arg" >&2; exit 2 ;;
  esac
done
if $ENABLE && ! $APPLY; then
  echo "--enable 必須搭配 --apply" >&2
  exit 2
fi

# ── required checks ───────────────────────────────────────────────────────────
# 名稱必須與 ci.yml 各 job 的 `name:` 逐字相同（含 emoji 與空白）；改 ci.yml 的 job 名稱
# 時要同步改這裡並重跑本腳本，否則 required check 永遠等不到、PR 卡死。
#
# 為何列了「依路徑條件跳過」的 job（cargo test、E2E 等）：job 因 `if` 被 skip 時，
# GitHub 視為通過；只有真的跑了又失敗才會擋。所以前端 only 的 PR 不會被後端 job 卡住。
#
# 刻意不列：
#   - semgrep SAST：continue-on-error 的 non-blocking job，且 Dependabot PR 一律跳過
#   - 📊 Backend: coverage N/4（matrix 分片）：權威 job 是 ratchet 那支「coverage (llvm-cov)」
#   - 🛡️ Frontend: docker-entrypoint.sh、🐳 Trivy backup image：與依賴更新關聯低，
#     且前者 needs frontend-check，失敗時已由 frontend-check 擋住
CHECKS=(
  "🧭 Changed paths"
  "🔑 Security: secret scanning (gitleaks)"
  "🔑 Security: pii-scan 自我測試"
  "🛡️ Guards (6-in-1)"
  "Backend: cargo check"
  "Backend: cargo test"
  "Backend: clippy"
  "📊 Backend: coverage (llvm-cov)"
  "🔒 Security: cargo audit"
  "🛡️ Security: cargo deny"
  "Frontend: tsc + eslint + vitest"
  "🔒 Security: pnpm audit"
  "🔒 Security: npm 依賴掃描"
  "🔒 Security: Python 依賴掃描"
  "🔗 print-pdf: Dockerfile/requirements 版本一致性"
  "🩺 print-pdf: 容器啟動 smoke test"
  "🐳 Security: Trivy container scan"
  "🧪 E2E: Playwright"
)

CHECKS_JSON=$(printf '%s\n' "${CHECKS[@]}" | jq -R . | jq -s .)

# ── bypass 設定 ───────────────────────────────────────────────────────────────
# 1. RepositoryRole 5 = Admin，bypass_mode=pull_request：擁有者可在 PR 上強制合併
#    （緊急修復用），但不能繞過 PR 直接 push。
# 2. Integration 15368 = github-actions：ci.yml 的 coverage ratchet 兩步會在 main push 後
#    用 GITHUB_TOKEN 直接 `git push` 回 main（[skip ci] commit）。沒有這個 bypass，
#    required checks 會擋掉那次 push，main 的 CI 會在 ratchet 步驟變紅。
#    ⚠️ 此 bypass 是否對 GITHUB_TOKEN 生效**尚未實測**；上線後第一個 main push 要看
#    ratchet 步驟有沒有被拒（見 DEPENDABOT_AUTOMERGE.md「已知風險 2」）。
RULESET_BODY=$(jq -n --arg name "$RULESET_NAME" --argjson checks "$CHECKS_JSON" '{
  name: $name,
  target: "branch",
  enforcement: "active",
  conditions: { ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] } },
  bypass_actors: [
    { actor_id: 5,     actor_type: "RepositoryRole", bypass_mode: "pull_request" },
    { actor_id: 15368, actor_type: "Integration",    bypass_mode: "always" }
  ],
  rules: [
    {
      type: "required_status_checks",
      parameters: {
        strict_required_status_checks_policy: false,
        do_not_enforce_on_create: false,
        required_status_checks: ($checks | map({ context: . }))
      }
    }
  ]
}')

echo "=== 目標 repo：$REPO ==="
echo "--- 將設定的 ruleset（$RULESET_NAME）---"
echo "$RULESET_BODY" | jq .
echo "--- 將執行 ---"
echo "1. PATCH repos/$REPO  allow_auto_merge=true"
echo "2. 建立或更新 ruleset：$RULESET_NAME"
if $ENABLE; then
  echo "3. gh variable set DEPENDABOT_AUTOMERGE=true（正式啟用）"
else
  echo "3. （略過）DEPENDABOT_AUTOMERGE 維持現狀，未加 --enable"
fi

if ! $APPLY; then
  echo ""
  echo "dry-run 結束，未做任何變更。確認無誤後加 --apply。"
  exit 0
fi

# 對外寫入前先確認執行身分：gh 可能登入多個帳號，active 帳號不一定是 repo 擁有者。
ACTOR=$(gh api user --jq .login)
if [ "$ACTOR" != "$OWNER" ]; then
  echo "目前 gh 登入身分是「$ACTOR」，但 $REPO 需要「$OWNER」。" >&2
  echo "請先執行：gh auth switch -u $OWNER" >&2
  exit 1
fi

echo ""
echo "── 1/3 開啟 allow_auto_merge ──"
gh api -X PATCH "repos/$REPO" -F allow_auto_merge=true --jq '"allow_auto_merge=\(.allow_auto_merge)"'

echo "── 2/3 建立/更新 ruleset ──"
EXISTING_ID=$(gh api "repos/$REPO/rulesets" --jq ".[] | select(.name == \"$RULESET_NAME\") | .id" | head -n1)
if [ -n "$EXISTING_ID" ]; then
  echo "$RULESET_BODY" | gh api -X PUT "repos/$REPO/rulesets/$EXISTING_ID" --input - --jq '"updated ruleset id=\(.id)"'
else
  echo "$RULESET_BODY" | gh api -X POST "repos/$REPO/rulesets" --input - --jq '"created ruleset id=\(.id)"'
fi

echo "── 3/3 kill switch ──"
if $ENABLE; then
  gh variable set DEPENDABOT_AUTOMERGE --repo "$REPO" --body true
  echo "DEPENDABOT_AUTOMERGE=true（已啟用）"
else
  echo "未加 --enable：DEPENDABOT_AUTOMERGE 維持原狀。確認 ruleset 生效後再加 --apply --enable。"
fi

echo ""
echo "=== 完成。驗證：gh api repos/$REPO/rulesets --jq '.[].name' ==="
