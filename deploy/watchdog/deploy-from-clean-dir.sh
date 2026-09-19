#!/usr/bin/env bash
#
# 從 repo 之外的乾淨目錄部署 ipig-watchdog。
#
# ## 為什麼需要這支
#
# 在 `deploy/watchdog/` 底下直接跑 `wrangler deploy` 會卡死（wrangler 4.128.0／
# 4.129.0，Windows 實測；細節見 README §「已知問題」）。已驗證有效的繞法是把
# `src/worker.js` 與 `wrangler.toml` 複製到 repo 外的乾淨目錄再從那裡部署——
# Cloudflare 認的是 `wrangler.toml` 裡的 `name`，不是本機執行目錄。
#
# ## 為什麼是腳本而不是 README 裡的指令
#
# 那段繞法原本寫在 README 的程式碼區塊裡，被審查連續訂正了十二輪。最後幾輪的
# findings 全部是**前一輪加的守衛長出來的交互作用**：
#
#   - `set -euo pipefail` 寫在 README 區塊裡 → 使用者貼進現有 shell 後它會**留著**，
#     後面本來可以獨立跑的驗收指令一失敗就把整個 shell 關掉
#   - `trap ... EXIT` 綁的是 **shell 的生命週期**而不是那段指令 → 貼進互動式 shell 時，
#     含真實值的 `wrangler.toml` 會留在暫存目錄直到視窗關閉
#   - 為了讓「目錄不存在」不報錯而加的錯誤抑制 → 連「刪不掉」也一起吞掉
#
# 這三件事在腳本裡全部消失：腳本有自己的行程，`set -e` 不污染呼叫者、`trap EXIT`
# 綁的就是這一次執行、錯誤處理可以寫完整。**而且腳本可以真的被執行與測試**——
# 那十二輪的病根就是「文件裡寫了沒人跑過的步驟」。
#
# ## 用法
#
#   bash deploy/watchdog/deploy-from-clean-dir.sh [--dry-run] [--reinstall] [--keep]
#
#     --dry-run    只印出會做什麼，不真的動檔案也不部署（也不需要已登入）
#     --reinstall  在目的地重新 `npm ci --include=dev`。**WSL 必須加**——wrangler
#                  依賴 `workerd`，那是平台原生二進位，Windows 裝的那份在 Linux
#                  側執行不了。腳本會偵測 WSL，沒加時直接停下來說明。
#     --keep       部署後保留目的地的 `wrangler.toml`（預設刪除，見下）
#
#   目的地預設 `${TMPDIR:-/tmp}/watchdog-deploy`，可用 WATCHDOG_DEPLOY_DIR 覆寫。
#
# ## 安全性
#
# `wrangler.toml` 含真實收件信箱與 KV namespace id（R104 最小揭露原則，故不進版控）。
# 本腳本複製它到暫存目錄，並在**離開時刪除**——成功、失敗、Ctrl-C 都會刪。
# **唯一的例外是 `--keep`**：那是刻意保留，會在結束時印出警告說明檔案還在哪裡。
# （原文寫「一律刪除」，與 --keep 互相矛盾；CodeRabbit 於 MR !23 指出，成立。）
# 刪不掉會明講，不靜音，而且**會讓腳本以非零碼結束**——否則自動化會看到「成功」，
# 而含真實值的檔案還躺在磁碟上。`src/worker.js` 不刪：它本來就在公開 repo 裡。

set -euo pipefail

# 🔒 目的地會放一份 `wrangler.toml`（含真實收件信箱與 KV namespace id）。
#    本腳本繼承呼叫者的 umask——寬鬆的 umask（例如 002 或 022）會讓那個目錄與檔案
#    同機其他使用者讀得到。在建立任何東西之前先收緊。
#    （CodeRabbit 於 MR !23 以 CWE-276 指出。同 repo 的 `scripts/newprod/gen-secrets.sh`
#     有同樣的處置，見 TODO R103-2。）
#    ⚠️ R103-2 的教訓是「umask 077 會讓目錄變 0700，而以非 root 執行的容器讀不到
#    bind mount 進去的檔」——**這裡不適用**：目的地只有本使用者自己跑的 wrangler
#    會讀，全程沒有容器參與。
#    ⚠️ **在 Windows／Git Bash 上這行實際上是 no-op**：本機實測 chmod 777 與 chmod 700
#    之後 stat 都報 755，權限位元改不動也讀不準。保留它是因為它在 Linux／WSL 側
#    （本腳本同樣支援的平台）真的會生效——不要因為在 Windows 看不到效果就以為它沒用，
#    也不要因為有這行就以為 Windows 那側的檔案權限被收緊了。
umask 077

