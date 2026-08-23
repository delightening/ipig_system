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
#
# ⚠️ 必須帶**明確 refspec**寫進 refs/remotes/origin/<branch>，不能只 `git fetch origin <branch>`。
#    後者是否順帶更新 remote-tracking ref，取決於該 remote 有沒有預設 refspec——
#    2026-08-23 實測（CodeRabbit 於 PR #11 指出，成立）：
#      有 `+refs/heads/*:refs/remotes/origin/*` → origin/main 會更新
#      沒有（actions/checkout 某些設定就是如此）→ **只更新 FETCH_HEAD，origin/main 停在舊值**
#    第二種情況下 `have origin/<branch>` 仍為真，於是回傳一個**過期的 base**，
#    而且不會有任何錯誤——最壞的失敗模式：看起來成功、比錯對象。
#
# ⚠️ 這裡也**不可以 fail-open**。fail-open 只適用於「本來就沒有 base」（初始 commit）；
#    PR 事件明明有 base 卻取不到，是環境出問題，靜默退回 HEAD^ 會讓守門比錯東西
#    （PR 的 HEAD^ 通常是 base 分支的某個 commit，不是 merge base）。
#    故此處失敗直接 exit 3，由呼叫端讓 CI 紅。
if [ -n "${GITHUB_BASE_REF:-}" ]; then
  # 不帶 --depth，理由見檔頭
  if ! git fetch origin \
       "+refs/heads/${GITHUB_BASE_REF}:refs/remotes/origin/${GITHUB_BASE_REF}" >/dev/null 2>&1; then
    echo "ERROR: 取不到 PR base 分支 ${GITHUB_BASE_REF}（git fetch 失敗）" >&2
    exit 3
  fi
  if ! have "origin/${GITHUB_BASE_REF}"; then
    echo "ERROR: fetch 後 origin/${GITHUB_BASE_REF} 仍不存在" >&2
    exit 3
  fi
  echo "origin/${GITHUB_BASE_REF}"
  exit 0
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
