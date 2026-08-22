#!/usr/bin/env bash
# 從一顆「已套用全部 migration 的乾淨 DB」產生 squash 用的 003_seed.sql。
#
# 為什麼需要這支：`002_schema.sql` 由 `pg_dump --schema-only` 產生，**天生不含 INSERT**。
# 但這套系統有大量「系統定義資料」原本是靠舊 migration 裡的一次性 INSERT 建立的
# ——squash 掉那些檔案之後就沒了，新環境會是一個 schema 完整但完全不能用的空殼。
#
# 2026-08-20 實測，缺 seed 造成三種不同的失敗（每一種都要跑到才會發現）：
#   1. roles/role_permissions 為 0 → 測試在 pi_role_id() panic
#   2. 缺系統帳號 00000000-…-0001 → 稽核寫入 FK 違反（23503），症狀是「刪使用者失敗」
#   3. 缺 species → 建立動物回 400（品種驗證查不到 'white'）
# 因此改為**系統性列舉**所有需要的表，而不是踩到一個補一個。
#
# ⚠️ 只匯出系統定義資料，不含業務交易或個人資料。清單見下方 SEED_TABLES，
#    新增表時請確認其內容性質，並在產生後跑 `node scripts/pii-scan.mjs --full`。
#
# 用法：
#   SOURCE_CONTAINER=ipig-np-db ./scripts/migration/make-squash-seed.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$SCRIPT_DIR/../../backend/migrations/003_seed.sql"
SRC="${SOURCE_CONTAINER:-ipig-np-db}"
DB="${PGDATABASE:-ipig_db}"

q() { MSYS_NO_PATHCONV=1 docker exec "$SRC" psql -U "${PGUSER:-postgres}" -d "$DB" -Atc "$1"; }

# 依 FK 相依順序排列——被參照的表必須先插入。
# facilities → buildings → zones → pens 是明確的階層；departments 被 users 參照；
# roles/permissions 被 role_permissions 參照。
SEED_TABLES=(
  # 組織與設施結構
  facilities
  buildings
  zones
  pens
  departments
  # 權限體系
  roles
  permissions
  role_permissions
  # 主檔／參考資料
  species
  animal_sources
  treatment_drug_options
  sku_categories
  sku_subcategories
  chart_of_accounts
  blood_test_templates
  blood_test_panels
  blood_test_panel_items
  blood_test_presets
  # 系統設定
  system_settings
  security_alert_config
  expiry_notification_config
  notification_routing
  data_retention_policies
  audit_chain_known_breaks
)