# ── 用法 ─────────────────────────────────────────────────────────────────────
# 🔴 **不要用寫死的行號**（原本是 `sed -n '2,48p'`）。檔頭在第 43 行就結束，
#    48 會把 `set -euo pipefail` 和半截的 umask 註解一起印出來——而且每次改檔頭
#    都會再錯一次，這種錯不會有人發現，因為沒人把 --help 的輸出當成要驗的東西。
#    （CodeRabbit 於 MR !23 指出，成立。）
#    改成「從第二行印到第一個非註解行為止」，檔頭怎麼長它都跟得上。
usage() {
  local line first=1
  while IFS= read -r line; do
    if [ "$first" -eq 1 ]; then first=0; continue; fi   # 跳過 shebang
    case "$line" in
      '#'*) line="${line#\#}"; printf '%s\n' "${line# }" ;;
      *)    break ;;
    esac
  done < "${BASH_SOURCE[0]}"
}

# ── 參數 ─────────────────────────────────────────────────────────────────────
DRY_RUN=0
REINSTALL=0
KEEP_CONFIG=0
for arg in "$@"; do
  case "$arg" in
    --dry-run)   DRY_RUN=1 ;;
    --reinstall) REINSTALL=1 ;;
    --keep)      KEEP_CONFIG=1 ;;
    -h|--help)   usage; exit 0 ;;
    *)           echo "未知參數：$arg（--help 看用法）" >&2; exit 2 ;;
  esac
done

say()  { echo "  $*"; }
step() { echo; echo "▶ $*"; }
die()  { echo "🔴 $*" >&2; exit 1; }

# ── 定位來源（腳本自己的位置，不依賴呼叫者的 cwd）────────────────────────────
# 這是腳本相對於 README 的第一個好處：README 的指令必須交代「仍在 deploy/watchdog
# 底下」，而讀者從哪裡開始讀、有沒有照做，文件無從得知。腳本自己算得出來。
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
say "來源目錄：$SRC_DIR"

[ -f "$SRC_DIR/src/worker.js" ] || die "找不到 $SRC_DIR/src/worker.js"
[ -f "$SRC_DIR/wrangler.toml" ] || die "找不到 $SRC_DIR/wrangler.toml
       先從樣板複製一份：cp wrangler.toml.example wrangler.toml
       （它含真實收件信箱與 KV namespace id，刻意不進版控）"

# ── 偵測 WSL ─────────────────────────────────────────────────────────────────
IS_WSL=0
if [ -r /proc/version ] && grep -qiE 'microsoft|wsl' /proc/version 2>/dev/null; then
  IS_WSL=1
fi
if [ "$IS_WSL" -eq 1 ] && [ "$REINSTALL" -eq 0 ]; then
  die "偵測到 WSL，但沒有加 --reinstall。
       wrangler 依賴 workerd，那是**平台原生的二進位檔**：Windows 上裝的那份在
       Linux 側執行不了。請加 --reinstall，讓腳本在目的地用 Linux 的 Node 重裝。
       ⚠️ WSL 另外需要自己登入一次（憑證存在 Linux 側，不與 Windows 共用）：
          cd \"\$WATCHDOG_DEPLOY_DIR\" && ./node_modules/.bin/wrangler login"
fi

# ── 決定 wrangler 執行檔 ─────────────────────────────────────────────────────
# 兩種路徑都**不用 npx**：npx 只在 cwd 底下有裝好的 node_modules 時才取本地版本，
# 而目的地是乾淨目錄——在那裡打 npx 會轉去 registry 抓一個不受 lockfile 約束的版本。
WRANGLER=""
if [ "$REINSTALL" -eq 0 ]; then
  WRANGLER="$SRC_DIR/node_modules/.bin/wrangler"
  if [ ! -x "$WRANGLER" ] && [ "$DRY_RUN" -eq 0 ]; then
    die "找不到可執行的 $WRANGLER
       先在 $SRC_DIR 跑：npm ci --include=dev
       ⚠️ --include=dev 不能省——wrangler 在 devDependencies，NODE_ENV=production
       時 npm ci 會整段跳過它。"
  fi
fi

# ── 目的地 ───────────────────────────────────────────────────────────────────
DEST="${WATCHDOG_DEPLOY_DIR:-${TMPDIR:-/tmp}/watchdog-deploy}"

