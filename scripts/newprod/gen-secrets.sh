#!/usr/bin/env bash
# 為「新 prod」stack 產生一套獨立的 secrets。
#
# 背景：新 prod 與現行 prod 並行運作，兩者的金鑰完全隔離（使用者 2026-08-20 裁定）。
# 實測影響：現行 prod 只有 1 筆 totp_secret_encrypted 且 totp_enabled=false、
# signature_bridge_sessions 為 0 筆，故換金鑰無實質資料損失。
# 稽核 HMAC 鏈亦重新起算（使用者裁定「舊資料只要存一個結果即可，不用可驗」）。
#
# 用法：
#   ./scripts/newprod/gen-secrets.sh                    # 初次佈建（目標目錄須為空或不存在）
#   ./scripts/newprod/gen-secrets.sh --allow-existing   # 在既有部署上重跑
#
# 冪等：已存在的檔案不覆蓋（要重產請先自行刪除該檔）。
#
# ⚠️ R103-5：目標目錄裡**只要有任何檔案**，預設就直接拒絕執行（fail-closed）。
# 理由是這支腳本的落點在已部署的機器上就是現役 prod 的 secrets 目錄，
# 而它對既有檔案也會下 chmod。要在既有部署上重跑須明確帶 `--allow-existing`，
# 詳見下方守衛處的說明。
#
# ⚠️ 刻意用**位置參數而非環境變數**：環境變數 `export` 一次之後，同一個 shell
# 裡後續每一次執行都是無守衛的，而且從指令列看不出來。旗標每次都要重打。

set -euo pipefail

# R103-2：本腳本寫出的是 prod 的資料庫密碼、各服務 token 與 JWT 私鑰。
# 預設 umask（多為 022）會讓這些檔案 group/world-readable，等於把 prod 金鑰
# 攤給同機器上的任何使用者。改成 0600（目錄 0700）。
#
# ⚠️ 刻意**只**設 umask，不對既有的 `secrets/` 目錄或既有檔案下 chmod：
# 容器可能以非 root 身分掛載讀取，收緊既有權限有機會讓服務讀不到 secret 而起不來。
# 要調整既有檔案請先確認各服務的執行身分（見 docker-compose 的 user: 設定）。
umask 077

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SECRETS_DIR="$REPO_ROOT/secrets"

# R103-5：這支腳本的定位是「初次佈建」，但它的落點 `$REPO_ROOT/secrets` 在**已部署的
# 機器上就是現役 prod 正在用的那個目錄**（vet 實查 `ipig-api` 容器掛載確認）。
# 檔名與所在目錄都叫 `newprod`，而 newprod stack 已確認不存在（R103-1）——
# 名字指向一個不存在的東西，實際卻對著正式機。
#
# ⚠️ **它自己分不出「新機器」與「已在服務的機器」**，而誤跑的代價不只是產檔：
# 本腳本對既有檔案也會下 chmod（目錄 0711、六個監控檔 0644），所以就算一個檔都
# 沒新產生，也已經動到現役金鑰的權限。故此處 fail-closed。
#
# 判準為什麼是「已佈建」而不是「正在服務」：**檔案系統上沒有任何訊號能區分兩者**。
# 唯一能區分的訊號是「這些檔案是否被執行中的容器 bind-mount」，但**不拿它當閘門**，
# 兩個理由：
#   (1) 它在最危險的情況下 fail-open——docker CLI 不在、沒權限、daemon 沒起來時
#       一律放行，而「有人在正式機上手動跑這支腳本」正好常常是這個情境。
#   (2) 要保護的那台是 Windows，容器 mount source 形如 `C:\...\secrets\x.txt`，
#       而腳本在 Git Bash 算出的路徑是 `/c/...` 或 `C:/...`。正規化（大小寫、
#       `\` vs `/`、磁碟機表示法）只要寫錯一點，就是**沉默地永遠不命中**——
#       一個看起來有守衛、實際永遠放行的空殼，比沒有守衛更糟。
# 因此閘門取保守的上位集合——目標目錄裡只要有任何一個核心 secret，就當作已佈建而拒絕；
# docker 那個訊號寫進錯誤訊息當**診斷指引**，由人去確認。
# 寧可擋下一次合法的重跑（帶旗標即可放行），也不要在正式機上默默改權限。
#
# ⚠️ **這條改變了原本的冪等用法**：在已佈建的目錄上重跑（例如補回被刪的公鑰、
# 或讓權限套用到既有檔案）現在必須明確帶 `--allow-existing`。
# 這是刻意的——把「我知道這是既有部署」變成一個要打字的動作，而不是預設值。
#
# ⚠️ **判準是「目錄裡有任何檔案」，不是一份 marker 白名單。**
# 初版寫成五個核心 secret 的白名單，那有一個具體缺口：`secrets/` 若只含
# 監控堆疊那六個檔（`metrics_token.txt` 等，見檔案末尾 `BIND_MOUNTED_SECRETS`）、
# 或任何不含那五個的子集，守衛會判定為空而放行，接著照樣對現役目錄下
# `chmod 0711`、對那些**正被容器 bind-mount 的**檔下 `chmod 0644`
# ——正是本守衛要擋的事，只是繞過了守衛。而殘缺狀態（R103-3 處理的那個主題）
# 剛好就長這樣，等於同一支腳本裡兩處對「什麼算已佈建」的定義不一致。
# 改用「有任何檔案就擋」除了補掉那個缺口，還消掉了**清單漂移**這個風險來源：
# 白名單要跟著日後新增的 secret 一起維護，忘記維護就是靜默放行。
# `secrets/` 在 `.gitignore` 是整個目錄排除、零版控檔案（實查），
# 所以不會有 `.gitkeep` 這類無辜檔案被誤判。
ALLOW_EXISTING=0
while [ $# -gt 0 ]; do
  case "$1" in
    --allow-existing) ALLOW_EXISTING=1; shift ;;
    *)
      echo "ERROR: 未知參數：$1" >&2
      echo "用法：$0 [--allow-existing]" >&2
      exit 1
      ;;
  esac
