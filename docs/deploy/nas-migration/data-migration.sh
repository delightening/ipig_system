#!/bin/bash
# ============================================================
# iPig System — 資料搬移腳本（本機 → NAS）
# ============================================================
# 在本機（來源端）執行此腳本，將資料打包並傳到 NAS
#
# 前提：
#   - NAS 已開啟 SSH
#   - 已設定 SSH key 或知道密碼
#   - 本機 Docker 服務已停止（確保資料一致性）
#
# 使用方式：
#   chmod +x data-migration.sh
#   ./data-migration.sh <NAS_USER> <NAS_IP>
#
# 範例：
#   ./data-migration.sh admin 192.168.1.100
# ============================================================

set -euo pipefail

NAS_USER="${1:?用法: $0 <NAS_USER> <NAS_IP>}"
NAS_IP="${2:?用法: $0 <NAS_USER> <NAS_IP>}"
NAS_BASE="/volume1/docker/ipig"
BACKUP_DIR="./nas-migration-backup"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "============================================"
echo "iPig NAS 資料搬移"
echo "目標: ${NAS_USER}@${NAS_IP}:${NAS_BASE}"
echo "時間: ${TIMESTAMP}"
echo "============================================"

# --------------------------------------------------
# Step 0: 確認 Docker 服務已停止
# --------------------------------------------------
echo ""
echo "[Step 0] 確認 Docker 服務狀態..."
if docker compose ps --status running 2>/dev/null | grep -q "ipig"; then
    echo "⚠️  偵測到 iPig 容器仍在運行！"
    echo "   請先執行: docker compose down"
    echo "   確保資料一致性後再繼續。"
    read -p "   是否繼續？(y/N) " confirm
    [ "$confirm" = "y" ] || exit 1
fi

# --------------------------------------------------
# Step 1: NAS 端建立目錄結構
# --------------------------------------------------
echo ""
echo "[Step 1] 在 NAS 建立目錄結構..."
ssh "${NAS_USER}@${NAS_IP}" "
    sudo mkdir -p ${NAS_BASE}/{config,data,secrets}
    sudo mkdir -p ${NAS_BASE}/config/{geoip,prometheus,alertmanager,grafana/provisioning,grafana/dashboards}
    sudo mkdir -p ${NAS_BASE}/data/{postgres,uploads,backups}
    sudo chown -R ${NAS_USER}:${NAS_USER} ${NAS_BASE}
    echo '目錄結構建立完成'
    find ${NAS_BASE} -type d | head -20
"

# --------------------------------------------------
# Step 2: 匯出 PostgreSQL 資料（最可靠的方式）
# --------------------------------------------------
echo ""
echo "[Step 2] 匯出 PostgreSQL 資料庫..."
mkdir -p "${BACKUP_DIR}"

# 方法 A（推薦）：用 pg_dumpall 完整匯出
echo "  啟動臨時 DB 容器匯出..."
docker compose up -d db
sleep 5  # 等待 DB 就緒

docker compose exec -T db pg_dumpall \
    -U "${POSTGRES_USER:-postgres}" \
    --clean \
    > "${BACKUP_DIR}/pg_dumpall_${TIMESTAMP}.sql"

echo "  SQL dump 大小: $(du -h "${BACKUP_DIR}/pg_dumpall_${TIMESTAMP}.sql" | cut -f1)"

docker compose down

# --------------------------------------------------
# Step 3: 打包 uploads 目錄
# --------------------------------------------------
echo ""
echo "[Step 3] 打包 uploads 目錄..."
if [ -d "./uploads" ]; then
    tar czf "${BACKUP_DIR}/uploads_${TIMESTAMP}.tar.gz" -C . uploads/
    echo "  uploads 打包大小: $(du -h "${BACKUP_DIR}/uploads_${TIMESTAMP}.tar.gz" | cut -f1)"
else
    echo "  ⚠️  ./uploads 不存在，跳過"
fi

# --------------------------------------------------
# Step 4: 打包設定檔
# --------------------------------------------------
echo ""
echo "[Step 4] 打包設定檔..."

# GeoIP
if [ -d "./geoip" ]; then
    tar czf "${BACKUP_DIR}/geoip_${TIMESTAMP}.tar.gz" -C . geoip/
fi

# Prometheus config
if [ -d "./monitoring/prometheus" ]; then
    tar czf "${BACKUP_DIR}/prometheus_config_${TIMESTAMP}.tar.gz" -C ./monitoring prometheus/
fi

# Alertmanager config
if [ -d "./monitoring/alertmanager" ]; then
    tar czf "${BACKUP_DIR}/alertmanager_config_${TIMESTAMP}.tar.gz" -C ./monitoring alertmanager/
fi

# Grafana provisioning + dashboard
if [ -d "./deploy/grafana" ]; then
    tar czf "${BACKUP_DIR}/grafana_config_${TIMESTAMP}.tar.gz" -C ./deploy grafana/
