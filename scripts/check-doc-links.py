#!/usr/bin/env python3
"""文件連結守門（CI guards）。

檢兩件事，只看 PR 新增的行，不管既有債：

1. **洩漏防護**——新增的行不得把「不隨公開 repo 發布的目錄」印出來。
   判定用**純字串掃描**，**不解析 markdown**（見下面「為什麼不解析」），
   外加一條補強：解析得出的連結目標若**解析後**落在排除目錄，同樣算。

2. **斷連結防護**——新增的 markdown 連結，目標必須是 repo 內存在的檔案。
   這半仍用 regex 解析連結；漏掉一條只是「連結壞了沒被擋」，不是洩漏。

## 為什麼洩漏那半不解析 markdown（2026-08-20 架構裁定）

初版用一條 `LINK_RE` 解析連結、再比對三個維度。三輪審查下來，每一輪都
再找到新的繞過寫法——badge 圖片包連結 `[![x](i.png)](docs/security/a.md)`、
標籤內含 `]`、角括號目標含空白、單引號 / 括號 title、reference-style 的
**顯示文字**、collapsed reference、HTML `<a href>`、`%2F` 編碼路徑……
CommonMark 的連結語法不是 regex 能窮舉的，補洞永遠落後一步。

洩漏這件事**根本不需要知道它是不是連結**：只要排除目錄的路徑字樣出現在
檔案裡，GitHub 就會把它印出來。所以改成掃字串，上述整類繞過一次全消。

代價是誤報面改由「fence / code span 豁免」與「路徑起點錨定」承擔，這兩者
的規則單純、可窮舉，比追 CommonMark 邊角穩得多。

用法：
    check-doc-links.py <base-ref>      # CI：只檢 base...HEAD 新增的行
    check-doc-links.py --all           # 本機：全掃（含既有債；有發現一樣回 exit 1）
"""

import os
import re
import subprocess
import sys

# .gitignore 排除、不隨公開 repo 發布的目錄（來源：.gitignore「新 repo 排除清單」）。
# 語意是**自 repo 根目錄起算**——`frontend/docs/audit/` 這種同名子路徑不算命中。
EXCLUDED_DIRS = (
    "docs/security/",
    "docs/runbooks/",
    "docs/audit/",
    "docs/deploy/",
    # 2026-08-20：原為 docs/archive/compliance-reports/，使用者裁定整個 archive/ 都不發布。
    # 這條與上面四條的理由不同——archive 不是攻擊面情報，只是不再隨公開 repo 出貨；
    # 但守門的處置一樣：新連結不得指過去（在公開 checkout 上必然是斷的）。
    "docs/archive/",
)

# 路徑起點錨定：前面不能接路徑字元，否則 `frontend/docs/audit/` 會被誤判。
# 同時吃 URL 編碼的斜線（`docs%2Fsecurity%2F`），偏移量不變才能沿用 code span 區間。
_ANCHOR = r"(?<![A-Za-z0-9_.\-/])"
EXCLUDED_RES = tuple(
    (d, re.compile(_ANCHOR + re.escape(d).replace("/", "(?:/|%2[Ff])")))
    for d in EXCLUDED_DIRS
)

LINK_RE = re.compile(r"\[([^\]]*)\]\(([^)\s]+?)(?:\s+\"[^\"]*\")?\)")
EXTERNAL_RE = re.compile(r"^(https?:|mailto:|tel:|#)")
# markdown 允許 [label](<path with spaces.md>)。角括號**不代表外部連結**。
ANGLE_RE = re.compile(r"^<(.*)>$")
# CommonMark fence：連續 ≥3 個 ` 或 ~。**不限縮排**——list item 內縮 4 空白的 fence
# 仍是合法 fence，寫死 `^ {0,3}` 會把列表裡的範例整段當成真內容（實測誤報）。
FENCE_RE = re.compile(r"^(\s*)(`{3,}|~{3,})")
# blockquote 內的 fence（`> ```markdown`）同樣要認得，先剝引用標記。
QUOTE_RE = re.compile(r"^\s*(?:>\s?)+")
# git diff -U0 的 hunk header：@@ -a,b +c,d @@ → 新檔自第 c 行起
HUNK_RE = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")
# 行內 code span：`...` / ``...``
INLINE_CODE_RE = re.compile(r"(`+)(?:(?!\1).)*?\1")


class GitError(RuntimeError):
    """git 指令非 0 結束。**不可以吞掉**——見 run_git()。"""


def run_git(args):
    """跑 git 並在非 0 時丟例外。

    早期版本三處 `subprocess.run(capture_output=True)` 都不看 returncode：
    base ref 取不到時 git 回 128、stdout 空，腳本就當成「沒有改動的檔案」
    印綠燈退 0——**靜默假綠**，守門形同不存在而沒人會發現。
    （本 repo 的 `guards` job 有 `fetch-depth: 0`，現行設定下取得到 base；
    但這種失效模式不能靠別處的設定來保證，故在此 fail-closed。）
    """
    out = subprocess.run(["git"] + args, capture_output=True)
    if out.returncode != 0:
        raise GitError(
            f"git {' '.join(args)} 失敗（exit {out.returncode}）："
            f"{out.stderr.decode(errors='replace').strip()}"
        )
    return out.stdout


