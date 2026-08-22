#!/bin/bash
#
# E2E 後端日誌分析腳本
#
# 用法：
#   bash scripts/analyze-e2e-logs.sh
#
# 功能：
# - 分析最近 5 分鐘的後端日誌
# - 檢查 401 錯誤
# - 檢查 JWT 相關日誌
# - 檢查 Session 相關日誌
# - 統計 API 請求
#

set -euo pipefail

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 檢查 Docker 是否運行
if ! docker compose ps api &>/dev/null; then
    echo -e "${RED}錯誤：Docker services 未運行${NC}"
    echo "請先執行：docker compose up -d"
    exit 1
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  E2E 後端日誌分析（最近 5 分鐘）${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 1. 檢查 401 錯誤
echo -e "${YELLOW}[1/5] 401 Unauthorized 錯誤${NC}"
echo "---------------------------------------------"
UNAUTHORIZED_LOGS=$(docker compose logs api --since 5m 2>&1 | grep -E "401|Unauthorized" || true)
if [ -z "$UNAUTHORIZED_LOGS" ]; then
    echo -e "${GREEN}✅ 無 401 錯誤${NC}"
else
    echo -e "${RED}❌ 發現 401 錯誤：${NC}"
    echo "$UNAUTHORIZED_LOGS" | tail -20
fi
echo ""

# 2. 檢查 JWT 相關日誌
echo -e "${YELLOW}[2/5] JWT 相關日誌${NC}"
echo "---------------------------------------------"
JWT_LOGS=$(docker compose logs api --since 5m 2>&1 | grep -iE "jwt|token|expired" || true)
if [ -z "$JWT_LOGS" ]; then
    echo -e "${GREEN}✅ 無 JWT 相關錯誤${NC}"
else
    echo -e "${YELLOW}⚠️  JWT 相關日誌：${NC}"
    echo "$JWT_LOGS" | tail -20
fi
echo ""

# 3. 檢查 Session 相關日誌
echo -e "${YELLOW}[3/5] Session 相關日誌${NC}"
echo "---------------------------------------------"
SESSION_LOGS=$(docker compose logs api --since 5m 2>&1 | grep -iE "session|login|logout|auth" || true)
if [ -z "$SESSION_LOGS" ]; then
    echo -e "${GREEN}✅ 無 Session 相關日誌${NC}"
else
    echo "Session 相關日誌（最近 20 行）："
    echo "$SESSION_LOGS" | tail -20
fi
echo ""

# 4. 檢查錯誤日誌（ERROR 級別）
echo -e "${YELLOW}[4/5] ERROR 級別日誌${NC}"
echo "---------------------------------------------"
ERROR_LOGS=$(docker compose logs api --since 5m 2>&1 | grep -E "ERROR|error|Error" || true)
if [ -z "$ERROR_LOGS" ]; then
    echo -e "${GREEN}✅ 無 ERROR 級別日誌${NC}"
else
    echo -e "${RED}❌ 發現 ERROR：${NC}"
    echo "$ERROR_LOGS" | tail -20
fi
echo ""

# 5. 統計 API 請求
echo -e "${YELLOW}[5/5] API 請求統計${NC}"
echo "---------------------------------------------"
REQUEST_STATS=$(docker compose logs api --since 5m 2>&1 | grep -oE "GET|POST|PUT|DELETE|PATCH" | sort | uniq -c | sort -rn || true)
if [ -z "$REQUEST_STATS" ]; then
    echo -e "${YELLOW}⚠️  無法統計 API 請求（可能日誌格式不同）${NC}"
else
    echo "請求方法統計："
    echo "$REQUEST_STATS"
fi
echo ""

# 總結
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  分析完成${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 建議
echo -e "${GREEN}💡 建議：${NC}"
echo "  1. 如果有 401 錯誤，檢查 JWT_EXPIRATION_MINUTES 設定"
echo "  2. 如果有 JWT expired，增加 JWT TTL（建議 >= 15 分鐘）"
echo "  3. 如果有 Session 失效，執行配置驗證："
echo "     cd frontend && npx tsx e2e/scripts/verify-config.ts"
echo "  4. 查看完整日誌："
echo "     docker compose logs api --since 5m"
echo ""
