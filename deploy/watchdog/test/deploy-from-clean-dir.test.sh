#!/usr/bin/env bash
#
# `deploy-from-clean-dir.sh` 的回歸測試。
#
# ## 為什麼有這支
#
# 這個繞法原本寫在 README 的程式碼區塊裡，被審查連續訂正十二輪。抽成腳本的**理由
# 之一就是它可以被執行與測試**——如果只是把同一段指令搬進 .sh 而沒有測試，那只是
# 換個地方放同樣沒被驗證的東西。
#
# 第一版寫完立刻被本檔的案例「含 .. 但實際在外面」抓到一個真 bug：目的地檢查用
# **字串前綴**比對，於是 `/tmp/x/../out`（實際在 x 外面）被誤判成「在 repo 裡」
# 而拒絕。現在解析成實體路徑再比，正反兩個案例把兩個方向都釘住。
#
# ## 執行
#
#   bash deploy/watchdog/test/deploy-from-clean-dir.test.sh
#
# 不連網、不需要 wrangler 或 Cloudflare 登入。**刻意不碰真的 `deploy/watchdog/`**
# ——那裡的 `wrangler.toml` 含真實收件信箱與 KV namespace id，測試自己在 `mktemp -d`
# 造一份最小的假來源。
#
# ⚠️ **涵蓋範圍的誠實邊界**（改動測試時這段要跟著更新，否則它會變成謊話）：
#
#   涵蓋：參數處理、路徑判斷、各道前置檢查、「dry-run 真的什麼都不動」，
#         以及用一支 wrangler stub 走完**非 dry-run** 那條路——複製部署物、
#         寫標記檔、離開時清掉 `wrangler.toml`、`--keep` 時保留它。
#         （原文寫「全程 --dry-run」，第五輪補了 stub 之後就不再成立；
#          CodeRabbit 於 MR !23 指出，成立。）
#
#   沒涵蓋：真正的 `wrangler deploy` 與 `npm ci`——那需要真的部署。
#           還有「清不掉敏感檔要回非零碼」（CWE-459）那條，在 Windows 上造不出
#           情境（chmod 是 no-op），會印 `⏭ SKIP` 並在總結加警告，**不假裝通過**。
#
# 所以綠燈的意思是「**該擋的有擋、該清的有清**」，不是「部署一定會成功」。

set -uo pipefail   # 刻意不用 -e：本檔要靠非零結束碼做斷言

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_SRC="$HERE/../deploy-from-clean-dir.sh"
[ -f "$SCRIPT_SRC" ] || { echo "🔴 找不到 $SCRIPT_SRC"; exit 1; }

pass=0; fail=0; skip=0
ck() { # ck <名稱> <期望 exit> <實得 exit>
  if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "  PASS  $1"
  else fail=$((fail+1)); echo "  FAIL  $1 —— 期望 exit=$2，實得 $3"; fi
}

echo "--- 語法 ---"
if bash -n "$SCRIPT_SRC"; then pass=$((pass+1)); echo "  PASS  bash -n"
else fail=$((fail+1)); echo "  FAIL  bash -n"; fi

# 造一份最小的假來源。**刻意不碰真的 deploy/watchdog**——那裡的 wrangler.toml
# 含真實收件信箱與 KV namespace id，測試不該讀它、更不該複製它。
SRC="$(mktemp -d)"
DEST_OUT="$SRC-dest"
trap 'rm -rf "$SRC" "$DEST_OUT" "$SRC-dest2"' EXIT
cp "$SCRIPT_SRC" "$SRC/deploy-from-clean-dir.sh"
mkdir -p "$SRC/src"
echo "export default {}" > "$SRC/src/worker.js"
printf 'name = "ipig-watchdog"\nmain = "src/worker.js"\n' > "$SRC/wrangler.toml"
S="$SRC/deploy-from-clean-dir.sh"

echo "--- 案例 ---"

