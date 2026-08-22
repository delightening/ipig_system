#!/usr/bin/env bash
# 駁回兩筆誤送審計畫（APIG-115011 / APIG-115014）。
# 用法：TOKEN="<admin Bearer token>" bash scripts/reject_erroneous_protocols.sh
# 不含任何密鑰；token 由呼叫者提供（短效）。
set -euo pipefail

: "${TOKEN:?請先設定 TOKEN 環境變數（admin Bearer token）}"
API="${API:-http://127.0.0.1:8000}"
REMARK="誤送審：錯誤送出之申請，經系統管理員確認後駁回（未走新申請須知簽核流程）。"

# id|代號
P1="ad8618e5-6214-4561-a07b-7e852d178413|APIG-115011"
P2="3192e093-be5e-4203-8f64-51b914f3c355|APIG-115014"

reject() {
  local id="${1%%|*}" code="${1##*|}"
  echo "=== 駁回 ${code} (${id}) ==="
  curl -sS -o /tmp/_reject_body.json -w 'HTTP=%{http_code}\n' \
    -X POST "${API}/api/v1/protocols/${id}/status" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json" \
    --data "{\"to_status\":\"REJECTED\",\"remark\":\"${REMARK}\"}"
  echo "回應：$(cat /tmp/_reject_body.json)"
  echo
}

reject "$P1"
reject "$P2"

echo "=== 驗證最終狀態 ==="
for p in "$P1" "$P2"; do
  id="${p%%|*}"; code="${p##*|}"
  st=$(curl -sS "${API}/api/v1/protocols/${id}" -H "Authorization: Bearer ${TOKEN}" \
        | python -c "import sys,json;print(json.load(sys.stdin).get('status'))")
  echo "${code}: status=${st}  (預期 REJECTED)"
done
