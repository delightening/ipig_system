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
# ⚠️ 本案例**必須真的重現事故**才算數。第一版寫成「重現不出來就印個註解跳過」，
#    等於測試可以綠著卻什麼都沒證明（CodeRabbit 於 PR #11 指出，成立）。
#    現在改成：重現不出來就 FAIL——寧可測試環境問題被吵出來，也不要假綠。
cd "$TMP/seed"
for i in 4 5; do echo "$i" > "f$i.txt"; git add .; git commit -qm "c$i"; done
git push -q origin main
cd "$TMP/work"
git fetch -q origin main --depth=1 2>/dev/null || true
if git merge-base origin/main HEAD >/dev/null 2>&1; then
  fail "事故前提未重現：origin/main...HEAD 仍算得出 merge base" "算不出" "算得出"
  echo "        （淺層 fetch 沒有截斷祖先鏈，本案例無法證明任何事——先修測試環境）"
else
  pass "已重現：舊寫法的 origin/main...HEAD 確實算不出 merge base"
fi
# 新寫法應完全不受影響——它用 BEFORE_SHA，不碰 origin/main
got="$(run BEFORE_SHA="$C2")"
[ "$got" = "$C2" ] && pass "新寫法不受 main 前進與淺層化影響" || fail "新寫法" "$C2" "$got"

echo "--- 案例 5：PR 事件（GITHUB_BASE_REF）→ 應回 origin/<base> ---"
got="$(run GITHUB_BASE_REF=main)"
[ "$got" = "origin/main" ] && pass "回 origin/main" || fail "回 origin/main" "origin/main" "$got"

echo "--- 案例 5b：PR 事件時 origin/<base> 必須是 fetch 後的新值，不能是過期的 ---"
# CodeRabbit 指出的核心：`git fetch origin <branch>` 在沒有預設 refspec 的 remote 上
# 只更新 FETCH_HEAD、不更新 origin/<branch>，於是回傳過期 base 卻毫無錯誤。
# 這裡把 refspec 拿掉來重現該環境（actions/checkout 某些設定就是如此）。
git -c init.defaultBranch=main clone -q -b main "$TMP/up.git" "$TMP/norefspec" 2>/dev/null \
  || git -c init.defaultBranch=main clone -q -b main "$TMP/upstream.git" "$TMP/norefspec"
cd "$TMP/norefspec"; git config user.email t@t; git config user.name t
git config --unset-all remote.origin.fetch
STALE="$(git rev-parse origin/main)"
cd "$TMP/seed"; echo 6 > f6.txt; git add .; git commit -qm c6; git push -q origin main
cd "$TMP/norefspec"
got="$( cd "$TMP/norefspec" && env -u BEFORE_SHA GITHUB_BASE_REF=main bash "$SCRIPT" )"
FRESH="$(git rev-parse origin/main 2>/dev/null || echo none)"
if [ "$got" != "origin/main" ]; then
  fail "無 refspec 時仍回 origin/main" "origin/main" "$got"
elif [ "$FRESH" = "$STALE" ]; then
  fail "origin/main 沒被更新（會比錯對象）" "新的 commit" "仍是 $STALE"
else
  pass "明確 refspec 確實把 origin/main 更新到最新（不再回傳過期 base）"
fi

echo "--- 案例 5c：PR 事件但 base 分支不存在 → 必須 exit 3，不可退回 HEAD^ ---"
set +e
out="$( cd "$TMP/work" && env -u BEFORE_SHA GITHUB_BASE_REF=no-such-branch bash "$SCRIPT" 2>/dev/null )"
rc=$?
set -e 2>/dev/null || true
if [ "$rc" -eq 3 ] && [ -z "$out" ]; then
  pass "exit 3 且無輸出（不 fail-open）"
else
  fail "base 不存在應 exit 3" "rc=3 且無輸出" "rc=$rc out=[$out]"
fi

echo "--- 案例 6：什麼都沒有 → 應回空字串（呼叫端 fail-open）---"
git -c init.defaultBranch=main init -q "$TMP/single"
cd "$TMP/single"; git config user.email t@t; git config user.name t
echo x > a.txt; git add .; git commit -qm only
got="$( cd "$TMP/single" && env -u GITHUB_BASE_REF -u BEFORE_SHA bash "$SCRIPT" )"
[ -z "$got" ] && pass "回空字串" || fail "回空字串" "(空)" "$got"

echo ""
if [ "$fails" -eq 0 ]; then echo "全部通過"; else echo "有案例失敗"; exit 1; fi