WATCHDOG_DEPLOY_DIR="$DEST_OUT" bash "$S" --dry-run >/dev/null 2>&1
ck "dry-run 目的地在外面 → 成功" 0 $?

# 🔴 這一條釘住第一版的 bug：字面開頭是來源目錄，實際在外面，必須放行
WATCHDOG_DEPLOY_DIR="$SRC/../$(basename "$SRC")-dest2" bash "$S" --dry-run >/dev/null 2>&1
ck "含 .. 但實際在外面 → 放行" 0 $?

WATCHDOG_DEPLOY_DIR="$SRC/inside" bash "$S" --dry-run >/dev/null 2>&1
ck "目的地在來源底下 → 拒絕" 1 $?

# 反方向：字面看起來在外面，繞回來源底下，必須擋
WATCHDOG_DEPLOY_DIR="$SRC/x/../inside2" bash "$S" --dry-run >/dev/null 2>&1
ck "用 .. 繞回來源底下 → 拒絕" 1 $?

mv "$SRC/src/worker.js" "$SRC/src/worker.js.bak"
WATCHDOG_DEPLOY_DIR="$DEST_OUT" bash "$S" --dry-run >/dev/null 2>&1
ck "缺 src/worker.js → 停下" 1 $?
mv "$SRC/src/worker.js.bak" "$SRC/src/worker.js"

mv "$SRC/wrangler.toml" "$SRC/wrangler.toml.bak"
WATCHDOG_DEPLOY_DIR="$DEST_OUT" bash "$S" --dry-run >/dev/null 2>&1
ck "缺 wrangler.toml → 停下（並提示從 .example 複製）" 1 $?
mv "$SRC/wrangler.toml.bak" "$SRC/wrangler.toml"

# ── 🔴 以下三組是 CodeRabbit 在 MR !23 指出的，第一版全部會過 ──────────────
# 那一版有 10 個案例、全綠，而其中一條是 `rm -rf $HOME` 等級的洞。
# **測試只證明它測的那些事**——所以這幾條要先確認「沒有防護時真的會炸」，
# 才寫得出有鑑別力的斷言。

# (a) 中間有不存在的段時，`..` 會留在尾巴上，字串比對不匹配，
#     而 mkdir -p 會把它解開、真的建進 repo 裡
WATCHDOG_DEPLOY_DIR="$SRC/nope/../../$(basename "$SRC")/sneaky" bash "$S" --dry-run >/dev/null 2>&1
ck "中間有缺段的 .. 繞路 → 拒絕" 1 $?

# (b) 排除範圍要是整個 repo，不只 deploy/watchdog。
#     造一個假的 repo 結構：$SRC 當 deploy/watchdog，上兩層當 repo root。
REPO="$(mktemp -d)"
mkdir -p "$REPO/.git" "$REPO/deploy/watchdog/src"
cp "$SCRIPT_SRC" "$REPO/deploy/watchdog/deploy-from-clean-dir.sh"
echo "export default {}" > "$REPO/deploy/watchdog/src/worker.js"
printf 'name = "x"\n' > "$REPO/deploy/watchdog/wrangler.toml"
( cd "$REPO" && git init -q . 2>/dev/null ) || true
WATCHDOG_DEPLOY_DIR="$REPO/.watchdog-deploy" \
  bash "$REPO/deploy/watchdog/deploy-from-clean-dir.sh" --dry-run >/dev/null 2>&1
ck "目的地在 repo 根目錄底下（非 deploy/watchdog）→ 拒絕" 1 $?
rm -rf "$REPO"

# (c) 🔴 Critical：目的地是一個既有的、不屬於本腳本的目錄 → 絕不能 rm -rf
VICTIM="$(mktemp -d)"
echo "別人的重要檔案" > "$VICTIM/important.txt"
WATCHDOG_DEPLOY_DIR="$VICTIM" bash "$S" --dry-run >/dev/null 2>&1
ck "既有目錄且無標記檔 → 拒絕（不得 rm -rf）" 1 $?
if [ -f "$VICTIM/important.txt" ]; then pass=$((pass+1)); echo "  PASS  受害目錄的內容完好"
else fail=$((fail+1)); echo "  FAIL  🔴 受害目錄被刪了"; fi
# 有標記檔的話才允許
printf 'marker\n' > "$VICTIM/.watchdog-deploy-dir"
WATCHDOG_DEPLOY_DIR="$VICTIM" bash "$S" --dry-run >/dev/null 2>&1
ck "既有目錄但有標記檔 → 放行" 0 $?
rm -rf "$VICTIM"

