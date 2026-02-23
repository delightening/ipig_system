#!/bin/bash
# ============================================
# MaxMind GeoLite2-City 自動更新腳本
# 用途：定期下載最新的 GeoIP 資料庫檔案
# 需求：MAXMIND_LICENSE_KEY 環境變數
# ============================================

set -euo pipefail

# 設定
GEOIP_DIR="${GEOIP_DIR:-./geoip}"
EDITION="GeoLite2-City"
LOG_PREFIX="[$(date '+%Y-%m-%d %H:%M:%S')]"

# 檢查 License Key
if [ -z "${MAXMIND_LICENSE_KEY:-}" ]; then
    echo "${LOG_PREFIX} ❌ 請設定 MAXMIND_LICENSE_KEY 環境變數"
    echo "  取得方式：https://www.maxmind.com/en/accounts/current/license-key"
    exit 1
fi

# 確保目錄存在
mkdir -p "${GEOIP_DIR}"

# 下載 URL
DOWNLOAD_URL="https://download.maxmind.com/app/geoip_download?edition_id=${EDITION}&license_key=${MAXMIND_LICENSE_KEY}&suffix=tar.gz"
CHECKSUM_URL="https://download.maxmind.com/app/geoip_download?edition_id=${EDITION}&license_key=${MAXMIND_LICENSE_KEY}&suffix=tar.gz.sha256"

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "${TEMP_DIR}"' EXIT

echo "${LOG_PREFIX} 📥 下載 ${EDITION} 資料庫..."

# 下載檔案與校驗碼
if ! curl -sS -o "${TEMP_DIR}/${EDITION}.tar.gz" "${DOWNLOAD_URL}"; then
    echo "${LOG_PREFIX} ❌ 下載失敗"
    exit 1
fi

if ! curl -sS -o "${TEMP_DIR}/${EDITION}.tar.gz.sha256" "${CHECKSUM_URL}"; then
    echo "${LOG_PREFIX} ⚠️ 無法下載校驗碼，跳過驗證"
else
    # 驗證 SHA256
    echo "${LOG_PREFIX} 🔍 驗證檔案完整性..."
    EXPECTED=$(awk '{print $1}' "${TEMP_DIR}/${EDITION}.tar.gz.sha256")
    ACTUAL=$(sha256sum "${TEMP_DIR}/${EDITION}.tar.gz" | awk '{print $1}')
    if [ "${EXPECTED}" != "${ACTUAL}" ]; then
        echo "${LOG_PREFIX} ❌ SHA256 校驗失敗！"
        echo "  預期: ${EXPECTED}"
        echo "  實際: ${ACTUAL}"
        exit 1
    fi
    echo "${LOG_PREFIX} ✅ SHA256 校驗通過"
fi

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