done

if [ "$ALLOW_EXISTING" -ne 1 ] && [ -d "$SECRETS_DIR" ]; then
  # 只看一般檔案；目錄本身為空則視為未佈建
  existing_count=$(find "$SECRETS_DIR" -maxdepth 1 -type f | wc -l)
  if [ "$existing_count" -gt 0 ]; then
    echo "ERROR: 目標目錄已經有東西了，本腳本拒絕在上面執行。" >&2
    echo "       目標：$SECRETS_DIR（既有檔案 $existing_count 個）" >&2
    # 不用 `find -printf`：那是 GNU 專屬，busybox 沒有，會靜默少印這一段
    find "$SECRETS_DIR" -maxdepth 1 -type f | head -8 | while read -r p; do
      echo "         - $(basename "$p")" >&2
    done
    if [ "$existing_count" -gt 8 ]; then
      echo "         …（其餘 $((existing_count - 8)) 個略）" >&2
    fi
    echo "" >&2
    echo "       本腳本的定位是「初次佈建」。在已部署的機器上，這個路徑就是" >&2
    echo "       現役 prod 正在使用的 secrets 目錄；即使一個檔都不新產生，" >&2
    echo "       它仍會對既有檔案與目錄下 chmod，動到現役金鑰的權限。" >&2
    echo "" >&2
    echo "       要確認這批檔案是不是正在被服務使用（本腳本刻意不自己判斷，理由見原始碼註解）：" >&2
    echo "           docker ps --filter status=running --format '{{.Names}}'" >&2
    echo "           docker inspect <容器> --format '{{range .Mounts}}{{.Source}}{{\"\\n\"}}{{end}}'" >&2
    echo "       source 若落在上面那個目標路徑底下，就是現役 prod 在用。" >&2
    echo "" >&2
    echo "       依你的意圖選一個：" >&2
    echo "       - 這台是新機器，上面那些是殘留 → 先把 $SECRETS_DIR 移到別處備份，再重跑" >&2
    echo "       - 我就是要在既有部署上重跑（補回缺檔／套用權限）→" >&2
    echo "           $0 --allow-existing" >&2
    echo "         ⚠️ 那會對既有檔案套用權限變更，動 prod 前請先確認影響範圍。" >&2
    exit 1
  fi
fi

mkdir -p "$SECRETS_DIR"

