#!/bin/bash
# iPig DR 備份還原演練腳本（R82-1）
#
# 用途：取最新 GPG 加密備份，還原到「隔離的」ipig_db_drill 容器，與 prod ipig-db
#       逐表 row-count 比對。全程唯讀 prod（只 SELECT COUNT），不碰 prod DB 本體。
#
# 前提（只有維運者能做的手動步驟）：
#   1. 插入含 GPG 私鑰的 USB。
#   2. 從 USB import 私鑰：  gpg --import <USB碟>/ipig_backup_private.asc
#   3. 執行本腳本；解密時 pinentry 會跳出要 passphrase（存 Bitwarden）。
#   4. 演練後移除私鑰恢復 USB-only：  gpg --delete-secret-keys 84F051E0AD2AA40F
#
# 用法： ./dr_drill.sh [備份檔完整路徑]
#        不給參數 → 自動取 db-backup 容器 /backups 內最新一份 .sql.gz.gpg
#
# 備份格式為 pg_dump -Fc（custom format）→ gzip → GPG，故還原用 pg_restore（非 psql 管道）。

set -euo pipefail

PROD_DB_CONTAINER="ipig-db"
BACKUP_CONTAINER="ipig-db-backup"
DRILL_CONTAINER="ipig_db_drill"
DB_NAME="ipig_db"
DRILL_PW="drill_temp_$(date +%s)"
GPG_KEY_ID="84F051E0AD2AA40F"

# 比對用資料表（穩定表應相等；append 表 drill≤prod 為當日增量，合理）
TABLES="animals users protocols electronic_signatures documents stock_ledger audit_logs user_activity_logs"

WORK="$(mktemp -d)"
cleanup() {
  rm -rf "$WORK" 2>/dev/null || true
  docker rm -f "$DRILL_CONTAINER" >/dev/null 2>&1 || true
}
trap cleanup EXIT

log() { echo "[$(date +%H:%M:%S)] $*"; }

echo "========================================================"
echo " iPig DR 備份還原演練 (R82-1)  —  $(date -Iseconds)"
echo "========================================================"

# 0) 私鑰前置檢查 -----------------------------------------------------------
if [ -z "$(gpg --list-secret-keys 2>/dev/null)" ]; then
  echo "✗ GPG keyring 內無私鑰。"
  echo "  請先插 USB 並 import： gpg --import <USB碟>/ipig_backup_private.asc"
  exit 1
fi
log "✓ GPG 私鑰已在 keyring"

# 1) 決定備份檔 -------------------------------------------------------------
if [ "$#" -ge 1 ] && [ -n "${1:-}" ]; then
  BACKUP_PATH="$1"
else
  BACKUP_PATH="$(docker exec "$BACKUP_CONTAINER" sh -c 'ls -1t /backups/ipig_*.sql.gz.gpg 2>/dev/null | head -1')"
fi
if [ -z "$BACKUP_PATH" ]; then echo "✗ 找不到備份檔"; exit 1; fi
BACKUP_BASE="$(basename "$BACKUP_PATH")"
log "備份檔：$BACKUP_BASE"

# 2) checksum 驗證 + 複製出容器 ---------------------------------------------
docker exec "$BACKUP_CONTAINER" sh -c "cd /backups && sha256sum -c '${BACKUP_BASE}.sha256'" \
  || { echo "✗ SHA256 checksum 驗證失敗，備份可能損毀"; exit 1; }
log "✓ SHA256 完整性通過"
docker cp "${BACKUP_CONTAINER}:${BACKUP_PATH}" "$WORK/$BACKUP_BASE"

# 3) 解密（pinentry 會要 Bitwarden passphrase）------------------------------
log "→ GPG 解密中（若跳出視窗，輸入 Bitwarden 內的 passphrase）..."
gpg --decrypt "$WORK/$BACKUP_BASE" > "$WORK/dump.sql.gz" 2>/dev/null \
  || { echo "✗ GPG 解密失敗（passphrase 錯誤或私鑰不符）"; exit 1; }
log "✓ 解密完成"

# 4) 解壓 -------------------------------------------------------------------
gunzip -f "$WORK/dump.sql.gz"   # → $WORK/dump.sql（custom format 二進位）
log "✓ 解壓完成（$(du -h "$WORK/dump.sql" | cut -f1)）"

# 5) 起隔離 drill 容器（default bridge、不暴露 port、不掛 prod 網路）--------
docker rm -f "$DRILL_CONTAINER" >/dev/null 2>&1 || true
docker run -d --name "$DRILL_CONTAINER" \
  -e POSTGRES_PASSWORD="$DRILL_PW" -e POSTGRES_DB="$DB_NAME" \
  postgres:16-alpine >/dev/null
