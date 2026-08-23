#!/usr/bin/env bash
# 解析「這次變更該跟誰比」的 git 基準，印到 stdout；取不到就印空字串（呼叫端 fail-open）。
#
# 為什麼要抽成共用腳本：guards job 裡有兩處各自寫了一份基準判斷，兩份都踩到同一個坑，
# 但只有其中一處被發現。判斷邏輯放一起，修一次就兩處都好，也才有地方寫測試。
#
# ⚠️ 呼叫端**不可以**再自己跑 `git fetch --depth=1`：
#    guards job 的 checkout 已設 `fetch-depth: 0` 取回完整歷史，淺層 fetch 會把 repo
#    重新變成 shallow、截斷祖先鏈，之後所有 `A...HEAD`（三點＝需要 merge base）
#    都會 `fatal: no merge base`。
#
# 2026-08-23 實際事故：連續合併四支 PR，CI 排隊 23 分鐘期間 main 又前進了三個 commit。
# 等 #3 的 runner 真正開跑時，`origin/main` 已經是 #6 的 commit、與該次 push 的 HEAD
# 不同；再加上淺層 fetch 讓 `origin/main` 沒有父節點，merge base 直接算不出來，
# 四支已合併的 PR 在 main 上整批紅——而它們的內容一行問題都沒有。
#
# 判斷順序：
#   1. pull_request 事件 → 用 base 分支（GITHUB_BASE_REF）
#   2. push 事件 → 用 github.event.before（這次推送前的 tip）。這才是「這次推了什麼」
#      的正確答案，而且**不受 main 之後前進影響**——用 origin/main 才會賽跑。
#   3. 前兩者都取不到 → HEAD^
#   4. 連 HEAD^ 都沒有（初始 commit）→ 印空字串，呼叫端跳過
set -uo pipefail

ZERO="0000000000000000000000000000000000000000"

have() { git rev-parse -q --verify "$1^{commit}" >/dev/null 2>&1; }

# 1. PR：跟 base 分支比
if [ -n "${GITHUB_BASE_REF:-}" ]; then
  # 不帶 --depth，理由見檔頭
  git fetch origin "$GITHUB_BASE_REF" >/dev/null 2>&1 || true
  if have "origin/${GITHUB_BASE_REF}"; then
    echo "origin/${GITHUB_BASE_REF}"
    exit 0
  fi
fi

# 2. push：跟推送前的 tip 比
if [ -n "${BEFORE_SHA:-}" ] && [ "$BEFORE_SHA" != "$ZERO" ] && have "$BEFORE_SHA"; then
  echo "$BEFORE_SHA"
  exit 0
fi

# 3. 退而求其次
if have "HEAD^"; then
  echo "HEAD^"
  exit 0
fi

# 4. 沒有基準
echo ""