# ⚠️ 目錄權限要單獨設，不能放給 umask 決定。
# `umask 077` 會讓上面這行建出 0700 的目錄，而 0700 對 other 沒有 execute，
# **裡面的檔案再怎麼放寬都讀不到**——監控堆疊那幾個 bind mount（見檔案末尾）
# 就是這樣被擋住的，實測 uid 472 連 0644 的檔案都開不起來。
# 這也正是本檔案原始待辦（R103-2）警告過的「不要讓 secrets/ 變成 0700」，
# 只是它經由 umask 間接發生，比直接下 chmod 700 更難察覺。
#
# 用 0711 而不是 0755：容器以完整路徑讀取，不需要列目錄；
# 而檔名清單本身會透露「這台機器有哪些服務的憑證」，沒必要對外開放。
chmod 0711 "$SECRETS_DIR"

created=0
skipped=0

# 產生隨機字串（base64，去掉換行）
rand_b64() { openssl rand -base64 "$1" | tr -d '\n'; }
# 產生 URL-safe 隨機字串
rand_hex() { openssl rand -hex "$1"; }

write_if_absent() { # $1=檔名 $2=內容
  local f="$SECRETS_DIR/$1"
  if [ -e "$f" ]; then
    echo "  skip   $1（已存在）"
    skipped=$((skipped + 1))
  else
    printf '%s' "$2" > "$f"
    echo "  create $1"
    created=$((created + 1))
  fi
}

echo "=== 產生新 prod 的 secrets 到 $SECRETS_DIR ==="
echo ""
echo "--- 隨機金鑰／密碼 ---"

# ENCRYPTION_KEY 必須是 base64 解碼後剛好 32 bytes（見 utils/crypto.rs:48）
write_if_absent encryption_key.txt        "$(rand_b64 32)"
# AUDIT_HMAC_KEY 長度須 >= 44（見 bin/verify_audit_chain.rs:40）
write_if_absent audit_hmac_key.txt        "$(rand_b64 48)"
write_if_absent csrf_secret.txt           "$(rand_b64 32)"
write_if_absent db_password.txt           "$(rand_hex 24)"
write_if_absent admin_initial_password.txt "$(rand_b64 18)"
write_if_absent pdf_service_token.txt     "$(rand_hex 32)"
write_if_absent metrics_token.txt         "$(rand_hex 32)"
write_if_absent alertmanager_webhook_token.txt "$(rand_hex 32)"
write_if_absent prometheus_password.txt   "$(rand_hex 24)"
write_if_absent grafana_admin_password.txt "$(rand_b64 18)"
write_if_absent grafana_pg_password.txt   "$(rand_hex 24)"

echo ""
echo "--- JWT EC 金鑰對（ES256）---"
# ⚠️ 必須是 PKCS8（標頭 `BEGIN PRIVATE KEY`），不能是 openssl ecparam 預設的
#    SEC1（標頭 `BEGIN EC PRIVATE KEY`）。後端只認 PKCS8，餵 SEC1 會在啟動時
#    直接 InvalidKeyFormat 並無限重啟（2026-08-20 實測踩過）。
#    `openssl genpkey` 預設就輸出 PKCS8，不需要額外 pkcs8 轉換步驟。
#
# 註：本檔刻意不寫出含破折號的完整 PEM 標頭字面值——gitleaks 的 private-key 規則
#     會把它判成洩漏（實測命中本檔 L62）。改用不含破折號的關鍵字比對，鑑別力相同：
#     `BEGIN PRIVATE KEY` 不會命中 SEC1 的 `BEGIN EC PRIVATE KEY`（中間隔著 `EC `），
#     配合 `head -1` 仍只看第一行。這樣本檔就不必列進 .gitleaks.toml 的豁免清單，
#     日後若有人在這支產生 secrets 的腳本裡寫死憑證，掃描器照樣抓得到。
# R103-3：這對金鑰是**兩個獨立的檔案**，所以要分四種狀態處理。
# 舊版只看私鑰在不在，在就 `skipped += 2` 跳過兩個檔——於是「私鑰在、公鑰被刪或
# 前次執行中斷」這個狀態下，重跑**不會**補出公鑰，secrets 集合殘缺而腳本回報成功。
# 冪等的意思是「跑完之後狀態一致」，不是「跑過就不再看」。
priv="$SECRETS_DIR/jwt_ec_private_key.pem"
pub="$SECRETS_DIR/jwt_ec_public_key.pem"

# 私鑰必須是 PKCS8（見上方註解）。抽成函式讓新產生與既有檔案走同一條驗證。
assert_pkcs8() {
  if ! head -1 "$priv" | grep -q 'BEGIN PRIVATE KEY'; then
    echo "ERROR: $priv 不是 PKCS8 格式，後端會拒絕啟動。" >&2
    echo "       （SEC1 的標頭是 BEGIN EC PRIVATE KEY，後端只認 PKCS8）" >&2
    exit 1
  fi
}

