#!/bin/bash
# ============================================
# MaxMind GeoLite2-City 自動更新腳本
# 用途：定期下載最新的 GeoIP 資料庫檔案
# 需求：License Key —— 優先取 MAXMIND_LICENSE_KEY 環境變數，
#       未設時才回退讀 secrets/maxmind_license_key.txt
# ============================================

set -euo pipefail

# 設定
GEOIP_DIR="${GEOIP_DIR:-./geoip}"
EDITION="GeoLite2-City"
LOG_PREFIX="[$(date '+%Y-%m-%d %H:%M:%S')]"

# 檢查 License Key
# 未設環境變數時退回讀 secret 檔（對齊專案其他 secret 的 secrets/*.txt 慣例）。
# 走檔案的好處：呼叫者不必把 key 打在指令列（會留在 shell history），也不必寫進 .env
# ——.env 是以 env_file 灌進每個容器的，而這把 key 只有 host 端這支腳本會用到，
# API 執行期只讀 .mmdb 檔，完全不需要它。
# 注意：這只解決「呼叫端」的暴露，key 進到本腳本後仍需交給 curl，
# 見下方下載段落如何避免它出現在 curl 的 argv。
LICENSE_KEY_FILE="${MAXMIND_LICENSE_KEY_FILE:-${GEOIP_SECRET_DIR:-./secrets}/maxmind_license_key.txt}"
if [ -z "${MAXMIND_LICENSE_KEY:-}" ] && [ -f "${LICENSE_KEY_FILE}" ]; then
    # 先去掉 UTF-8 BOM（記事本存檔可能帶）再清空白：BOM 不屬於 [:space:]，
    # 留著會被當成 key 的一部分，下載直接回 401。
    MAXMIND_LICENSE_KEY=$(tr -d '[:space:]' < "${LICENSE_KEY_FILE}" | tr -d '\357\273\277')
fi

if [ -z "${MAXMIND_LICENSE_KEY:-}" ]; then
    echo "${LOG_PREFIX} ❌ 找不到 License Key"
    echo "  方式一（建議）：存成 ${LICENSE_KEY_FILE}（純文字一行）"
    echo "  方式二：設定 MAXMIND_LICENSE_KEY 環境變數"
    echo "  取得方式：https://www.maxmind.com/en/accounts/current/license-key"
    exit 1
fi

# 確保目錄存在
mkdir -p "${GEOIP_DIR}"

BASE_URL="https://download.maxmind.com/app/geoip_download?edition_id=${EDITION}&license_key=${MAXMIND_LICENSE_KEY}"

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "${TEMP_DIR}"' EXIT

echo "${LOG_PREFIX} 📥 下載 ${EDITION} 資料庫..."

# 以 stdin 餵 curl 設定，讓帶著 license key 的 URL 不出現在 argv。
#
# 為何不直接 `curl "${URL}"`：行程參數對同機其他使用者可見（Linux 的 ps /
# /proc/<pid>/cmdline），等於把 key 洩給整台機器。改用 `curl --config -`：
# 設定從 stdin 讀，URL 只存在於 pipe 內容裡，不進 argv。
#
# 旗標語義（別再寫錯）：
#   -L：跟隨 3xx 轉址。MaxMind 已把此端點改為 302，不跟隨會拿到 0 byte 空檔。
#   -f：讓 **HTTP >= 400** 變成非零 exit。它**不會**讓 302 失敗——3xx 由 -L 處理。
#       沒有 -f 時 4xx 也照樣 exit 0，錯誤會被帶到很後面才炸（2026-07-29 實際踩到：
#       缺 -L 拿到空檔、又因 exit 0 沒被 `if ! curl` 攔下，一路走到 SHA256 才發現）。
# 只有 URL 需要藏（key 在裡面），輸出路徑留在參數：Git Bash 會把 POSIX 路徑
# 自動轉成 Windows 路徑，但 config 檔內的 `output =` 不吃這層轉換，
# 寫進去會讓 curl.exe 拿到不存在的 /tmp/... 而回 error 23。
fetch() {
    local suffix="$1" out="$2"
    printf 'url = "%s&suffix=%s"\n' "${BASE_URL}" "${suffix}" \
        | curl -fsSL --config - -o "${out}"
}

if ! fetch "tar.gz" "${TEMP_DIR}/${EDITION}.tar.gz"; then
    echo "${LOG_PREFIX} ❌ 下載失敗（檢查 License Key 是否有效）"
    exit 1
fi

# 校驗碼必須拿到——fail closed。
# 舊版在這裡只 warn 就往下走，等於「校驗端點掛掉時，未經驗證的檔案照樣覆蓋現行資料庫」，
# 而現行資料庫是好的：這種情況下寧可中止也不要用未驗證的檔取代它。
if ! fetch "tar.gz.sha256" "${TEMP_DIR}/${EDITION}.tar.gz.sha256"; then
    echo "${LOG_PREFIX} ❌ 無法下載校驗碼，中止（不以未驗證的檔案覆蓋現行資料庫）"
    exit 1
fi

# 驗證 SHA256
echo "${LOG_PREFIX} 🔍 驗證檔案完整性..."
EXPECTED=$(awk '{print $1}' "${TEMP_DIR}/${EDITION}.tar.gz.sha256")
ACTUAL=$(sha256sum "${TEMP_DIR}/${EDITION}.tar.gz" | awk '{print $1}')
if [ -z "${EXPECTED}" ]; then
    echo "${LOG_PREFIX} ❌ 校驗碼檔案為空，中止"
    exit 1
fi
if [ "${EXPECTED}" != "${ACTUAL}" ]; then
    echo "${LOG_PREFIX} ❌ SHA256 校驗失敗！"
    echo "  預期: ${EXPECTED}"
    echo "  實際: ${ACTUAL}"
    exit 1
fi
echo "${LOG_PREFIX} ✅ SHA256 校驗通過"

# 解壓縮
echo "${LOG_PREFIX} 📦 解壓縮..."
tar -xzf "${TEMP_DIR}/${EDITION}.tar.gz" -C "${TEMP_DIR}"

# 找到 .mmdb 檔案
MMDB_FILE=$(find "${TEMP_DIR}" -name "*.mmdb" -type f | head -1)
if [ -z "${MMDB_FILE}" ]; then
    echo "${LOG_PREFIX} ❌ 解壓縮後找不到 .mmdb 檔案"
    exit 1
fi

# 備份舊版
if [ -f "${GEOIP_DIR}/${EDITION}.mmdb" ]; then
    BACKUP_NAME="${EDITION}.mmdb.bak.$(date +%Y%m%d)"
    cp "${GEOIP_DIR}/${EDITION}.mmdb" "${GEOIP_DIR}/${BACKUP_NAME}"
    echo "${LOG_PREFIX} 📋 舊版已備份為 ${BACKUP_NAME}"
fi

# 替換
cp "${MMDB_FILE}" "${GEOIP_DIR}/${EDITION}.mmdb"
SIZE=$(du -h "${GEOIP_DIR}/${EDITION}.mmdb" | cut -f1)
echo "${LOG_PREFIX} ✅ GeoIP 更新完成: ${EDITION}.mmdb (${SIZE})"
echo "${LOG_PREFIX} ℹ️  請重啟 API 服務以載入新資料庫: docker compose restart api"
