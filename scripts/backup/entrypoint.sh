#!/bin/sh
# ============================================
# 備份容器 Entrypoint
# 設定 cron 排程並啟動
# ============================================

set -e

SCHEDULE="${BACKUP_SCHEDULE:-0 2 * * *}"

echo "📋 iPIG DB Backup Container"
echo "   排程: ${SCHEDULE}"
echo "   保留: ${BACKUP_RETENTION_DAYS:-30} 天"
echo "   異地: ${BACKUP_RCLONE_REMOTES:-${RSYNC_TARGET:-（未設定 — 僅本機 docker volume）}}"
echo "   加密: ${BACKUP_REQUIRE_ENCRYPTION:-false}（recipient: ${BACKUP_GPG_RECIPIENT:-未設定}）"
echo ""

# R36-4: 容器啟動時自動 import GPG 公鑰（如果 secret 有內容）
# 使用者把 dedicated backup keypair 的公鑰放在 secrets/backup_gpg_pubkey.asc，
# compose 把它 mount 到 /run/secrets/backup_gpg_pubkey，這裡 import 進 keyring。
# -s 確保檔案非空（容許空 placeholder 跳過此步驟，由 BACKUP_REQUIRE_ENCRYPTION 把關）
# 2026-07-04 修：GPG / cron / 備份寫入一律以 root 執行。
# 先前以 backup(uid1000) 跑：寫不進 root-owned /backups，且金鑰匯 backup keyring
# 導致 root 找不到 recipient；且 crond 未執行 /etc/crontabs/backup（backup.log 全空）。
# → 全部改 root：root 可寫 /backups，金鑰匯 root keyring。
export GNUPGHOME=/root/.gnupg
mkdir -p "$GNUPGHOME"
chmod 700 "$GNUPGHOME"

if [ -s "/run/secrets/backup_gpg_pubkey" ]; then
    if gpg --import /run/secrets/backup_gpg_pubkey 2>&1 | tee /tmp/gpg_import.log | grep -q "imported\|unchanged"; then
        echo "🔑 GPG 公鑰已 import（從 /run/secrets/backup_gpg_pubkey）"
        # 設 ultimate trust：剛 import 的 key 預設 untrusted，gpg --encrypt 會拒絕
        # 容器只用這把 key 加密 backup，沒理由不完全信任 → ownertrust=6 (ultimate)
        FPR=$(gpg --list-keys --with-colons 2>/dev/null | awk -F: '/^fpr:/ {print $10; exit}')
        if [ -n "$FPR" ]; then
            echo "$FPR:6:" | gpg --import-ownertrust 2>/dev/null
            echo "🔐 GPG ownertrust 設為 ultimate ($FPR)"
        fi
    else
        echo "ERROR: 無法 import GPG 公鑰，內容："
        cat /tmp/gpg_import.log
        exit 1
    fi
fi

# E-3: 容器啟動時立即驗證 GPG 設定，而非等到凌晨 2 點備份才失敗
if [ "${BACKUP_REQUIRE_ENCRYPTION:-}" = "true" ]; then
    if [ -z "${BACKUP_GPG_RECIPIENT:-}" ]; then
        echo "ERROR: BACKUP_REQUIRE_ENCRYPTION=true 但 BACKUP_GPG_RECIPIENT 未設定。"
        echo "       請在 .env 設定收件者 Key ID 或 email，並匯入 GPG 公鑰後重新啟動。"
        exit 1
    fi
    if ! gpg --list-keys "${BACKUP_GPG_RECIPIENT}" > /dev/null 2>&1; then
        echo "ERROR: GPG keyring 找不到 '${BACKUP_GPG_RECIPIENT}'。"
        echo "       (1) 把 backup public key 放在 secrets/backup_gpg_pubkey.asc"
        echo "       (2) 重啟 db-backup container"
        echo "       詳見內部備份 GPG 設定 SOP"
        exit 1
    fi
    echo "✅ GPG 加密已設定，收件者: ${BACKUP_GPG_RECIPIENT}"
fi

# R36-5/6/7: rclone 異地上傳設定
# 把 secrets/rclone.conf mount 到 /run/secrets/rclone_config，
# rclone CLI 預設讀 ~/.config/rclone/rclone.conf — 在此建立 symlink。
# -s 容許空 placeholder（disable 異地上傳）
if [ -s "/run/secrets/rclone_config" ]; then
    mkdir -p /root/.config/rclone
    ln -sf /run/secrets/rclone_config /root/.config/rclone/rclone.conf
    if [ -n "${BACKUP_RCLONE_REMOTES:-}" ]; then
        echo "📤 異地備份目標 (rclone): ${BACKUP_RCLONE_REMOTES}"
        # 驗證每個 remote 設定存在
        IFS=',' && for remote_path in $BACKUP_RCLONE_REMOTES; do
            remote_name="${remote_path%%:*}"
            if ! rclone config show "$remote_name" > /dev/null 2>&1; then
                echo "ERROR: rclone remote '$remote_name' 未在 secrets/rclone.conf 設定"
                echo "       詳見內部備份設定 SOP"
                exit 1
            fi
        done
        unset IFS
    fi
fi

# 確保 log 檔可寫（root 執行，root 擁有）
touch /var/log/backup.log

# 建立 crontab：將所有環境變數傳入 cron job
# 排除 BACKUP_SCHEDULE：其值含空格（"0 2 * * *"），被 pg_backup.sh 以 `. /etc/backup.env`
# source 時會被當指令執行（sh: line N: 2: not found）。此變數僅 cron 用、備份腳本不需要。
env | grep -E '^(POSTGRES_|PGPASSWORD|PGHOST|PGPORT|BACKUP_|RSYNC_|POSTGRES_PASSWORD_FILE|GNUPGHOME|WATCHDOG_)' | grep -v '^BACKUP_SCHEDULE=' > /etc/backup.env
# 644→600（Qodo review on PR #82）：本檔現含 WATCHDOG_PING_TOKEN，全域可讀會讓容器內
# 其他行程偷看 token 去偽造心跳／查詢 /status；root 執行 cron 不受影響。
chmod 600 /etc/backup.env

# 用 `crontab -` 安裝 root crontab 到 crond 實際讀取的 spool 目錄（不依賴 /etc/crontabs 路徑，
# 這正是先前 /etc/crontabs/backup 從未被執行的疑因）。以 root 跑 → 可寫 /backups、GPG 用 root keyring。
echo "${SCHEDULE} . /etc/backup.env && /usr/local/bin/pg_backup.sh >> /var/log/backup.log 2>&1" | crontab -

echo "✅ Cron 排程已設定，啟動 crond..."

# 執行一次立即備份（容器首次啟動時，以 backup 使用者身分）
if [ "${BACKUP_ON_START:-false}" = "true" ]; then
    echo "📦 執行啟動備份..."
    sh -c '. /etc/backup.env && /usr/local/bin/pg_backup.sh'
fi

# 啟動 cron（前景模式）
crond -f -l 2