# ⚠️ 兩個檔都在時，只驗「存在」不夠——它們可能不是同一對。
# 後端把兩者**各自獨立**載入（`backend/src/config.rs`：`EncodingKey::from_ec_pem(私鑰)`
# 與 `DecodingKey::from_ec_pem(公鑰)`），中間沒有配對檢查，啟動路徑上也查無其他檢查
# （`startup/security_checks.rs` 的 H7 只看私鑰檔的 unix mode）。於是一對「兩個檔都在、
# 格式都對、但彼此不配對」的金鑰會讓服務**正常啟動**，卻是簽出來的 token 一律驗不過
# ——全部使用者登入即失效，而本腳本原本會回報 skip 說一切正常。
# 成因不必假設得很奇特：手動換過其中一個、從不同世代的備份還原、或上一次修復時
# 弄錯方向（該補公鑰卻蓋了私鑰）都會造成。
assert_pair_matches() {
  local derived canonical
  derived="$(mktemp)"
  canonical="$(mktemp)"
  # 兩邊都轉成 SPKI 公鑰的標準輸出再比，避免換行或編碼差異造成假不符
  if ! openssl pkey -in "$priv" -pubout -out "$derived" 2>/dev/null; then
    rm -f "$derived" "$canonical"
    echo "ERROR: 無法從既有私鑰推導公鑰（$priv 可能已損壞）。" >&2
    exit 1
  fi
  if ! openssl pkey -pubin -in "$pub" -pubout -out "$canonical" 2>/dev/null; then
    rm -f "$derived" "$canonical"
    echo "ERROR: 既有公鑰不是合法的 EC 公鑰 PEM（$pub）。" >&2
    echo "       刪除它後重跑，本腳本會由既有私鑰重新推導。" >&2
    exit 1
  fi
  if ! cmp -s "$derived" "$canonical"; then
    rm -f "$derived" "$canonical"
    echo "ERROR: 既有的 JWT 私鑰與公鑰不是同一對。" >&2
    echo "       後端用私鑰簽、用公鑰驗，不配對＝簽出來的 token 一律驗不過。" >&2
    echo "       請先確認哪一個才是要保留的：" >&2
    echo "       - 私鑰是對的 → 刪掉 $pub 後重跑，本腳本會由私鑰推導出正確的公鑰" >&2
    echo "       - 私鑰是錯的 → 從備份還原正確的私鑰後再跑" >&2
    echo "       本腳本不自行猜測——猜錯的那一邊會讓既有已簽發的 token 全數失效。" >&2
    exit 1
  fi
  rm -f "$derived" "$canonical"
}

if [ -e "$priv" ] && [ -e "$pub" ]; then
  assert_pkcs8
  assert_pair_matches
  echo "  skip   jwt_ec_private_key.pem / jwt_ec_public_key.pem（皆已存在且互相配對）"
  skipped=$((skipped + 2))
elif [ -e "$priv" ] && [ ! -e "$pub" ]; then
  # 公鑰是私鑰的函數，可以無損重建——這種狀態要修好，不是報錯。
  assert_pkcs8
  openssl pkey -in "$priv" -pubout -out "$pub" 2>/dev/null
  echo "  skip   jwt_ec_private_key.pem（已存在）"
  echo "  create jwt_ec_public_key.pem（由既有私鑰推導）"
  skipped=$((skipped + 1))
  created=$((created + 1))
elif [ ! -e "$priv" ] && [ -e "$pub" ]; then
  # 反過來不可修復：公鑰推不回私鑰。這種狀態多半代表私鑰被誤刪，
  # 若逕自產生一對新的，既有已簽發的 token 會全部驗不過——要停下讓人決定。
  echo "ERROR: 只有公鑰存在、私鑰不見了（$pub）。" >&2
  echo "       公鑰無法推導回私鑰。請先確認私鑰是否還能從備份取回；" >&2
  echo "       若確定要重新起算（既有 JWT 全數失效、使用者被登出），" >&2
  echo "       請自行刪除該公鑰後重跑本腳本。" >&2
  exit 1
else
  openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:prime256v1 \
    -out "$priv" 2>/dev/null
  openssl pkey -in "$priv" -pubout -out "$pub" 2>/dev/null
  assert_pkcs8
  echo "  create jwt_ec_private_key.pem（PKCS8）"
  echo "  create jwt_ec_public_key.pem"
  created=$((created + 2))