# ── 🔴 第二輪：CodeRabbit 指出 `trap cleanup EXIT` 裝在擁有權檢查之前 ──────────
# 上面 (c) 驗的是「拒絕」與「important.txt 還在」，但 **cleanup() 刪的是
# `$DEST/wrangler.toml`**——受害目錄裡剛好沒有那個檔名，所以那條斷言對這個洞
# 完全沒有鑑別力。**測試只證明它測的那些事**，這次把檔名對準 cleanup 真正會刪的那個。

# (d) 被拒絕的目的地，裡面的 wrangler.toml 不能被 EXIT trap 順手刪掉
VICTIM2="$(mktemp -d)"
printf 'name = "someone-elses"\n' > "$VICTIM2/wrangler.toml"
WATCHDOG_DEPLOY_DIR="$VICTIM2" bash "$S" --dry-run >/dev/null 2>&1
ck "拒絕未標記目的地 → exit 1" 1 $?
if [ -f "$VICTIM2/wrangler.toml" ]; then pass=$((pass+1)); echo "  PASS  被拒絕的目的地，其 wrangler.toml 完好"
else fail=$((fail+1)); echo "  FAIL  🔴 拒絕之後仍把別人的 wrangler.toml 刪了"; fi
rm -rf "$VICTIM2"

# (e) dry-run 對「有標記的既有目的地」也不得刪任何東西——dry-run 承諾的是什麼都不動
MARKED="$(mktemp -d)"
printf 'marker\n' > "$MARKED/.watchdog-deploy-dir"
printf 'name = "previous-run"\n' > "$MARKED/wrangler.toml"
WATCHDOG_DEPLOY_DIR="$MARKED" bash "$S" --dry-run >/dev/null 2>&1
ck "dry-run 對已標記的既有目的地 → exit 0" 0 $?
if [ -f "$MARKED/wrangler.toml" ]; then pass=$((pass+1)); echo "  PASS  dry-run 沒有刪既有目的地的 wrangler.toml"
else fail=$((fail+1)); echo "  FAIL  🔴 dry-run 卻刪了檔案（它承諾什麼都不動）"; fi
rm -rf "$MARKED"

bash "$S" --nonsense >/dev/null 2>&1
ck "未知參數 → exit 2" 2 $?

bash "$S" --help >/dev/null 2>&1
ck "--help → exit 0" 0 $?

# ── 🔴 第三輪 ────────────────────────────────────────────────────────────────

# (f) --help 的內容。原本用寫死的行號 `sed -n '2,48p'`，檔頭第 43 行就結束，
#     於是把 `set -euo pipefail` 和半截註解一起印出來。**只驗 exit code 的斷言
#     對這個洞沒有作用**——它一直都是 0。
#     ⚠️ 這條斷言第一版寫成 `case "$HELP_OUT" in *"set -euo pipefail"*)`，**當場誤判**
#     ——檔頭自己就在散文裡提到這串（「`set -euo pipefail` 寫在 README 區塊裡 → …」）。
#     **比對字串會比到散文，不是程式碼。** 改成整行相等比對。
HELP_OUT="$(bash "$S" --help 2>&1)"
leaked=0
while IFS= read -r l; do
  [ "$l" = "set -euo pipefail" ] && leaked=1
  [ "$l" = "umask 077" ] && leaked=1
