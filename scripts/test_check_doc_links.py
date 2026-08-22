#!/usr/bin/env python3
"""`check-doc-links.py` 的回歸測試。純 stdlib，不連網，CI 與本機同一支。

為什麼要有這支：守門的價值等於「該擋的有沒有擋住」，而 2026-08-20 三輪審查
每一輪都在**已經宣稱測過**的地方找到漏檢。前三輪的矩陣都只跑在暫存目錄、
沒有一行進 repo——別人無法重跑，等於那些驗證不存在。這支把它們釘住。

每個案例都對應一個真實踩過的坑，動到 `check-doc-links.py` 時整組要全綠。

    python3 scripts/test_check_doc_links.py
"""

import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
GUARD = os.path.join(HERE, "check-doc-links.py")

# (檔名, 內容, 是否該被擋, 這條在測什麼)
# 檔案放在 docs/sub/ 底下，讓相對路徑有東西可以爬。
BLOCK = [
    ("badge.md", "[![badge](img/b.png)](docs/security/secret.md)\n",
     "badge 圖片包連結：正則貪到內層就收工，外層目標永遠看不到"),
    ("bracket_label.md", "[see [1] here](docs/security/secret.md)\n",
     "標籤內含 ]，CommonMark 允許但 [^\\]]* 不允許"),
    ("angle_space.md", "[plan](<docs/runbooks/Secret Plan.md>)\n",
     "角括號目標含空白：LINK_RE 的 [^)\\s]+? 禁空白，整條不匹配"),
    ("title_quote.md", "[dr](docs/runbooks/dr.md 'the runbook')\n",
     "單引號 title：CommonMark 三種分隔，原本只支援雙引號"),
    ("ref_display.md", "[docs/security/secret.md][r1]\n\n[r1]: real.md\n",
     "reference-style 的顯示文字：GitHub 原樣印出完整路徑"),
    ("collapsed_ref.md", "[docs/runbooks/dr.md][]\n\n[docs/runbooks/dr.md]: real.md\n",
     "collapsed reference：label 本身就是渲染出來的文字"),
    ("relative.md", "[中性描述](../../docs/audit/C.md)\n",
     "字面不含排除目錄，但解析後落進去"),
    ("label_span.md", "[`docs/security/B.md`](wrong/B.md)\n",
     "標籤內的 code span 是連結文字，不是示範寫法——不得豁免"),
    ("plain.md", "維運手冊見 docs/deploy/DEPLOYMENT.md 這份。\n",
     "純文字提及，根本不是連結"),
    ("html.md", '<a href="docs/security/secret.md">內部文件</a>\n',
     "HTML 連結，GitHub 照樣渲染"),
    ("encoded.md", "[e](docs%2Frunbooks%2Fdr.md)\n",
     "URL 編碼的斜線"),
    ("backslash.md", "路徑 docs\\security\\x.md 見內部文件。\n",
     "反斜線路徑"),
    ("refdef.md", "[用法][r2]\n\n[r2]: <docs/runbooks/Secret Plan.md>\n",
     "link reference definition 的目標"),
    ("after_fence.md",
     "~~~\n``` 這行在 ~~~ fence 內\n[x](docs/audit/InFence.md)\n~~~\n\n之後的真洩漏 docs/audit/After.md\n",
     "fence 盲目 toggle 會讓其後的真內容被當成在 fence 內（漏檢）"),
    ("after_diff.md",
     "# 貼一段 diff\n\n```diff\n--- a/x.md\n+++ b/x.md\n-old\n+new\n```\n\n[dr](docs/runbooks/dr.md)\n",
     "文件內的 +++ 行讓 added_line_numbers 整批位移，檔尾漏檢"),
    ("nonexistent.md", "[gone](./nope.md)\n",
     "一般斷連結"),
    ("absolute.md", "[abs](/etc/passwd)\n",
     "絕對路徑：os.path.join 丟棄前綴，runner 上該檔真的存在"),
    ("escape.md", "[esc](../../../../etc/hostname)\n",
     "`..` 爬出 repo 根"),
    ("windows.md", "[win](C:/System/x.md)\n",
     "Windows 磁碟機代號：os.path.isabs 在 Linux 上為 False"),
    ("archive.md", "[old](../../docs/archive/compliance-reports/x.md)\n",
     "排除清單裡的巢狀目錄"),
    ("有 空白與中文.md", "[x](docs/runbooks/A.md)\n",
     "含空白與非 ASCII 的檔名：git 會加引號跳脫，.split() 會切碎並靜默跳過"),
]

PASS = [
    ("ext.md", "[ext](https://example.com/a.md)\n", "外部連結"),
    ("mail.md", "[mail](mailto:a@b.c)\n", "mailto"),
    ("anchor.md", "[anchor](#x)\n", "純錨點"),
    ("uses_real.md", "[real](real.md)\n", "指向真實檔案"),
    ("span.md", "`[demo](docs/security/Z.md)`\n", "整條包在反引號內＝示範寫法"),
    ("span_label.md", "[`docs/x.md`](real.md)\n", "標籤有 code span 但沒洩漏"),
    ("public_sub.md", "[pub](../../frontend/docs/audit/public.md)\n",
     "公開子路徑：排除比對要錨定 repo 根，不能無錨點子字串比對"),
    ("not_refdef.md", "說明如下：\n\n[Warning]: do not ship\n",
     "像 reference definition 的普通句子"),
    ("not_refdef_cjk.md", "紀錄：\n\n[2026-08-20]: 調整了設定與說明\n",
     "中文版的同型誤報"),
    ("list_fence.md",
     "1. 這樣寫：\n\n    ```markdown\n    [DR](docs/runbooks/dr.md)\n    ```\n\n2. 完成。\n",
     "list item 內縮排 4 空白的 fence 仍是合法 fence"),
    ("quote_fence.md",
     "> 範例：\n>\n> ```markdown\n> [dr](docs/runbooks/dr.md)\n> ```\n",
     "blockquote 內的 fence"),
    ("tilde_fence.md",
     "~~~\n``` 這是 ~~~ 內的內容\n[x](docs/audit/X.md)\n~~~\n",
     "~~~ fence 內出現 ``` 不該提早關閉"),
]


