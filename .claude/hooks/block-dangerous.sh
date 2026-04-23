#!/usr/bin/env bash
# PreToolUse hook: block dangerous commands during Rust refactor project.
#
# 用 Python 的 shlex 做**真正的** shell tokenization。關鍵好處：
# commit message 內的字串（例如 `git commit -m "... git push ..."`）
# 會被 shlex 視為**單一 token**，不會跟前面的 "git" 形成 [git, push] 相鄰
# tokens → 不誤攔 commit message。
#
# stdin 傳 JSON（hook 契約），因 heredoc 會搶 stdin 不能直接給 Python `-`，
# 改用 env var 傳 payload 讓 Python 讀取。
#
# 若 Python 不存在 → fail-open（放行，只記 warning）。

set -u

INPUT=$(cat)

# 優先 python3，fallback python
PY=""
if command -v python3 >/dev/null 2>&1; then
    PY=python3
elif command -v python >/dev/null 2>&1; then
    PY=python
else
    echo "[hook] python not found; fail-open (命令放行)" >&2
    exit 0
fi

# 用 env var 傳 JSON，避開 heredoc 和 pipe 搶 stdin 的問題
export HOOK_INPUT_JSON="$INPUT"

# Python 直接 exit 2 表示 block，exit 0 表示放行；訊息自行 print 到 stderr
"$PY" <<'PYEOF'
import json
import os
import shlex
import sys


NON_INVOKING = {"echo", "printf", "cat", "grep", "awk", "sed", "head", "tail", "tee", "xargs"}
INVOKING_WRAPPERS = {"rtk", "sudo", "time", "env", "exec", "nohup"}
CHAIN_OPS = {"&&", "||", ";", "|", "&", "("}

SQL_CLIENTS = {"psql", "mysql", "mysqladmin", "sqlplus", "mongo", "sqlx"}

DANGEROUS_SEQUENCES = [
    ("git push", ["git", "push"]),
    ("rtk git push", ["rtk", "git", "push"]),
    ("git reset --hard", ["git", "reset", "--hard"]),
    ("rtk git reset --hard", ["rtk", "git", "reset", "--hard"]),
    ("git force-push", ["git", "force-push"]),
    ("rtk git force-push", ["rtk", "git", "force-push"]),
    ("cargo sqlx migrate run", ["cargo", "sqlx", "migrate", "run"]),
    ("rtk cargo sqlx migrate run", ["rtk", "cargo", "sqlx", "migrate", "run"]),
]

TMP_PREFIXES = (
    "/tmp/",
    "/c/tmp/",
    "C:/tmp/",
    "/var/tmp/",
    "/c/Users/",   # 涵蓋 AppData/Local/Temp 子目錄
    "C:/Users/",
)


def is_invocation_position(tokens, i):
    if i == 0:
        return True
    prev = tokens[i - 1]
    if prev in NON_INVOKING:
        return False
    if prev in INVOKING_WRAPPERS or prev in CHAIN_OPS:
        return True
    return True  # 保守：預設視為 invocation


def has_sequence(tokens, pattern):
    plen = len(pattern)
    for i in range(len(tokens) - plen + 1):
        if tokens[i:i + plen] == pattern and is_invocation_position(tokens, i):
            return True
    return False


def check_rm_rf(tokens):
    for i in range(len(tokens) - 1):
        if tokens[i] != "rm":
            continue
        if not is_invocation_position(tokens, i):
            continue
        flags = tokens[i + 1]
        if not (flags.startswith("-") and "r" in flags.lower() and "f" in flags.lower()):
            continue
        for target in tokens[i + 2:]:
            if target.startswith("-"):
                continue
            if any(target.startswith(p) for p in TMP_PREFIXES):
                return False  # 明確 tmp 目標 → 放行
            return True  # 非 tmp → block
        return True  # rm -rf 沒目標 → 保守 block
    return False


def check_sql_danger(tokens):
    idx = 0
    while idx < len(tokens) and tokens[idx] in INVOKING_WRAPPERS:
        idx += 1
    if idx >= len(tokens) or tokens[idx] not in SQL_CLIENTS:
        return None
    # 只掃 SQL client 之後的 tokens
    for tok in tokens[idx + 1:]:
        upper = tok.upper()
        if "DROP TABLE" in upper:
            return "DROP TABLE in SQL client"
        if "DELETE FROM" in upper:
            return "DELETE FROM in SQL client"
    return None


def block(label, cmd):
    sys.stderr.write(f"\U0001f6d1 \u5371\u96aa\u64cd\u4f5c\u5df2\u6514\u622a\uff1a{label}\n")
    sys.stderr.write(f"  \u547d\u4ee4: {cmd}\n")
    sys.stderr.write("  \u82e5\u78ba\u5b9a\u8981\u57f7\u884c\uff0c\u8acb\u660e\u78ba\u8ddf\u4f7f\u7528\u8005\u78ba\u8a8d\u5f8c\u518d\u624b\u52d5\u57f7\u884c\n")
    sys.stderr.write("  \uff08\u6216\u5728\u672c\u6b21 session \u66ab\u6642\u4fee\u6539 .claude/settings.local.json \u79fb\u9664\u6b64 hook\uff09\n")
    sys.exit(2)


raw = os.environ.get("HOOK_INPUT_JSON", "")
if not raw:
    sys.exit(0)

try:
    data = json.loads(raw)
except Exception:
    sys.exit(0)

cmd = (data.get("tool_input") or {}).get("command", "")
if not cmd:
    sys.exit(0)

try:
    tokens = shlex.split(cmd, posix=True)
except ValueError:
    # 引號不平衡（可能斷開的 heredoc 或多行指令）→ 放行
    sys.exit(0)

if not tokens:
    sys.exit(0)

for label, pattern in DANGEROUS_SEQUENCES:
    if has_sequence(tokens, pattern):
        block(label, cmd)

if check_rm_rf(tokens):
    block("rm -rf (non-tmp target)", cmd)

sql_label = check_sql_danger(tokens)
if sql_label:
    block(sql_label, cmd)

sys.exit(0)
PYEOF

# Python 的 exit code 會透到這裡（subshell 會保留）
exit $?