fi

echo ""
echo "--- db_url.txt（依 db_password 組出）---"
if [ -e "$SECRETS_DIR/db_url.txt" ]; then
  echo "  skip   db_url.txt（已存在）"
  skipped=$((skipped + 1))
else
  DBPW="$(cat "$SECRETS_DIR/db_password.txt")"
  printf 'postgres://postgres:%s@db:5432/ipig_db' "$DBPW" > "$SECRETS_DIR/db_url.txt"
  echo "  create db_url.txt"
  created=$((created + 1))
fi

echo ""
echo "--- 對外服務憑證：留空佔位，需人工填入才會生效 ---"
# 這些連到真實外部服務（Gmail SMTP、Google API、R2/NAS）。留空 = 該功能不啟用，
# 不影響核心系統啟動。要用的話由使用者自行填入新 prod 專用的憑證。
write_if_absent smtp_password.txt         ""
write_if_absent alert_smtp_password.txt   ""
write_if_absent grafana_smtp_password.txt ""
write_if_absent google-service-account.json '{}'
write_if_absent rclone.conf               ""
write_if_absent backup_gpg_pubkey.asc     ""

echo ""
echo "--- 監控堆疊的 bind mount 例外（見下方說明）---"
# ⚠️ R103-2 的配套，不是可以順手刪掉的東西。
#
# `secrets/` 底下的檔案有**兩種**進容器的方式，權限語意完全不同：
#
#   (1) compose 的 `secrets:` 機制（api / web / outbox-worker 等多數服務）
#       —— daemon 以 root 讀取宿主檔案再放進容器的 /run/secrets/，
#          宿主端 0600 root-owned **不影響**容器內讀取。
#
#   (2) 直接 bind mount（`./secrets/x.txt:/run/secrets/x:ro`，監控堆疊在用）
#       —— **宿主端的權限直接生效**。而 grafana 官方映像的 `USER` 實測為 `472`
#          （`docker image inspect grafana/grafana:13.0.1 --format '{{.Config.User}}'`），
#          prometheus / alertmanager 同樣以非 root 執行。
#
# 於是 `umask 077` 對 (2) 這幾個檔案就會變成「全新機器上監控堆疊讀不到密碼」。
# 既有部署不受影響（檔案已存在，上面一律 skip），但新機器第一次部署就會中。
#
# 這裡明確把這幾個檔案放寬到 0644，並在輸出中講清楚代價；
# 敏感度較高的 JWT 私鑰 / DB 密碼 / audit HMAC key / encryption key 仍是 0600。
#
# 🔴 這是**取捨不是解法**：0644 等於同機器上任何使用者都讀得到這幾個值，
#    其中兩個是 SMTP 密碼。根治要把這幾個掛載改走 compose 的 `secrets:` 機制
#    （或指定 group 並讓容器以該 group 執行），屬 prod compose 變更，已另立待辦。
BIND_MOUNTED_SECRETS="
metrics_token.txt
prometheus_password.txt
alert_smtp_password.txt
grafana_pg_password.txt
grafana_smtp_password.txt
grafana_admin_password.txt
"
relaxed=0
for name in $BIND_MOUNTED_SECRETS; do
  f="$SECRETS_DIR/$name"
  if [ -e "$f" ]; then
    chmod 0644 "$f"
    relaxed=$((relaxed + 1))
  fi
done
echo "  已放寬 $relaxed 個檔案為 0644（監控堆疊以非 root 身分 bind mount 讀取）"
echo "  其餘檔案為 0600。"

echo ""
echo "=== 完成：新增 $created 個、略過 $skipped 個 ==="
echo ""
echo "⚠️ 下列檔案是空的佔位值，對應功能不會運作，要用請自行填入："
echo "     smtp_password.txt / alert_smtp_password.txt / grafana_smtp_password.txt  → 郵件通知"
echo "     google-service-account.json                                              → Google 行事曆同步"
echo "     rclone.conf / backup_gpg_pubkey.asc                                      → 異地備份與備份加密"
echo ""
echo "⚠️ 這些金鑰與現行 prod 完全不同，故："
echo "     - 現行 prod 的稽核 HMAC 鏈在新 prod 驗不過（已裁定接受）"
echo "     - 現行 prod 的 totp_secret_encrypted 在新 prod 解不開（實測僅 1 筆且未啟用）"
