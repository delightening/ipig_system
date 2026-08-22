#!/usr/bin/env bash
# 為「新 prod」stack 產生一套獨立的 secrets。
#
# 背景：新 prod 與現行 prod 並行運作，兩者的金鑰完全隔離（使用者 2026-08-20 裁定）。
# 實測影響：現行 prod 只有 1 筆 totp_secret_encrypted 且 totp_enabled=false、
# signature_bridge_sessions 為 0 筆，故換金鑰無實質資料損失。
# 稽核 HMAC 鏈亦重新起算（使用者裁定「舊資料只要存一個結果即可，不用可驗」）。
#
# 用法：
#   ./scripts/newprod/gen-secrets.sh
#
# 冪等：已存在的檔案不覆蓋（要重產請先自行刪除該檔）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SECRETS_DIR="$REPO_ROOT/secrets"

mkdir -p "$SECRETS_DIR"

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
if [ -e "$SECRETS_DIR/jwt_ec_private_key.pem" ]; then
  echo "  skip   jwt_ec_private_key.pem（已存在）"
  skipped=$((skipped + 2))
else
  openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:prime256v1 \
    -out "$SECRETS_DIR/jwt_ec_private_key.pem" 2>/dev/null
  openssl pkey -in "$SECRETS_DIR/jwt_ec_private_key.pem" -pubout \
    -out "$SECRETS_DIR/jwt_ec_public_key.pem" 2>/dev/null
  # 驗證真的是 PKCS8，不是靠假設
  if ! head -1 "$SECRETS_DIR/jwt_ec_private_key.pem" | grep -q 'BEGIN PRIVATE KEY'; then
    echo "ERROR: 產出的 JWT 私鑰不是 PKCS8 格式，後端會拒絕啟動。" >&2
    exit 1
  fi
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
