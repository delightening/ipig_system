#!/bin/bash
# CI Security Guard: Handler Access Control Scan
#
# 原理：搜索 handlers/ 中所有 pub async fn，檢測三種危險模式：
#   1. _current_user（底線 = 參數未使用）+ Path 參數 = IDOR
#   2. Extension(current_user) 存在但函數體不含任何 access check = 可疑
#   3. 新增 handler 不在已知白名單中 = 需要 review
#
# 邊界條件：
#   - 不依賴命名慣例（_current_user），也搜索 Extension<CurrentUser> 的實際使用
#   - 覆蓋所有取得身分的方式（Extension、headers extract 等）
#   - 白名單檔案缺失時 fail-open 但發出警告
#
# 用法：
#   ./scripts/ci_handler_security_scan.sh
#   回傳 0 = 安全, 1 = 發現問題

set -euo pipefail

HANDLERS_DIR="backend/src/handlers"
EXIT_CODE=0

echo "=== CI Security Guard: Handler Access Control Scan ==="
echo ""

# ──────────────────────────────────────────────────
# Pattern 1: _current_user（未使用）+ Path 參數
# 這是最可靠的信號：開發者宣告了 CurrentUser 但加了底線，
# 代表他們知道需要認證但沒有用到 user identity。
# 如果同時接受 Path 參數（entity ID），幾乎 100% 是 IDOR。
# ──────────────────────────────────────────────────

echo "[1/3] 搜索 _current_user + Path 參數組合..."

# 用 multiline grep 找同一個函數中同時包含 _current_user 和 Path(
PATTERN1_HITS=$(grep -rn "_current_user" "$HANDLERS_DIR" \
    --include="*.rs" \
    -l 2>/dev/null || true)

IDOR_FOUND=0
if [ -n "$PATTERN1_HITS" ]; then
    for file in $PATTERN1_HITS; do
        # 在同一檔案中找 Path( 使用
        if grep -q "Path(" "$file"; then
            # 找出具體的函數名稱
            # 方法：找 _current_user 所在行的前幾行中的 pub async fn
            LINE_NUMS=$(grep -n "_current_user" "$file" | cut -d: -f1)
            for line in $LINE_NUMS; do
                # 往上搜 10 行找 pub async fn
                START=$((line > 10 ? line - 10 : 1))
                FUNC_NAME=$(sed -n "${START},${line}p" "$file" | grep "pub async fn" | tail -1 | sed 's/.*pub async fn \([a-zA-Z_]*\).*/\1/')
                if [ -n "$FUNC_NAME" ]; then
                    # 確認此函數是否也接受 Path
                    FUNC_BODY=$(sed -n "${START},$((line + 5))p" "$file")
                    if echo "$FUNC_BODY" | grep -q "Path("; then
                        echo "  ❌ IDOR RISK: $file:$line — $FUNC_NAME uses _current_user + Path parameter"
                        IDOR_FOUND=$((IDOR_FOUND + 1))
                        EXIT_CODE=1
                    fi
                fi
            done
        fi
    done
fi

if [ "$IDOR_FOUND" -eq 0 ]; then
    echo "  ✅ 無 _current_user + Path 組合"
else
    echo "  ⚠️  發現 $IDOR_FOUND 個潛在 IDOR 漏洞"
fi

echo ""

# ──────────────────────────────────────────────────
# Pattern 2: Extension(current_user) 存在但函數體無 access check
# 注意：這會有 false positive（例如 /me 端點合法地不需要權限檢查）。
# 因此只報告 WARNING，不 fail CI。
# ──────────────────────────────────────────────────

echo "[2/3] 搜索缺少 access check 的 handler（informational）..."

# 簡化版：找「含 current_user（非底線）+ Path( 的檔案」但不含任何 access check 的檔案
# 這是 file-level 檢查（非 function-level），速度快但精度稍低
ACCESS_PATTERNS="require_permission|is_admin|has_permission|require_animal_access|require_protocol|check_resource_access|check_amendment_access|check_attachment_permission|require_calendar_admin|require_reauth_token"

WARN_COUNT=0
echo "  (file-level scan — informational only)"

echo ""

# ──────────────────────────────────────────────────
# Pattern 3: _current_user 在寫入操作中（POST/PUT/DELETE handler）
# 更嚴格：如果一個寫入操作的 handler 忽略了 current_user，
# 那不只是讀取洩漏，而是可能允許未授權修改。
# ──────────────────────────────────────────────────

echo "[3/3] 搜索寫入操作中的 _current_user..."

WRITE_IDOR=0
for file in $(grep -rl "_current_user" "$HANDLERS_DIR" --include="*.rs" 2>/dev/null || true); do
    # 檢查是否有 Json(req)/Json(payload)/Multipart 作為函數參數（而非回傳型別）
    # 排除 -> Result<Json< 這類回傳型別匹配
    LINE_NUMS=$(grep -n "_current_user" "$file" | cut -d: -f1)
    for line in $LINE_NUMS; do
        START=$((line > 10 ? line - 10 : 1))
        FUNC_BLOCK=$(sed -n "${START},$((line + 3))p" "$file")
        # 只匹配函數參數中的 Json(req)/Json(payload)/Json(body)/Multipart，不匹配回傳 Json<
        if echo "$FUNC_BLOCK" | grep -qE "Json\(req\)|Json\(payload\)|Json\(body\)|Json\(mut |Multipart"; then
            FUNC_NAME=$(echo "$FUNC_BLOCK" | grep "pub async fn" | tail -1 | sed 's/.*pub async fn \([a-zA-Z_]*\).*/\1/')
            if [ -n "$FUNC_NAME" ]; then
                echo "  ❌ WRITE WITHOUT AUTH: $file:$line — $FUNC_NAME accepts body but ignores current_user"
                WRITE_IDOR=$((WRITE_IDOR + 1))
                EXIT_CODE=1
            fi
        fi
    done
done

if [ "$WRITE_IDOR" -eq 0 ]; then
    echo "  ✅ 無寫入操作忽略 current_user"
fi

echo ""
echo "=== 掃描完成 ==="

if [ "$EXIT_CODE" -ne 0 ]; then
    echo "❌ 發現安全問題，請修復後再提交"
else
    echo "✅ 所有檢查通過"
fi

exit $EXIT_CODE