# 把路徑解析成絕對、無 `.`／`..` 的形式。
#
# ⚠️ 不能直接拿字串比前綴：`/tmp/x/../out` 的**字面**開頭是 `/tmp/x`，但它實際
#    在 `/tmp/x` 外面。（本腳本的測試抓到——第一版就是用字串前綴比的。）
#
# ⚠️ 只做「往上找到存在的目錄再 pwd -P」也不夠：**中間有不存在的段時，`..` 會留在
#    尾巴上**，例如 `/exists/nope/../../x` 會原樣回傳。字串比對因此不匹配，
#    而後面的 `mkdir -p` 會把它解開，真的建到 repo 裡去。
#    （CodeRabbit 於 MR !23 指出，成立。）
#    所以：先把存在的前綴用 `pwd -P` 解掉 symlink，**再對整條做逐段的字面正規化**。
# 全程不建立任何東西——dry-run 必須真的什麼都不動。
#
# 🔴 **切段不可以靠 `IFS='/'; set -- $p`。** 未加引號的 `$p` 除了分詞之外還會做
#    **pathname expansion**，路徑裡只要有 `*`／`?`／`[...]` 就會被 cwd 的內容改寫：
#        輸入 /tmp/base/*/dest ，cwd 裡有 AAA 與 BBB
#        → 實得 /tmp/base/AAA/BBB/dest    （**一條完全不同的路徑**，不是「少一段」）
#    而這條路徑後面會被送進 `rm -rf`，也會被 repo 排除檢查與標記檔檢查拿去比對。
#    （CodeRabbit 於 MR !23 指出，成立；上面那個輸出是實際跑出來的。）
#    改用純參數展開逐段切，全程在引號內——不分詞、不展開萬用字元，
#    連路徑含換行也不會被截斷（`read -a` 會）。
normalize_lexical() {
  local p="$1" seg rest out=""
  case "$p" in /*) ;; *) p="$PWD/$p" ;; esac
  rest="$p"
  while [ -n "$rest" ]; do
    seg="${rest%%/*}"
    if [ "$seg" = "$rest" ]; then rest=""; else rest="${rest#*/}"; fi
    case "$seg" in
      ''|.) ;;
      ..)   out="${out%/*}" ;;
      *)    out="$out/$seg" ;;
    esac
  done
  printf '%s\n' "${out:-/}"
}
resolve_path() {
  local p="$1" tail=""
  case "$p" in /*) ;; *) p="$PWD/$p" ;; esac
  while [ ! -d "$p" ] && [ "$p" != "/" ]; do
    tail="$(basename "$p")${tail:+/$tail}"
    p="$(dirname "$p")"
  done
  [ -d "$p" ] && p="$(cd "$p" && pwd -P)"
  normalize_lexical "$p${tail:+/$tail}"
}

# 🔴 排除範圍是**整個 repo**，不是只有 deploy/watchdog。
#    `<repo>/.watchdog-deploy` 一樣在 repo 裡，wrangler 照樣看得到 `.git`——
#    那正是這個繞法要避開的東西。（CodeRabbit 於 MR !23 指出，成立。）
#
# 🔴 **不要用 `git rev-parse --show-toplevel` 取 repo 根目錄。**
#    在 Windows／Git Bash 上，git.exe 回的是 **Windows 路徑**：
#        bash 這側：  /tmp/tmp.XXXX
#        git 回報的： C:/Users/…/Temp/tmp.XXXX
#    兩個命名空間永遠比不中，於是這道守衛**靜默失效**——而 Windows 正是本腳本
#    存在的平台。（本腳本的測試抓到；同一類路徑轉換問題今天已經在
#    scripts/ci/test-resolve-diff-base.sh 造成過實害，見 MR !21。）
#    改用 bash 自己往上找 `.git`，全程留在同一個命名空間。
find_repo_root() {
  local d="$1"
  while [ "$d" != "/" ] && [ -n "$d" ]; do
    [ -e "$d/.git" ] && { printf '%s\n' "$d"; return 0; }
    d="$(dirname "$d")"
  done
  return 1
}
REPO_ROOT="$(find_repo_root "$(resolve_path "$SRC_DIR")" || true)"
[ -n "$REPO_ROOT" ] || REPO_ROOT="$SRC_DIR"   # 不在 git 工作區時退回來源目錄
REPO_REAL="$(resolve_path "$REPO_ROOT")"

DEST_REAL="$(resolve_path "$DEST")"
case "$DEST_REAL" in
  "$REPO_REAL"|"$REPO_REAL"/*)
    die "目的地不能在 repo 裡。
       指定：$DEST
       解析：$DEST_REAL
       repo：$REPO_REAL
       整個 repo 都不行，不只 deploy/watchdog——wrangler 會看到 .git，
       而那正是這個繞法要避開的東西。" ;;
esac
say "目的地　：$DEST_REAL"

MODE="實際部署"
[ "$DRY_RUN" -eq 1 ] && MODE="dry-run（不動檔案、不部署）"
[ "$REINSTALL" -eq 1 ] && MODE="$MODE＋目的地重裝依賴"
[ "$IS_WSL" -eq 1 ] && MODE="$MODE＋WSL"
say "模式　　：$MODE"

# ── 離開時清掉敏感設定 ───────────────────────────────────────────────────────
# 🔒 成功、失敗、Ctrl-C 都刪。刪不掉要看得見——不抑制錯誤，改為明講。
#
# 🔴 **只有在本腳本真的取得目的地的擁有權之後，這個 trap 才准動手。**
#    第一版把 `trap cleanup EXIT` 裝在擁有權檢查之前，於是有兩條路會刪到不該刪的：
#      (1) `die`「這不是我建的目錄，拒絕刪除」→ 走 EXIT trap → 把那個目錄裡**別人的**
#          `wrangler.toml` 刪掉。剛剛才說不碰它。
#      (2) `--dry-run` 對一個有標記的既有目的地 → trap 照樣刪，而 dry-run 承諾
#          「什麼都不動」。
#    （CodeRabbit 於 MR !23 指出，成立。⚠️ 我的測試**斷言過**「dry-run 不動檔案」，
#     但只測了目的地**不存在**的情況——斷言涵蓋不到的地方，正是漏洞住的地方。）
#    `OWNS_DEST` 只在非 dry-run、且 rm／mkdir／寫標記檔三步都成功之後才設為 1。
OWNS_DEST=0
cleanup() {
  local rc=$?
  [ "$OWNS_DEST" -eq 1 ] || return $rc   # 沒有擁有權就什麼都不碰
  if [ "$KEEP_CONFIG" -eq 1 ]; then
    [ -f "$DEST/wrangler.toml" ] && {
      echo
      echo "⚠️ --keep：$DEST/wrangler.toml 保留著，內含真實收件信箱與 KV namespace id。"
    }
    return $rc
  fi
  if [ -f "$DEST/wrangler.toml" ]; then
    if ! rm -f "$DEST/wrangler.toml"; then
      echo "🔴 刪不掉 $DEST/wrangler.toml，請手動清除——裡面有真實值。" >&2
      # 🔴 **清不掉敏感檔必須反映在結束碼上。** 原本只 echo 就 `return $rc`，
      #    部署成功時 rc=0，於是自動化看到「成功」，而含真實收件信箱與 KV
      #    namespace id 的檔案還留在磁碟上。（CodeRabbit 於 MR !23 以 CWE-459
      #    指出，成立。）
      # 🔴 **而且這裡只能用 `exit`，不能用 `return`。** 實測（bash，本機）：
      #        trap 內 return 9 + 腳本 exit 0 → 實得 exit 0   ← 守衛無效
      #        trap 內 exit   9 + 腳本 exit 0 → 實得 exit 9
      #    照「回傳失敗狀態」的字面寫成 return，寫出來的是一道不會生效的守衛。
      # ⚠️ 但也不能無條件 exit 1：同一組實測顯示 trap 內的 exit 會**蓋掉**腳本
      #    原本的結束碼（exit 3 → 實得 9）。本來就失敗的話，原碼比 1 有資訊，
      #    保留它；本來成功才改寫成 1。
      if [ "$rc" -ne 0 ]; then exit "$rc"; else exit 1; fi
    fi
  fi
  return $rc
}
trap cleanup EXIT

# ── 準備目的地 ───────────────────────────────────────────────────────────────
step "準備乾淨目的地"

# 🔴🔴 這裡有一個 `rm -rf`，而目的地來自環境變數。第一版只檢查「不在 repo 裡」就動手
#      ——那代表 `WATCHDOG_DEPLOY_DIR=$HOME` 會**把家目錄刪掉**，而它完全通過檢查。
#      （CodeRabbit 於 MR !23 以 Critical 指出，成立。本腳本自己的 10 個測試一條都
#      沒抓到——測試只證明它測的那些事。）
#
# 規則：**只刪本腳本自己建立的目錄。**
#   - 目的地不存在      → 直接建，沒有東西要刪
#   - 存在且有標記檔    → 是上一次本腳本建的，可以整個刪掉重來
#   - 存在但沒有標記檔  → **拒絕**。那是別人的目錄，本腳本不碰
#
# 標記檔在建立時寫入，內容只是一句說明——它的作用是「這個目錄歸本腳本管」的憑證。
MARKER=".watchdog-deploy-dir"

if [ -e "$DEST_REAL" ] && [ ! -f "$DEST_REAL/$MARKER" ]; then
  die "目的地已存在，但不是本腳本建立的，拒絕刪除：
       $DEST_REAL
       本腳本只會清空自己建過的目錄（靠裡面的 $MARKER 判斷）。
       請改指一個不存在的路徑，或先自行確認並清空該目錄。
       ⚠️ 這道檢查是刻意的：這一步是 rm -rf，而目的地來自環境變數。"
fi

# 父目錄必須事先存在。理由寫在下面 mkdir 那段的 🔴 註解裡：本腳本不代建中間層。
DEST_PARENT="$(dirname "$DEST_REAL")"
[ -d "$DEST_PARENT" ] || die "目的地的父目錄不存在：
       $DEST_PARENT
       本腳本**刻意不代建中間層目錄**（代建出來的那幾層會多開一個 TOCTOU 空窗，
       見 mkdir 那段的註解）。請自己先建好父目錄再跑一次：
           mkdir -p \"$DEST_PARENT\"
       預設目的地 \${TMPDIR:-/tmp}/watchdog-deploy 的父層本來就存在，不受影響。"

# ⚠️ **沒有做的事：檢查祖先目錄「是不是他人可寫」。**
#    CodeRabbit 給了兩個選項，這裡只取第二個（逐層驗證），第一個沒採用——
#    因為它在本腳本存在的平台上讀到的是假數字。本機實測（MSYS_NT-10.0-22631）：
#        chmod 777 <dir> → stat 仍報 755
#        chmod 700 <dir> → stat 仍報 755
#        /tmp 的 sticky bit：無
#    也就是說權限位元在 Git Bash 底下既改不動也讀不準，而 /tmp 沒有 sticky bit
#    會讓「拒絕他人可寫且無 sticky 的祖先」這條規則**擋掉預設目的地本身**。
#    一道必須被刪掉才能做事的守衛，比沒有守衛更糟——它會連同旁邊真的有用的那幾道
#    一起被刪。在 Linux／WSL 上這個檢查是有意義的，但那需要平台分支，
#    而本腳本目前沒有任何平台分支的先例，不在這一輪加。
#    （同理，上一輪加的 `umask 077` 在 Windows 上實際上也是 no-op——它在
#     Linux／WSL 側才真的收緊權限。那條保留，因為它在會生效的平台上是對的。）

if [ "$DRY_RUN" -eq 1 ]; then
  if [ -e "$DEST_REAL" ]; then say "(dry-run) rm -rf \"$DEST_REAL\"（已確認帶有 $MARKER）"
  else say "(dry-run) 目的地不存在，會新建"; fi
  say "(dry-run) mkdir \"$DEST_REAL\" 與 \"$DEST_REAL/src\" 並寫入 $MARKER"
else
  # 先整個刪掉再建：mkdir 只保證目錄存在，不會清掉上一次留下的 worker.js。
  # 帶著舊檔部署會「成功」，而且每個訊號都說成功了——那是最難發現的失敗。
  rm -rf "$DEST_REAL"
  # 🔴 **這裡刻意不用 `mkdir -p`。** `rm -rf` 與 `mkdir` 之間有一個空窗，同機的
  #    其他人可以在那一瞬間把 $DEST_REAL 建成一個指向別處的 symlink；`mkdir -p`
  #    對「已經存在」是靜默成功，於是後面寫標記檔與複製含真實值的 wrangler.toml
  #    全部被導去對方指定的位置。不帶 -p 的 mkdir 碰到已存在（含 symlink）會
  #    直接失敗，`set -e` 當場中止——空窗關掉了。
  #    （CodeRabbit 於 MR !23 以 CWE-367 TOCTOU 指出，成立。）
  #
  # 🔴 **父層也不用 `mkdir -p` 代建。** 上一版寫的是
  #    `mkdir -p "$(dirname "$DEST_REAL")"`，CodeRabbit 接著指出它只是把同一個洞
  #    往上搬了一層：`-p` 代建出來的**中間層**一樣可以在下一個 mkdir 之前被換成
  #    symlink，後面的 `cp` 與 `cd` 就沿著被換掉的父路徑走。成立。
  #    處置是**把這件事整個拿掉**——父層必須事先存在（檢查在上面），不存在就停下來
  #    請使用者自己建。少一個「本腳本製造出來、卻還不歸本腳本管」的中間狀態，
  #    就少一個空窗；這是移除機制，不是再加一層防護。
  mkdir "$DEST_REAL"
  mkdir "$DEST_REAL/src"
  # 空窗關掉了，但**不等於路徑一定沒被動過**：祖先目錄可能在更早之前就被換掉，
  # 那樣 mkdir 一樣會成功，只是建在別的地方。所以建完立刻複驗實體路徑。
  # ⚠️ 這道檢查**不消滅競態**（shell 裡做不到 openat 那種原子性），
  #    它做的是把「靜默改道」變成「當場失敗」。不要把它讀成「這裡已經安全了」。
  [ ! -L "$DEST_REAL" ] || die "目的地在建立後變成 symlink，中止：$DEST_REAL"
  DEST_CHECK="$(cd "$DEST_REAL" && pwd -P)"
  [ "$DEST_CHECK" = "$DEST_REAL" ] || die "目的地的實體路徑與預期不符，中止：
       預期 $DEST_REAL
       實得 $DEST_CHECK
       ⚠️ 路徑上某一層可能被換成了 symlink。下一步要複製含真實值的
       wrangler.toml，不確定會寫到哪裡就不能繼續。"
  printf '%s\n' \
    "這個目錄由 ipig_system 的 deploy/watchdog/deploy-from-clean-dir.sh 建立與管理。" \
    "它會在每次部署前被整個刪除重建。不要把任何自己的東西放在這裡。" \
    > "$DEST_REAL/$MARKER"
  # 三步都成功了，從這裡開始這個目錄歸本腳本管——清理 trap 現在才准動手。
  OWNS_DEST=1
fi

# 之後一律用解析過的路徑，避免 $DEST 裡殘留的 .. 在後面被 shell 重新解開
DEST="$DEST_REAL"

step "複製部署物"
COPY_LIST=(src/worker.js wrangler.toml)
[ "$REINSTALL" -eq 1 ] && COPY_LIST+=(package.json package-lock.json)
for f in "${COPY_LIST[@]}"; do
  [ -f "$SRC_DIR/$f" ] || die "來源缺少 $f"
  say "$f"
  [ "$DRY_RUN" -eq 1 ] || cp "$SRC_DIR/$f" "$DEST/$f"
done

if [ "$REINSTALL" -eq 1 ]; then
  step "在目的地安裝依賴"
  if [ "$DRY_RUN" -eq 1 ]; then
    say "(dry-run) cd \"$DEST\" && npm ci --include=dev"
  else
    ( cd "$DEST" && npm ci --include=dev )
  fi
  WRANGLER="$DEST/node_modules/.bin/wrangler"
fi

step "部署"
say "wrangler：$WRANGLER"
if [ "$DRY_RUN" -eq 1 ]; then
  say "(dry-run) cd \"$DEST\" && \"\$WRANGLER\" deploy"
  echo
  echo "✅ dry-run 完成——上面每一步都沒有真的執行。"
  exit 0
fi

# subshell：只有這一段的 cwd 是目的地，呼叫者的 cwd 不受影響。
# 這是 README 版做不到的事——貼進現有 shell 的 `cd` 會把使用者留在暫存目錄裡。
( cd "$DEST" && "$WRANGLER" deploy )

echo
if [ "$KEEP_CONFIG" -eq 1 ]; then
  echo "✅ 部署完成（--keep：設定檔保留）。"
else
  # ⚠️ 這裡**還沒刪**——刪除發生在 EXIT trap，也就是這行印完之後。原本寫「已刪除」，
  #    於是 rm -f 失敗時的畫面是「✅ 已刪除」後面緊接著「🔴 刪不掉」，自相矛盾。
  #    （CodeRabbit 於 MR !23 指出，成立。）改成未來式，讓訊息順序與事實一致：
  #    真的刪不掉時，後面會印錯誤，而且結束碼會變成非零。
  echo "✅ 部署完成。目的地的 wrangler.toml 會在離開時刪除——下面若沒有紅字即為成功。"
fi