def git(cwd, *args):
    return subprocess.run(
        ["git", "-c", "user.email=t@t", "-c", "user.name=t"] + list(args),
        cwd=cwd, capture_output=True, check=True,
    )


def run_guard(cwd, *args):
    out = subprocess.run(
        [sys.executable, GUARD] + list(args),
        cwd=cwd, capture_output=True, text=True,
    )
    return out.returncode, out.stdout + out.stderr


def build_repo(root):
    """建一個含公開子路徑陷阱與所有案例的暫存 repo，回傳 base commit。"""
    os.makedirs(os.path.join(root, "docs", "sub"))
    os.makedirs(os.path.join(root, "frontend", "docs", "audit"))
    git(root, "init", "-q", ".")
    for rel, body in [
        ("docs/sub/real.md", "# real\n"),
        ("frontend/docs/audit/public.md", "# 這是公開的\n"),
    ]:
        with open(os.path.join(root, rel), "w", encoding="utf-8") as fh:
            fh.write(body)
    git(root, "add", "-A")
    git(root, "commit", "-qm", "base")

    for name, body, _ in BLOCK + PASS:
        with open(os.path.join(root, "docs", "sub", name), "w", encoding="utf-8") as fh:
            fh.write(body)
    git(root, "add", "-A")
    git(root, "commit", "-qm", "cases")
    return "HEAD~1"


def reported_files(output):
    """從輸出裡挑出被報的檔名集合。"""
    names = set()
    for line in output.split("\n"):
        if "::error file=" in line:
            rest = line.split("::error file=", 1)[1]
            names.add(os.path.basename(rest.split(",")[0]))
    return names


def main():
    """跑完整組，回 0 全綠、1 有失敗。"""
    failures = []
    root = tempfile.mkdtemp(prefix="doclinks-")
    try:
        base = build_repo(root)

        rc_diff, out_diff = run_guard(root, base)
        rc_all, out_all = run_guard(root, "--all")
        got_diff = reported_files(out_diff)
        got_all = reported_files(out_all)

        for fname, _, why in BLOCK:
            if fname not in got_diff:
                failures.append(f"漏檢（diff 模式）：{fname} —— {why}")
            if fname not in got_all:
                failures.append(f"漏檢（--all 模式）：{fname} —— {why}")

        for fname, _, why in PASS:
            if fname in got_diff:
                failures.append(f"誤報（diff 模式）：{fname} —— {why}")
            if fname in got_all:
                failures.append(f"誤報（--all 模式）：{fname} —— {why}")

        if rc_diff != 1:
            failures.append(f"有發現時 exit code 應為 1，實得 {rc_diff}")
        if rc_all != 1:
            failures.append(f"--all 有發現時 exit code 應為 1，實得 {rc_all}")

        # 兩模式對同一份內容的判定必須一致（曾因行號位移而相反）
        if got_diff != got_all:
            only_d = sorted(got_diff - got_all)
            only_a = sorted(got_all - got_diff)
            failures.append(f"兩模式判定不一致：僅 diff={only_d}、僅 all={only_a}")

        # base 取不到 → fail-closed，不可靜默綠燈
        rc_bad, out_bad = run_guard(root, "no-such-ref")
        if rc_bad != 3:
            failures.append(f"base 取不到時應 fail-closed 回 3，實得 {rc_bad}：{out_bad[:200]}")

        # 用法錯誤
        rc_usage, _ = run_guard(root)
        if rc_usage != 2:
            failures.append(f"無參數時應回 2，實得 {rc_usage}")

        # 乾淨的樹要能過（正向對照：不是「什麼都擋」才全綠）
        clean = tempfile.mkdtemp(prefix="doclinks-clean-")
        try:
            os.makedirs(os.path.join(clean, "docs"))
            git(clean, "init", "-q", ".")
            with open(os.path.join(clean, "docs", "a.md"), "w", encoding="utf-8") as fh:
                fh.write("# a\n")
            git(clean, "add", "-A")
            git(clean, "commit", "-qm", "base")
            with open(os.path.join(clean, "docs", "b.md"), "w", encoding="utf-8") as fh:
                fh.write("[a](a.md) 與 [外部](https://example.com)\n")
            git(clean, "add", "-A")
            git(clean, "commit", "-qm", "add")
            rc_clean, out_clean = run_guard(clean, "HEAD~1")
            if rc_clean != 0:
                failures.append(f"乾淨的新增內容不該被擋，實得 {rc_clean}：{out_clean[:300]}")
        finally:
            shutil.rmtree(clean, ignore_errors=True)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    total = len(BLOCK) + len(PASS)
    if failures:
        print(f"❌ 回歸測試失敗 {len(failures)} 項（案例共 {total} 型）：")
        for f in failures:
            print(f"   - {f}")
        return 1
    print(f"✅ 回歸測試全綠：該擋 {len(BLOCK)} 型全擋、不該擋 {len(PASS)} 型零誤報、"
          f"兩模式一致、exit code 契約正確")
    return 0


if __name__ == "__main__":
    sys.exit(main())