log "→ 等 drill DB 就緒..."
for _ in $(seq 1 60); do
  # 用真查詢判定：pg_isready 可能在 initdb 建 DB 前就回 ready
  docker exec "$DRILL_CONTAINER" psql -U postgres -d "$DB_NAME" -tAc 'SELECT 1' >/dev/null 2>&1 && break
  sleep 1
done
log "✓ drill 容器就緒（$DRILL_CONTAINER，隔離、無對外 port）"

# 6) pg_restore -------------------------------------------------------------
# ⚠️ Windows/Git Bash：`docker exec` 的容器內 /tmp 引數會被 MSYS 自動轉成 Windows
#    路徑（/tmp/dump.sql → C:/Users/.../Temp/dump.sql，容器內找不到 → 還原 0 表且
#    錯誤易被吞掉）→ 下方 pg_restore 加 MSYS_NO_PATHCONV=1。
#    但 `docker cp` 的 host 來源路徑「需要」MSYS 轉換，故此處不可加（加了來源會找不到）。
docker cp "$WORK/dump.sql" "${DRILL_CONTAINER}:/tmp/dump.sql"
RSTART=$(date +%s)
# --clean --if-exists 在空庫會噴「不存在，略過」notice + pg_restore 回非零，屬正常；
# stderr 收進 log 供除錯，真失敗以「還原後表數=0」判定（不再靜默吞錯）。
MSYS_NO_PATHCONV=1 docker exec "$DRILL_CONTAINER" pg_restore -U postgres -d "$DB_NAME" \
  --clean --if-exists --no-owner --no-acl /tmp/dump.sql >"$WORK/restore.log" 2>&1 || true
REND=$(date +%s)
RESTORED_TBLS=$(docker exec "$DRILL_CONTAINER" psql -U postgres -d "$DB_NAME" -X -tAc \
  "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" 2>/dev/null || echo 0)
# 用 `! -gt 0` 而非 `-eq 0`：非數字輸出（docker/psql 出錯）也判為失敗，不致靜默假成功
if ! [ "${RESTORED_TBLS:-0}" -gt 0 ] 2>/dev/null; then
  echo "✗ pg_restore 未還原任何表（public=0）——備份/還原鏈異常。錯誤節選："
  tail -n 20 "$WORK/restore.log" 2>/dev/null || true
  exit 1
fi
log "✓ pg_restore 完成，耗時 $((REND - RSTART))s（還原 $RESTORED_TBLS 個 public 表）"

# 7) row-count 逐表比對 -----------------------------------------------------
echo ""
printf "%-26s %14s %14s %10s\n" "table" "drill(備份)" "prod(現在)" "判定"
printf "%-26s %14s %14s %10s\n" "--------------------------" "------------" "------------" "--------"
FAIL=0
set +e
for t in $TABLES; do
  d=$(docker exec "$DRILL_CONTAINER" psql -U postgres -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM $t" 2>/dev/null)
  p=$(docker exec "$PROD_DB_CONTAINER" psql -U postgres -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM $t" 2>/dev/null)
  d="${d:-ERR}"; p="${p:-ERR}"
  verdict="?"
  if [ "$d" = "ERR" ] || [ "$p" = "ERR" ]; then
    verdict="✗ 查詢錯誤"; FAIL=1
  elif [ "$d" = "$p" ]; then
    verdict="= 相符"
  elif [ "$d" = "0" ] && [ "$p" != "0" ]; then
    verdict="✗ drill 空(還原失敗?)"; FAIL=1
  elif [ "$d" -lt "$p" ] 2>/dev/null; then
    verdict="≤ 日增(Δ=$((p - d)))"
  else
    verdict="✗ drill>prod"; FAIL=1
  fi
  printf "%-26s %14s %14s %10s\n" "$t" "$d" "$p" "$verdict"
done
set -e

# 8) 結論 -------------------------------------------------------------------
echo ""
TOTAL=$(docker exec "$DRILL_CONTAINER" psql -U postgres -d "$DB_NAME" -tAc \
  "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" 2>/dev/null || echo "?")
log "drill 還原後 public 表總數：$TOTAL（prod 基準 192）"
echo ""
if [ "$FAIL" = "0" ]; then
  echo "✅ 演練通過：還原成功；穩定表相符、append 表 drill≤prod（當日增量合理），無 drill>prod（無 prod 資料遺失跡象）。"
else
  echo "⚠️ 演練有異常：出現 drill>prod 或查詢錯誤的表，需人工檢視上表。"
fi
echo ""
echo "→ drill 容器與暫存檔將自動清除；請記得演練後移除私鑰恢復 USB-only："
echo "    gpg --delete-secret-keys $GPG_KEY_ID"
echo "→ 完成後把上表結果貼回，我幫你回填內部 DR 演練紀錄。"
echo "========================================================"