# 產生某張表所有列的 INSERT（自動取欄位、自動處理型別與 NULL）。
#
# ⚠️ 取值一律用 `format('%L', col)` 而非 `to_jsonb(x)->>col`。
#    2026-08-20 踩過：`to_jsonb()->>` 會把值先轉成 **JSON 表示法**，對兩種型別會壞掉——
#      * `jsonb` 欄位：多包一層，或引號跳脫方式改變
#      * text 陣列（`_text`）：輸出 JSON 的 `["mg","ml"]`，但 Postgres 陣列字面值
#        要的是 `{mg,ml}` → 報 `malformed array literal`
#    `format('%L', col)` 直接產生該型別**正確的 SQL 字面值**（含 NULL → 'NULL'），
#    不經過 JSON 這一層，所有型別都正確。
gen_table() {
  local t="$1"
  # 動態組出 format() 的參數列，每欄一個 %L
  local sel
  sel=$(q "
SELECT string_agg('format(''%L'', ' || quote_ident(column_name) || ')', ' || '', '' || '
                  ORDER BY ordinal_position)
  FROM information_schema.columns
 WHERE table_schema='public' AND table_name='$t';")
  local cols
  cols=$(q "
SELECT string_agg(quote_ident(column_name), ', ' ORDER BY ordinal_position)
  FROM information_schema.columns
 WHERE table_schema='public' AND table_name='$t';")
  q "SELECT 'INSERT INTO \"$t\" ($cols) VALUES (' || ($sel) || ') ON CONFLICT DO NOTHING;' FROM \"$t\";"
}

# ---- 先驗證 SEED_TABLES 的順序符合 FK 相依 ----
# 手排順序遲早會錯（2026-08-20 就把 blood_test_panel_items 排在 templates 之前，
# 套用時 FK 違反）。這裡用實際 FK 關係檢查：對每個「A 參照 B」且兩者都在清單裡的
# 情形，B 必須排在 A 前面。用 node 判斷，避免依賴 nl/awk/grep 的行為差異。
echo "=== 驗證表順序符合 FK 相依 ==="
FK_PAIRS=$(q "
SELECT DISTINCT c.conrelid::regclass::text || ' ' || c.confrelid::regclass::text
  FROM pg_constraint c
 WHERE c.contype='f' AND c.conrelid <> c.confrelid;")

if ! printf '%s' "$FK_PAIRS" | node -e '
let fk="";
process.stdin.on("data",c=>fk+=c);
process.stdin.on("end",()=>{
  const order = process.argv.slice(1);
  const idx = Object.fromEntries(order.map((t,i)=>[t,i]));
  let bad = 0;
  fk.trim().split("\n").forEach(line=>{
    const [child,parent] = line.trim().split(/\s+/);
    if(!child||!parent) return;
    if(!(child in idx) || !(parent in idx)) return;   // 只檢查清單內的表
    if(idx[parent] > idx[child]){
      console.log("  ❌ "+child+" (第 "+(idx[child]+1)+") 參照 "+parent+" (第 "+(idx[parent]+1)+") — parent 必須排在前面");
      bad++;
    }
  });
  if(bad){ console.log("ERROR: SEED_TABLES 順序違反 FK 相依，請依上方提示調整。"); process.exit(1); }
  console.log("  ✅ 順序正確（檢查了清單內所有 FK 關係）");
});
' "${SEED_TABLES[@]}"; then
  exit 1
fi
echo ""

{
cat <<'HEADER'
-- Squash 起點 §3：系統 seed data。
--
-- 由 scripts/migration/make-squash-seed.sh 自動產生，請勿手改——要改請改腳本後重跑。
--
-- 內容僅限**系統定義資料**：組織與設施結構、權限體系、主檔／參考資料、系統設定。
-- 不含任何業務交易資料或個人資料。
--
-- 冪等：全部 ON CONFLICT DO NOTHING，可安全重跑。
--
-- 註：app 啟動時的 ensure_required_permissions() / ensure_all_role_permissions()
-- 只補齊程式碼裡宣告的權限，**不建立角色本身**、也不補歷史授權對照，故必須有這支。
--
-- ⚠️ 插入順序依 FK 相依關係排列（facilities → buildings → zones → pens 等），
--    不要重排。
HEADER

echo ""
echo "-- ============ 系統使用者 ============"
echo "-- ⚠️ users 表**不整表匯出**——裡面是真實使用者，屬個人資料。"
echo "-- 這裡只取那個固定 UUID 的系統帳號：user_activity_logs.actor_user_id 有 FK"
echo "-- 指向 users，系統自動產生的稽核紀錄（排程／匯入／軟刪除連帶處理）用它當 actor，"
echo "-- 缺了會讓任何觸發稽核寫入的操作 FK 違反（23503）。"
echo "-- 安全性：password_hash 為無法通過驗證的字串且 is_active=false，不可能被登入。"
q "
SELECT 'INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, login_attempts, theme_preference, language_preference, created_at, updated_at, totp_enabled, version) VALUES ('
  || quote_literal(id::text) || '::uuid, ' || quote_literal(email) || ', '
  || quote_literal(password_hash) || ', ' || quote_literal(display_name) || ', '
  || is_internal || ', ' || is_active || ', ' || must_change_password || ', ' || login_attempts || ', '
  || quote_literal(theme_preference) || ', ' || quote_literal(language_preference) || ', '
  || 'now(), now(), ' || totp_enabled || ', ' || version || ') ON CONFLICT (id) DO NOTHING;'
FROM users WHERE id = '00000000-0000-0000-0000-000000000001';"

for t in "${SEED_TABLES[@]}"; do
  echo ""
  echo "-- ============ $t ============"
  gen_table "$t"
done
} > "$OUT"

echo "產出 $OUT（$(wc -l < "$OUT") 行）"
echo ""
echo "來源筆數（供比對）："
for t in "${SEED_TABLES[@]}"; do
  printf '  %-28s %s\n' "$t" "$(q "SELECT count(*) FROM \"$t\";")"
done
echo ""
echo "⚠️ 下一步必做：node scripts/pii-scan.mjs --full 確認無個資"
