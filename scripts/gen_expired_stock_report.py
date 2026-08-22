#!/usr/bin/env python3
"""把過期庫存的 TSV 匯出轉成可列印的盤點清單 HTML。

用法：
    docker exec ipig-db psql -U postgres -d ipig_db -A -F '\t' -c "<query>" > expired.tsv
    python scripts/gen_expired_stock_report.py expired.tsv expired_stock.html

刻意做成獨立腳本而非一次性 inline：過期庫存是會反覆盤的東西，
下次只要重跑同一支就好。查詢本身寫在 docs/design/erp-compliance-r97/PLAN.md。
"""
import csv
import datetime
import decimal
import html
import io
import os
import sys

CSS = """
body{font-family:"Noto Sans TC","Microsoft JhengHei",sans-serif;margin:2rem auto;
     max-width:1100px;color:#1a1a1a;line-height:1.5}
h1{font-size:1.5rem;margin-bottom:.2rem}
h2{font-size:1.15rem;margin-top:2rem;border-bottom:2px solid #333;padding-bottom:.3rem}
.n{font-weight:400;font-size:.9rem;color:#666}
.note{color:#555;font-size:.9rem;margin:.4rem 0 .8rem}
.meta{color:#666;font-size:.85rem;margin-bottom:1.5rem}
table{border-collapse:collapse;width:100%;font-size:.88rem}
th,td{border:1px solid #ddd;padding:.4rem .55rem;text-align:left}
th{background:#f4f4f4;font-weight:600}
.r{text-align:right}
.sku{font-family:ui-monospace,monospace;font-size:.82rem;color:#555}
tr.warn{background:#fffbe6}tr.bad{background:#fff1e6}tr.vbad{background:#ffe9e9}
.susp{background:#e8e8f5;color:#443;font-size:.72rem;padding:.05rem .35rem;
      border-radius:3px;white-space:nowrap}
.lead{background:#fff4f4;border-left:4px solid #c33;padding:.8rem 1rem;margin:1rem 0}
@media print{body{margin:0;max-width:none}
  tr.warn,tr.bad,tr.vbad{background:#fff !important;-webkit-print-color-adjust:exact}}
"""


def qty(v):
    """數量一律走 Decimal，不經 float。

    這是要拿去現場點實物的清單：float 會把 `2000.0000` 這種
    NUMERIC(18,4) 值印成近似值，盤點時對不上就得重查。
    `normalize()` 去掉尾隨的 0（2000.0000 → 2000），
    指數形式（如 1E+3）再展開回一般寫法。
    """
    d = decimal.Decimal(v).normalize()
    return format(d, "f")


def section(title, rows, note):
    if not rows:
        return ""
    total = sum((decimal.Decimal(r["qty"]) for r in rows), decimal.Decimal(0))
    out = [
        '<h2>{} <span class="n">{} 筆 / {} 單位</span></h2>'.format(
            html.escape(title), len(rows), qty(total)
        ),
        '<p class="note">{}</p>'.format(note),
        "<table><thead><tr><th>✔</th><th>倉庫</th><th>儲位</th><th>品項</th>"
        "<th>SKU</th><th>批號</th><th>效期</th><th>過期</th>"
        '<th class="r">數量</th></tr></thead><tbody>',
    ]
    for r in rows:
        days = int(r["days_expired"])
        cls = "vbad" if days > 365 else ("bad" if days > 30 else "warn")
        # 效期早於 2015 年幾乎都是建檔時把批號／製造日誤填成效期。
        suspect = (
            ' <span class="susp">效期存疑</span>' if int(r["expiry_date"][:4]) < 2015 else ""
        )
        out.append(
            '<tr class="{}"><td><input type="checkbox"></td><td>{}</td>'
            "<td><b>{}</b></td><td>{}{}</td>"
            '<td class="sku">{}</td><td class="sku">{}</td><td>{}</td>'
            '<td class="r">{} 天</td><td class="r"><b>{}</b> {}</td></tr>'.format(
                cls,
                html.escape(r["warehouse"]),
                html.escape(r["shelf"]),
                html.escape(r["product"]),
                suspect,
                html.escape(r["sku"]),
                html.escape(r["batch"]),
                # 一併 escape：雖然 expiry_date 來自 DATE 欄位、格式受控，
                # 但這裡每個欄位都是外部資料，逐一 escape 才不用逐欄論證安全性。
                html.escape(r["expiry_date"]),
                days,
                qty(r["qty"]),
                html.escape(r["uom"]),
            )
        )
    out.append("</tbody></table>")
    return "\n".join(out)


def main(src, dst):
    with io.open(src, encoding="utf-8") as fh:
        rows = [r for r in csv.DictReader(fh, delimiter="\t") if r.get("sku")]

    drugs = [r for r in rows if r["cat"] == "DRG"]
    others = [r for r in rows if r["cat"] != "DRG"]
    # 專案慣例：時間一律以 GMT+8 呈現（CLAUDE.md）。
    stamp = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).strftime(
        "%Y-%m-%d %H:%M"
    )

    doc = (
        "<!doctype html>\n"
        '<html lang="zh-Hant">\n<meta charset="utf-8">\n'
        "<title>過期庫存盤點清單</title>\n"
        "<style>" + CSS + "</style>\n"
        "<h1>過期庫存盤點清單</h1>\n"
        '<div class="meta">資料來源：prod 資料庫 <code>storage_location_inventory</code>'
        "（僅列 on_hand_qty &gt; 0）｜產出時間 " + stamp + " GMT+8</div>\n"
        '<div class="lead"><b>準備室</b>的品項在領用動線上，建議優先處理。'
        "系統目前<b>不會</b>阻止領用過期批號（R97-3 待修）。</div>\n"
        + section("藥品 DRG（優先）", drugs, "過期藥品不應再用於動物。逐項核對實物後決定報廢或退貨。")
        + "\n"
        + section(
            "其他類別（耗材 CON / 化學品 CHM 等）",
            others,
            "部分效期明顯為建檔錯誤，核對實物後多半可修正效期繼續使用。",
        )
        + '\n<p class="meta">「效期存疑」= 效期早於 2015 年，多半是建檔時把批號或製造日誤填為效期，'
        "實物未必真的過期，建議核對包裝後修正資料而非直接報廢。</p>\n</html>\n"
    )
    with io.open(dst, "w", encoding="utf-8") as fh:
        fh.write(doc)
    print("DRG={} other={} -> {}".format(len(drugs), len(others), dst))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        prog = os.path.basename(sys.argv[0])
        sys.exit("用法：python {} <輸入.tsv> <輸出.html>".format(prog))
    main(sys.argv[1], sys.argv[2])