done <<< "$HELP_OUT"
if [ "$leaked" -eq 0 ]; then pass=$((pass+1)); echo "  PASS  --help 沒有印出檔頭以外的程式碼"
else fail=$((fail+1)); echo "  FAIL  🔴 --help 印出了檔頭以外的程式碼"; fi
case "$HELP_OUT" in
  *"--reinstall"*) pass=$((pass+1)); echo "  PASS  --help 有印到用法段落" ;;
  *) fail=$((fail+1)); echo "  FAIL  --help 沒印到用法段落（截太早）" ;;
esac

# (g) 路徑含萬用字元時不得被 cwd 的內容改寫。
#     舊實作的 `IFS='/'; set -- $p` 會做 pathname expansion：
#       /tmp/base/*/dest（cwd 裡有 AAA、BBB）→ /tmp/base/AAA/BBB/dest
#     那是一條**完全不同的路徑**，而它後面會被送進 rm -rf。
#     所以要把 cwd 造成「有東西可以被匹配」，再看解析結果有沒有被污染。
GLOBCWD="$(mktemp -d)"
mkdir -p "$GLOBCWD/AAA" "$GLOBCWD/BBB"
GLOB_OUT="$( cd "$GLOBCWD" && WATCHDOG_DEPLOY_DIR="$DEST_OUT/*/x" bash "$S" --dry-run 2>&1 )"
case "$GLOB_OUT" in
  *AAA*|*BBB*) fail=$((fail+1)); echo "  FAIL  🔴 路徑裡的萬用字元被 cwd 的內容展開了" ;;
  *) pass=$((pass+1)); echo "  PASS  萬用字元沒有被展開（cwd 內容不影響目的地）" ;;
esac
rm -rf "$GLOBCWD"

# ── 🔴 第四輪 ────────────────────────────────────────────────────────────────

# (h) 父目錄不存在 → 停下，不代建中間層。
#     上一版用 `mkdir -p "$(dirname …)"` 代建，CodeRabbit 指出代建出來的中間層
#     一樣可以在下一個 mkdir 之前被換成 symlink——洞只是往上搬了一層。
WATCHDOG_DEPLOY_DIR="$DEST_OUT/a/b/c" bash "$S" --dry-run >/dev/null 2>&1
ck "父目錄不存在 → 停下（不代建中間層）" 1 $?
if [ -e "$DEST_OUT" ]; then fail=$((fail+1)); echo "  FAIL  🔴 停下之前已經建了中間層"
else pass=$((pass+1)); echo "  PASS  停下時一層都沒建"; fi

# (i) 反例：父目錄存在就要放行，別把正常情況一起擋掉
mkdir -p "$SRC-parent"
WATCHDOG_DEPLOY_DIR="$SRC-parent/dest" bash "$S" --dry-run >/dev/null 2>&1
ck "父目錄存在 → 放行" 0 $?
rm -rf "$SRC-parent"

# ── 🔴 第五輪：非 dry-run 路徑 ───────────────────────────────────────────────
#
# 這一段的存在理由：前三輪的 findings **全部落在測試碰不到的那一半**——
# cleanup 的擁有權、mkdir 的序列、清不掉敏感檔的結束碼，沒有一條在 --dry-run
# 之下會被執行到。我連續三輪在 commit message 裡寫「這段沒有覆蓋」，
# 寫三次不如補上。
#
# 用一支 wrangler stub 冒充部署，全程不連網、不裝東西、不碰真的 Cloudflare。
mkdir -p "$SRC/node_modules/.bin"
cat > "$SRC/node_modules/.bin/wrangler" <<'STUB'
#!/usr/bin/env bash
echo "(stub wrangler) $*"
exit 0
STUB
chmod +x "$SRC/node_modules/.bin/wrangler"

# (j) 走完整條非 dry-run：含真實值的 wrangler.toml 必須在離開時被刪掉
REAL1="$SRC-real1"
WATCHDOG_DEPLOY_DIR="$REAL1" bash "$S" >/dev/null 2>&1
ck "非 dry-run 走完 → exit 0" 0 $?
if [ -f "$REAL1/src/worker.js" ]; then pass=$((pass+1)); echo "  PASS  worker.js 有部署過去"
else fail=$((fail+1)); echo "  FAIL  worker.js 沒被複製"; fi
if [ -f "$REAL1/wrangler.toml" ]; then fail=$((fail+1)); echo "  FAIL  🔴 含真實值的 wrangler.toml 留在磁碟上"
else pass=$((pass+1)); echo "  PASS  離開時刪掉了 wrangler.toml"; fi
if [ -f "$REAL1/.watchdog-deploy-dir" ]; then pass=$((pass+1)); echo "  PASS  標記檔有寫入"
else fail=$((fail+1)); echo "  FAIL  標記檔沒寫入（下次會被當成別人的目錄而拒絕）"; fi
rm -rf "$REAL1"

# (k) --keep 是刻意的例外：檔案要留著
REAL2="$SRC-real2"
WATCHDOG_DEPLOY_DIR="$REAL2" bash "$S" --keep >/dev/null 2>&1
ck "--keep 走完 → exit 0" 0 $?
if [ -f "$REAL2/wrangler.toml" ]; then pass=$((pass+1)); echo "  PASS  --keep 保留了 wrangler.toml"
else fail=$((fail+1)); echo "  FAIL  --keep 卻把檔案刪了"; fi
rm -rf "$REAL2"

# (l) 🔴 CWE-459：清不掉敏感檔時，結束碼必須是非零。
#     製造「刪不掉」的方法是把目的地設成唯讀——**而 chmod 在 Windows／Git Bash 上
#     是 no-op**（實測 chmod 777／700 之後 stat 都報 755）。所以這條在本平台上
#     造不出情境，會明確跳過而不是假裝通過。**自我跳過要看得見**，
#     否則它就是一條永遠綠的裝飾。
CHMODTEST="$(mktemp -d)"
mkdir -p "$CHMODTEST/d"; : > "$CHMODTEST/d/f"; chmod 500 "$CHMODTEST/d" 2>/dev/null
if rm -f "$CHMODTEST/d/f" 2>/dev/null; then
  skip=$((skip+1)); echo "  ⏭ SKIP  清不掉敏感檔→非零碼：本平台 chmod 無效，造不出唯讀目錄"
  chmod 700 "$CHMODTEST/d" 2>/dev/null
else
  chmod 700 "$CHMODTEST/d" 2>/dev/null
  REAL3="$SRC-real3"
  cat > "$SRC/node_modules/.bin/wrangler" <<'STUB2'
#!/usr/bin/env bash
# 部署「成功」，但順手把目的地變唯讀，讓後面的 rm -f wrangler.toml 失敗
echo "(stub wrangler) $*"
chmod 500 "$PWD"
exit 0
STUB2
  chmod +x "$SRC/node_modules/.bin/wrangler"
  WATCHDOG_DEPLOY_DIR="$REAL3" bash "$S" >/dev/null 2>&1
  rc=$?
  chmod 700 "$REAL3" 2>/dev/null
  if [ "$rc" -ne 0 ]; then pass=$((pass+1)); echo "  PASS  清不掉敏感檔 → 非零碼（實得 $rc）"
  else fail=$((fail+1)); echo "  FAIL  🔴 部署回報成功，而 wrangler.toml 還在磁碟上"; fi
  rm -rf "$REAL3"
fi
rm -rf "$CHMODTEST"

# dry-run 必須什麼都不動——否則「不部署」只是半個承諾
if [ -e "$DEST_OUT" ]; then fail=$((fail+1)); echo "  FAIL  dry-run 卻建立了 $DEST_OUT"
else pass=$((pass+1)); echo "  PASS  dry-run 沒有建立目的地"; fi

echo
echo "通過 $pass　失敗 $fail　跳過 $skip"
[ "$skip" -eq 0 ] || echo "⚠️ 有 $skip 條在本平台造不出情境而跳過——綠燈不涵蓋它們。"
[ "$fail" -eq 0 ] || exit 1
echo "全部通過"