fi
if [ -f "./deploy/grafana_dashboard.json" ]; then
    cp ./deploy/grafana_dashboard.json "${BACKUP_DIR}/grafana_dashboard.json"
fi

# Watchtower entrypoint
if [ -f "./scripts/watchtower-entrypoint.sh" ]; then
    cp ./scripts/watchtower-entrypoint.sh "${BACKUP_DIR}/watchtower-entrypoint.sh"
fi

echo "  設定檔打包完成"

# --------------------------------------------------
# Step 5: 傳輸到 NAS
# --------------------------------------------------
echo ""
echo "[Step 5] 傳輸資料到 NAS..."

# SQL dump
echo "  傳輸 DB dump..."
scp "${BACKUP_DIR}/pg_dumpall_${TIMESTAMP}.sql" \
    "${NAS_USER}@${NAS_IP}:${NAS_BASE}/data/"

# Uploads
if [ -f "${BACKUP_DIR}/uploads_${TIMESTAMP}.tar.gz" ]; then
    echo "  傳輸 uploads..."
    scp "${BACKUP_DIR}/uploads_${TIMESTAMP}.tar.gz" \
        "${NAS_USER}@${NAS_IP}:${NAS_BASE}/data/"
fi

# Config files
echo "  傳輸設定檔..."
[ -f "${BACKUP_DIR}/geoip_${TIMESTAMP}.tar.gz" ] && \
    scp "${BACKUP_DIR}/geoip_${TIMESTAMP}.tar.gz" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/"
[ -f "${BACKUP_DIR}/prometheus_config_${TIMESTAMP}.tar.gz" ] && \
    scp "${BACKUP_DIR}/prometheus_config_${TIMESTAMP}.tar.gz" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/"
[ -f "${BACKUP_DIR}/alertmanager_config_${TIMESTAMP}.tar.gz" ] && \
    scp "${BACKUP_DIR}/alertmanager_config_${TIMESTAMP}.tar.gz" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/"
[ -f "${BACKUP_DIR}/grafana_config_${TIMESTAMP}.tar.gz" ] && \
    scp "${BACKUP_DIR}/grafana_config_${TIMESTAMP}.tar.gz" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/"
[ -f "${BACKUP_DIR}/grafana_dashboard.json" ] && \
    scp "${BACKUP_DIR}/grafana_dashboard.json" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/grafana/dashboards/ipig-overview.json"
[ -f "${BACKUP_DIR}/watchtower-entrypoint.sh" ] && \
    scp "${BACKUP_DIR}/watchtower-entrypoint.sh" "${NAS_USER}@${NAS_IP}:${NAS_BASE}/config/"

echo "  傳輸完成！"

# --------------------------------------------------
# Step 6: NAS 端解壓
# --------------------------------------------------
echo ""
echo "[Step 6] NAS 端解壓..."
ssh "${NAS_USER}@${NAS_IP}" "
    cd ${NAS_BASE}

    # 解壓 uploads
    if [ -f data/uploads_${TIMESTAMP}.tar.gz ]; then
        tar xzf data/uploads_${TIMESTAMP}.tar.gz -C data/
        mv data/uploads/* data/uploads/ 2>/dev/null || true
        rm -f data/uploads_${TIMESTAMP}.tar.gz
        echo '  uploads 解壓完成'
    fi

    # 解壓 config
    [ -f config/geoip_${TIMESTAMP}.tar.gz ] && \
        tar xzf config/geoip_${TIMESTAMP}.tar.gz -C config/ && \
        rm -f config/geoip_${TIMESTAMP}.tar.gz
    [ -f config/prometheus_config_${TIMESTAMP}.tar.gz ] && \
        tar xzf config/prometheus_config_${TIMESTAMP}.tar.gz -C config/ && \
        rm -f config/prometheus_config_${TIMESTAMP}.tar.gz
    [ -f config/alertmanager_config_${TIMESTAMP}.tar.gz ] && \
        tar xzf config/alertmanager_config_${TIMESTAMP}.tar.gz -C config/ && \
        rm -f config/alertmanager_config_${TIMESTAMP}.tar.gz
    [ -f config/grafana_config_${TIMESTAMP}.tar.gz ] && \
        tar xzf config/grafana_config_${TIMESTAMP}.tar.gz -C config/ && \
        rm -f config/grafana_config_${TIMESTAMP}.tar.gz

    echo '  所有檔案解壓完成'
    echo ''
    echo '  目錄結構：'
    find ${NAS_BASE} -type f | head -30
"

echo ""
echo "============================================"
echo "✅ 資料傳輸完成！"
echo ""
echo "接下來請到 NAS 上執行："
echo "  1. 複製 secrets/ 到 ${NAS_BASE}/secrets/"
echo "  2. 複製 .env 到 ${NAS_BASE}/.env"
echo "  3. 複製 docker-compose.nas.yml 到 ${NAS_BASE}/docker-compose.yml"
echo "  4. 匯入 DB: 見 migration-sop.md Step 4"
echo "============================================"
