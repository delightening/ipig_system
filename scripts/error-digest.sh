#!/usr/bin/env bash
# error-digest.sh — 從 Loki 拉 ipig-api 報錯並彙總（給人或本機 AI 定期讀）。
#
# 用途：把累積在 Loki 的 ERROR / 5xx 依「錯誤型態」分類，附首/末發生時間，
#       讓你/AI 一眼看出「哪些還在發生 vs 已修」「哪個端點在噴錯」。
#
# 需求：Loki 在 127.0.0.1:3100（見 docker-compose）；python3、curl。
# 用法：
#   bash scripts/error-digest.sh            # 預設近 7 天、ipig-api
#   bash scripts/error-digest.sh 30         # 近 30 天
#   bash scripts/error-digest.sh 3 ipig-web # 近 3 天、web 容器
#
# 注意：Loki 單次 query 上限 1000 筆，本工具取「最近 N 天內最新 1000 筆 ERROR」。
#       錯誤行不含觸發者 user_id（api tracing 目前未記 actor）；「誰」需另比對稽核日誌。
set -euo pipefail

DAYS="${1:-7}"
CONTAINER="${2:-ipig-api}"
LOKI="${LOKI_URL:-http://127.0.0.1:3100}"

END=$(date +%s)000000000
START=$(date -d "${DAYS} days ago" +%s)000000000

curl -sG "${LOKI}/loki/api/v1/query_range" \
  --data-urlencode "query={container_name=\"${CONTAINER}\"} |~ \"(?i)ERROR|status=5[0-9][0-9]\"" \
  --data-urlencode "start=${START}" \
  --data-urlencode "end=${END}" \
  --data-urlencode "limit=1000" \
  --data-urlencode "direction=backward" \
| CONTAINER="${CONTAINER}" python3 -c '
import sys, os, json, re, collections, datetime
try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass
d = json.load(sys.stdin)
ANSI = re.compile(r"\x1b\[[0-9;]*m")
rows = []
for s in d.get("data", {}).get("result", []):
    for v in s.get("values", []):
        rows.append((int(v[0]), ANSI.sub("", v[1])))
if not rows:
    print("（此區間無 ERROR / 5xx）"); sys.exit(0)
rows.sort()
fmt = lambda ns: datetime.datetime.fromtimestamp(ns / 1e9).strftime("%m-%d %H:%M")
cn = os.environ.get("CONTAINER", "")
print("容器=%s 取樣 %d 行 | %s ~ %s" % (cn, len(rows), fmt(rows[0][0]), fmt(rows[-1][0])))
def cat(line):
    m = re.search(r"no column found for name: (\w+)", line)
    if m: return "FromRow缺欄: " + m.group(1)
    for pat, lab in [(r"pg_advisory", "DB: pg_advisory(42883)"),
                     (r"stack depth limit", "DB: stack depth exceeded"),
                     (r"null value in column", "DB: NOT NULL 違反"),
                     (r"violates foreign key", "DB: FK 違反"),
                     (r"duplicate key", "DB: duplicate key"),
                     (r"does not exist", "DB: column/object 不存在")]:
        if re.search(pat, line): return lab
    m = re.search(r"uri=(\S+).*?status=(5\d\d)", line)
    if m: return "5xx " + m.group(2) + " " + re.sub(r"[?].*", "", m.group(1))
    if "panic" in line.lower(): return "PANIC"
    return None
agg = collections.defaultdict(lambda: [0, 0, 0])
for ts, line in rows:
    c = cat(line)
    if not c: continue
    a = agg[c]; a[0] += 1
    a[1] = ts if a[1] == 0 else min(a[1], ts); a[2] = max(a[2], ts)
if not agg:
    print("（有 ERROR 行但無可分類的明確錯誤型態，建議直接看 Grafana）"); sys.exit(0)
print("%5s  %11s  %11s  類型" % ("次數", "首次", "末次"))
for c, (n, fi, la) in sorted(agg.items(), key=lambda x: -x[1][0]):
    print("%5d  %11s  %11s  %s" % (n, fmt(fi), fmt(la), c))
'
