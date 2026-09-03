#!/bin/bash
# CI Security Guard: Handler Access Control Scan
#
# 檢測三種形狀（前兩種會擋 CI，第三種只發警告）：
#   [1/3] _current_user（底線 = 宣告了身分卻不用）+ Path 參數        → EXIT 1
#   [3/3] _current_user + 接受 body（Json/Multipart）的寫入操作      → EXIT 1
#   [2/3] 有 current_user + Path( 但全檔查無任何 access check        → 只警告
#
# ⚠️ **這支腳本能抓到什麼、抓不到什麼，講清楚比較安全**（R66-D3，2026-09-03 訂正）：
#   - 抓得到：「宣告了 CurrentUser 卻加底線不用」這個字面信號，也就是粗心。
#   - 抓不到：「用了 current_user，但沒有做物件層授權」——而那正是 R66-D2／R75
#     那一整條線要防的東西。[2/3] 只是 file-level 的粗篩，不是保證。
#   要真正在結構上擋住漏授權，得走型別層（`Scoped::<T>::authorize(...)`，見 TODO 的 R94-4），
#   讓漏掉授權的程式碼編譯不過，而不是靠 grep。
#   （此處刻意寫 turbofish 而非 `Scoped<T>`：後者是描述型別時的自然寫法，但**實際 code 的
#   syntax 是 `Scoped::<T>::authorize`**，而下面 [2/3] 的 `ACCESS_PATTERNS` 比對的是實際
#   syntax。兩處寫法不一致會讓讀者以為 pattern 漏匹配——2026-09-03 CodeRabbit 即如此誤讀。）
#
# ⚠️ 本檔頭原本還寫著「3. 新增 handler 不在已知白名單中 = 需要 review」與
#   「白名單檔案缺失時 fail-open 但發出警告」——**那個白名單機制從來沒有實作過**
#   （全檔查無 whitelist／allowlist／known_handlers）。已刪除該敘述，不留假承諾。
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

# ⚠️ R66-D3（2026-09-03）：這一格原本是空殼——只印上面那行標題，然後定義
# ACCESS_PATTERNS 與 WARN_COUNT 就結束，兩個變數在全檔其餘地方一次都沒被用到。
# 於是「handler 有沒有真的做授權檢查」這個問題從來沒有被問過。
#
# 現在補實，但刻意只發警告、不設 EXIT_CODE=1：這是 file-level 比對
#（只問「這個檔案裡有沒有出現任何一種 access check」），誤報本來就多——
# 例如 /me 這類端點合法地不需要物件層授權、或授權發生在 service 層而非 handler 檔內。
# 直接當硬閘會製造大量假紅，那多半正是它當初被留空的原因。
# 先讓數字浮出來，再依實際命中決定要不要收斂成 function-level 或升級為硬閘。
#
# 判準刻意保守：只看同時有 current_user 與 Path(（有身分、也有物件 id）的檔案，
# 且該檔完全沒有任何一種 access check 字樣才報。
#
# ⚠️ **不要把 `Scoped<` 加進這串**（2026-09-03，CodeRabbit 曾建議兩種形式都匹配）：
# 這一格的語意是「檔案裡出現任一 access check 字樣 → 不警告」，所以每加一個字樣就是
# 放寬一次。而 `Scoped<T>` 這種寫法在本 repo 只出現在**註解裡**（例如
# `handlers/amendment.rs` 的「PI 寫入授權前移至 Scoped<AmendmentWrite>」），實際呼叫一律是
# `access::Scoped::<T>::authorize(...)`。把 `Scoped<` 加進來，等於讓「註解提過型別授權但
# 實際沒做」的檔案免於警告——那正是這一格要抓的東西。
# 當日實測：8 個被警告的檔案裡**沒有任何一個**含 `Scoped<` 或 `Scoped::`，
# 該建議要防的誤報一次都沒發生。
#
# ⚠️ 附帶查出：**`Scoped::` 這一項目前是冗餘的**（同日 mutation 驗證：把它從本串移除，
# 警告數 8 → 8、一個都沒多）。原因是 `Scoped::<T>::authorize(...)` 同時命中後面的
# `authorize\(`。保留它有兩個理由：一是讓「型別層授權也算 access check」這件事在
# pattern 上看得見，二是萬一日後出現不叫 `authorize` 的 `Scoped::<T>::xxx()`，
# 這一項才會接住。**但它現在不是防線，改動時不要把它當成有效的那一格。**
ACCESS_PATTERNS="require_permission|is_admin|has_permission|require_animal_access|require_protocol|check_resource_access|check_amendment_access|check_attachment_permission|require_calendar_admin|require_reauth_token|Scoped::|authorize\("

WARN_COUNT=0
echo "  (file-level scan — informational only，不影響 exit code)"

for file in $(grep -rl "current_user" "$HANDLERS_DIR" --include="*.rs" 2>/dev/null || true); do
    # 沒有 Path( 就沒有「別人的資源」可被越權存取，跳過
    if ! grep -q "Path(" "$file"; then
        continue
    fi
    if ! grep -qE "$ACCESS_PATTERNS" "$file"; then
        echo "  ⚠️  $file — 有 current_user + Path( 但全檔查無 access check"
        WARN_COUNT=$((WARN_COUNT + 1))
    fi
done

if [ "$WARN_COUNT" -eq 0 ]; then
    echo "  ✅ 無檔案落入此形狀"
else
    echo "  ⚠️  $WARN_COUNT 個檔案值得人工看一眼（不擋 CI）"
fi

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
