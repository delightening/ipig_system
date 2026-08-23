#!/usr/bin/env bash
# resolve-diff-base.sh 的回歸測試。在臨時 git repo 上建真實歷史來測，不 mock git。
#
# 為什麼要有這支：2026-08-23 的事故是「淺層 fetch 截斷祖先鏈」，症狀只在
# 「main 於 CI 排隊期間前進」時才出現——那是既有測試完全涵蓋不到的時序條件。
# 這裡直接把那個時序做出來（案例 4）。
set -uo pipefail

SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/resolve-diff-base.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
pass() { echo "  PASS  $1"; }
fail() { echo "  FAIL  $1"; echo "        預期 [$2] 實得 [$3]"; fails=1; }

# ── 建一個有 remote 的假 repo：upstream 當 origin，work 當 CI 工作區 ──
# ⚠️ bare repo 的 HEAD 必須指到 main，否則 clone 會「remote HEAD refers to
#    nonexistent ref, unable to checkout」——工作區沒有 HEAD，整批測試假失敗。
git -c init.defaultBranch=main init -q --bare "$TMP/upstream.git"
git -c init.defaultBranch=main init -q "$TMP/seed"
cd "$TMP/seed"
git config user.email t@t; git config user.name t
for i in 1 2 3; do echo "$i" > "f$i.txt"; git add .; git commit -qm "c$i"; done
git branch -M main
git remote add origin "$TMP/upstream.git"
git push -q origin main

git clone -q -b main "$TMP/upstream.git" "$TMP/work"
cd "$TMP/work"
git config user.email t@t; git config user.name t

C3="$(git rev-parse HEAD)"
C2="$(git rev-parse HEAD^)"
if [ -z "$C3" ] || [ -z "$C2" ]; then
  echo "ABORT: 測試環境沒建起來（HEAD/HEAD^ 取不到），先修測試再談結果"
  exit 1
fi

run() { ( cd "$TMP/work" && env -u GITHUB_BASE_REF -u BEFORE_SHA "$@" bash "$SCRIPT" ) ; }

echo "--- 案例 1：push 事件，BEFORE_SHA 有效 → 應回 BEFORE_SHA ---"
got="$(run BEFORE_SHA="$C2")"
[ "$got" = "$C2" ] && pass "回 BEFORE_SHA" || fail "回 BEFORE_SHA" "$C2" "$got"

echo "--- 案例 2：BEFORE_SHA 為全零（新分支首推）→ 應退回 HEAD^ ---"
got="$(run BEFORE_SHA=0000000000000000000000000000000000000000)"
[ "$got" = "HEAD^" ] && pass "退回 HEAD^" || fail "退回 HEAD^" "HEAD^" "$got"

echo "--- 案例 3：BEFORE_SHA 指向不存在的 commit → 應退回 HEAD^，不得炸 ---"
got="$(run BEFORE_SHA=deadbeefdeadbeefdeadbeefdeadbeefdeadbeef)"
[ "$got" = "HEAD^" ] && pass "退回 HEAD^" || fail "退回 HEAD^" "HEAD^" "$got"

echo "--- 案例 4（事故重現）：CI 排隊期間 main 前進，且 repo 被淺層化 ---"
# 模擬：工作區停在 C3，但 origin/main 已被推到 C5；再用淺層 fetch 截斷祖先
cd "$TMP/seed"
for i in 4 5; do echo "$i" > "f$i.txt"; git add .; git commit -qm "c$i"; done
git push -q origin main
cd "$TMP/work"
git fetch -q origin main --depth=1 2>/dev/null || true
# 這正是舊寫法會做的事：拿 origin/main 當基準
if git merge-base origin/main HEAD >/dev/null 2>&1; then
  echo "        （註：本機 git 未產生淺層截斷，事故條件未完全重現）"
else
  pass "已重現：origin/main...HEAD 確實算不出 merge base"
fi
# 新寫法應完全不受影響——它用 BEFORE_SHA，不碰 origin/main
got="$(run BEFORE_SHA="$C2")"
[ "$got" = "$C2" ] && pass "新寫法不受 main 前進與淺層化影響" || fail "新寫法" "$C2" "$got"

echo "--- 案例 5：PR 事件（GITHUB_BASE_REF）→ 應回 origin/<base> ---"
got="$(run GITHUB_BASE_REF=main)"
[ "$got" = "origin/main" ] && pass "回 origin/main" || fail "回 origin/main" "origin/main" "$got"

echo "--- 案例 6：什麼都沒有 → 應回空字串（呼叫端 fail-open）---"
git -c init.defaultBranch=main init -q "$TMP/single"
cd "$TMP/single"; git config user.email t@t; git config user.name t
echo x > a.txt; git add .; git commit -qm only
got="$( cd "$TMP/single" && env -u GITHUB_BASE_REF -u BEFORE_SHA bash "$SCRIPT" )"
[ -z "$got" ] && pass "回空字串" || fail "回空字串" "(空)" "$got"

echo ""
if [ "$fails" -eq 0 ]; then echo "全部通過"; else echo "有案例失敗"; exit 1; fi