def code_span_ranges(line):
    """回傳該行所有行內 code span 的 (起, 迄) 區間。"""
    return [(m.start(), m.end()) for m in INLINE_CODE_RE.finditer(line)]


def span_is_link_label(line, s, e):
    """該 code span 是不是 markdown 連結的**標籤**（`` [`docs/x.md`](y.md) ``）。

    是的話 GitHub 會把它渲染成連結文字、完整檔名照樣印出來——那是洩漏，
    不是「展示的寫法」，不得豁免。這條分界兩邊都踩過，別再合併掉。
    """
    return s > 0 and line[s - 1] == "[" and line[e:e + 2] == "]("


def leak_hits(line):
    """純字串掃描：這一行有沒有把排除目錄印出來。

    回傳命中的目錄清單（去重、保序）。整條 code span 內的命中視為文件範例
    而豁免，但**當該 code span 是連結標籤時不豁免**（見 span_is_link_label）。
    """
    # 反斜線路徑（`docs\security\x.md`）正規化；1:1 取代，偏移量不變。
    norm = line.replace("\\", "/")
    spans = code_span_ranges(norm)
    found = []
    for d, rx in EXCLUDED_RES:
        for m in rx.finditer(norm):
            exempt = False
            for s, e in spans:
                if s <= m.start() and m.end() <= e:
                    exempt = not span_is_link_label(norm, s, e)
                    break
            if not exempt and d not in found:
                found.append(d)
                break
    return found


def iter_lines(text):
    """吐出 (行號, 行內容)，跳過 code fence 內的內容。

    fence 要跳：不跳的話「建議貼進某文件的 markdown 範例」會被當成真內容。
    狀態機記 (字元, 長度)：盲目 toggle 會在 `~~~` fence 內遇到 ``` 時提早關閉，
    實測**同時**造成誤報（fence 內的範例被當真的）與漏檢（其後的真內容被當成
    在 fence 內）。
    """
    fence = None  # (字元, 長度)；None = 不在 fence 內
    for lineno, raw in enumerate(text.split("\n"), 1):
        line = QUOTE_RE.sub("", raw)  # 先剝 blockquote 標記
        m = FENCE_RE.match(line)
        if m:
            run = m.group(2)
            ch, n = run[0], len(run)
            if fence is None:
                fence = (ch, n)
            elif ch == fence[0] and n >= fence[1] and not line.strip()[n:].strip():
                # 收合條件：同字元、不短於開場、且後面沒有 info string
                fence = None
            # 其餘情況（fence 內出現另一種標記）就是普通內容，照樣跳過
            continue
        if fence is not None:
            continue
        yield lineno, line


def iter_links(line):
    """吐出該行的 (顯示文字, 目標)，跳過整條包在 code span 內的示範寫法。

    只服務**斷連結**那半。洩漏改由 leak_hits() 掃字串，不倚賴這裡解析得全。
    """
    spans = code_span_ranges(line)
    for m in LINK_RE.finditer(line):
        if any(s <= m.start() and m.end() <= e for s, e in spans):
            continue
        yield m.group(1), m.group(2)


class _Outside:
    """哨兵：目標解析後落在 repo 根目錄之外。"""

    def __repr__(self):
        return "<repo 根目錄之外>"


OUTSIDE = _Outside()


def resolve(md_path, target):
    """把連結目標解析成 repo 相對路徑。

    三種回傳值刻意分開，不共用 None：
    - `None`：純錨點（`#x`），根本不是路徑，不該當斷連結。
    - `OUTSIDE`：絕對路徑或 `..` 爬出 repo 根。`os.path.join()` 碰到絕對路徑會
      **丟棄前綴**，於是 `[x](/etc/passwd)` 在 Ubuntu runner 上因該檔真的存在而
      放行——但它不是 repo 內可用的文件連結。一律當斷連結。
    - `str`：repo 根目錄起算的相對路徑。
    """
    tgt = target.split("#")[0]
    if not tgt:
        return None
    # Windows 磁碟機代號（`C:\...`）在 Linux 上 os.path.isabs() 為 False，另外判。
    if os.path.isabs(tgt) or (len(tgt) > 1 and tgt[1] == ":"):
        return OUTSIDE
    # 一律用 posix 語意：本機在 Windows 跑時 os.path.normpath 會回反斜線，
    # 使下面的 "../" 判斷失效，出界防線等於沒有。
    r = os.path.normpath(os.path.join(os.path.dirname(md_path), tgt)).replace("\\", "/")
    if r == ".." or r.startswith("../"):
        return OUTSIDE
    return r


def resolved_hits_excluded(resolved):
    """解析後路徑是否落在排除目錄（自 repo 根錨定）。

    補字串掃描抓不到的那型：`[DR](../runbooks/X.md)` 字面不含 `docs/runbooks/`，
    但解析後正好落進去。
    """
    if not resolved or resolved is OUTSIDE:
        return None
    norm = resolved.replace("\\", "/")
    for d in EXCLUDED_DIRS:
        if norm == d.rstrip("/") or norm.startswith(d):
            return d
    return None


def changed_md_files(base):
    """本 PR 改動到的 .md 檔（排除已刪除者）。

    必須用 `-z`：git 預設會把含空白或非 ASCII 的檔名加引號跳脫輸出，
    用 `.split()` 切會把它們切碎，接著 `os.path.isfile()` 為 False，
    該檔被**靜默跳過**——守門對它完全失效。本 repo 就有含空白與中文的 .md。
    """
    out = run_git(["diff", "--name-only", "-z", "--diff-filter=d", f"{base}...HEAD"])
    return [f.decode() for f in out.split(b"\0") if f.endswith(b".md")]


def added_line_numbers(base, path):
    """該檔在此 PR 中新增的行，**於新檔中的行號**。

    比對行**內容**會出兩種錯（既有斷連結若與新增行內容相同會被誤判成新增、
    重複行判定不穩），所以解析 `git diff -U0` 的 hunk header 取真實行號。

    ⚠️ 不可以特判 `line.startswith("+++")`：`+++ b/x.md` 這個檔頭只出現在第一個
    `@@` 之前（那時 cur 還是 None，已被擋掉）；反而是**文件裡貼了一段 diff 範例**
    時，內容行 `+++ b/x.md` 在 diff 中變成 `++++ b/x.md`，被特判吃掉又不遞增 cur，
    其後所有行號整批少 1、檔尾數行永遠檢查不到（2026-08-20 實測漏掉真洩漏）。
    """
    out = run_git(["diff", "-U0", f"{base}...HEAD", "--", path]).decode(errors="replace")
    nums, cur = set(), None
    for line in out.split("\n"):
        m = HUNK_RE.match(line)
        if m:
            cur = int(m.group(1))
            continue
        if cur is None:
            continue
        if line.startswith("+"):
            nums.add(cur)
            cur += 1
        # -U0 無 context 行；'-' 行不佔新檔行號
    return nums


def scan(files, base):
    """回傳 (leaked, broken)。base 為 None 時全掃，否則只看新增的行。"""
    leaked, broken = [], []
    for f in files:
        if not os.path.isfile(f):
            continue
        text = open(f, encoding="utf-8", errors="replace").read()
        new_nums = None if base is None else added_line_numbers(base, f)

        for lineno, line in iter_lines(text):
            if new_nums is not None and lineno not in new_nums:
                continue

            hit = leak_hits(line)
            if hit:
                leaked.append((f, lineno, line.strip()[:120], hit[0]))
                continue

            for label, target in iter_links(line):
                m = ANGLE_RE.match(target)
                if m:
                    target = m.group(1)
                if EXTERNAL_RE.match(target):
                    continue
                resolved = resolve(f, target)
                d = resolved_hits_excluded(resolved)
                if d:
                    leaked.append((f, lineno, f"[{label}]({target})", d))
                    continue
                if resolved is OUTSIDE:
                    broken.append((f, lineno, target, "解析後落在 repo 根目錄之外"))
                    continue
                if resolved and not os.path.exists(resolved):
                    broken.append((f, lineno, target, f"解析為 {resolved}（不存在）"))
    return leaked, broken


def main():
    """入口：回 0 通過、1 有發現、2 用法錯誤、3 git 失敗。"""
    if len(sys.argv) < 2:
        print("usage: check-doc-links.py <base-ref> | --all", file=sys.stderr)
        return 2

    scan_all = sys.argv[1] == "--all"
    base = None if scan_all else sys.argv[1]

    try:
        if scan_all:
            files = [x.decode() for x in run_git(["ls-files", "-z", "*.md"]).split(b"\0") if x]
        else:
            files = changed_md_files(base)
        leaked, broken = scan(files, base)
    except GitError as e:
        # fail-closed：寧可紅一次讓人來看，也不要靜默放行
        print(f"::error::{e}", file=sys.stderr)
        return 3

    if leaked:
        print("::error::新增的內容印出了不隨公開 repo 發布的目錄：")
        for f, ln, snippet, d in leaked:
            print(f"::error file={f},line={ln}::{snippet} → 命中排除目錄 {d}")
        print("::error::這些目錄不隨公開 repo 發布（見 .gitignore 排除清單）。")
        print("::error::安全 / 稽核 / 部署類：改成不具名的中性描述，例如「內部災難復原 runbook」，")
        print("::error::不要寫出檔名或路徑。archive 類：改引用仍在本 repo 的現用文件。")
        print("::error::若這行只是文件裡的範例，請包進 code fence 或整條包進反引號。")

    if broken:
        print("::error::新增的連結指向不存在的檔案、或 repo 根目錄之外：")
        for f, ln, target, why in broken:
            print(f"::error file={f},line={ln}::{target} → {why}")

    if leaked or broken:
        return 1

    scope = "全 repo" if scan_all else "本 PR 新增的行"
    print(f"✅ 文件連結檢查通過（範圍：{scope}，掃了 {len(files)} 個 .md）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
