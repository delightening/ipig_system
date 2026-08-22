"""Visual audit: rasterize reference + rendered PDF pages side-by-side, emit HTML.

Reference PDFs in templates/reference/ are scanned (image-only) → text-diff
useless. This tool renders both sides to PNGs and builds a single HTML page
the user can open to do final eyeball comparison.

Usage:
    1. Re-run smoke_test.py to refresh _smoke_out/*.pdf
    2. python services/print-pdf/_tools/visual_audit.py
    3. Open _audit_out/index.html in a browser
"""

import sys
from pathlib import Path

import fitz  # PyMuPDF

ROOT = Path(__file__).resolve().parents[3]
OUT_DIR = Path(__file__).parent / "_audit_out"
RENDERED_DIR = Path(__file__).parent / "_smoke_out"

PAIRS = [
    ("review_result", "審核結果範例.pdf", "AD-04-01-10B 審核結果"),
    ("review_reply", "審查意見回覆表範例.pdf", "審查意見回覆表"),
    ("medical_record", "實驗豬隻病歷總表範例.pdf", "實驗豬隻病歷總表"),
    ("aup_protocol", "AD-04-01-01F動物試驗研究計畫書.pdf", "AD-04-01-01F AUP 計畫書"),
]
DPI = 110  # decent for screen, not huge


def rasterize(pdf_path: Path, out_prefix: Path, label: str) -> list[Path]:
    """Render each page of pdf_path to {out_prefix}_p{n}.png, return paths."""
    doc = fitz.open(pdf_path)
    paths: list[Path] = []
    for i, page in enumerate(doc, start=1):
        mat = fitz.Matrix(DPI / 72, DPI / 72)
        pix = page.get_pixmap(matrix=mat, alpha=False)
        out = out_prefix.with_name(f"{out_prefix.name}_p{i}.png")
        pix.save(out)
        paths.append(out)
    doc.close()
    print(f"    {label}: {len(paths)} page(s)")
    return paths


def build_html(report: list[dict]) -> str:
    rows = []
    for r in report:
        page_blocks = []
        max_pages = max(len(r["ref"]), len(r["ren"]))
        for i in range(max_pages):
            ref_img = (
                f'<img src="{r["ref"][i].name}" alt="ref p{i+1}">'
                if i < len(r["ref"])
                else '<div class="missing">(no reference page)</div>'
            )
            ren_img = (
                f'<img src="{r["ren"][i].name}" alt="ren p{i+1}">'
                if i < len(r["ren"])
                else '<div class="missing">(no rendered page)</div>'
            )
            page_blocks.append(
                f"""<div class="page-row">
                    <div class="page-label">p{i+1}</div>
                    <div class="ref">{ref_img}</div>
                    <div class="ren">{ren_img}</div>
                </div>"""
            )
        rows.append(
            f"""<section class="template">
                <h2>{r['title']} <small>(rendered {r['id']}.pdf)</small></h2>
                <div class="meta">reference {len(r['ref'])} page(s) | rendered {len(r['ren'])} page(s)</div>
                {''.join(page_blocks)}
            </section>"""
        )
    return f"""<!doctype html>
<html lang="zh-TW"><head><meta charset="utf-8">
<title>print-pdf visual audit</title>
<style>
  body {{ font-family: -apple-system, "Segoe UI", system-ui, sans-serif; margin: 24px; background: #f7f7f7; }}
  h1 {{ margin: 0 0 8px 0; }}
  .intro {{ color: #555; margin: 0 0 24px 0; max-width: 720px; }}
  section.template {{ background: #fff; border: 1px solid #ddd; border-radius: 6px; padding: 18px 22px; margin: 0 0 28px 0; }}
  h2 {{ margin: 0; font-size: 18pt; }}
  h2 small {{ font-size: 10pt; color: #888; font-weight: normal; margin-left: 8px; }}
  .meta {{ color: #777; font-size: 10pt; margin: 4px 0 14px 0; }}
  .page-row {{ display: grid; grid-template-columns: 40px 1fr 1fr; gap: 12px; align-items: start; padding: 12px 0; border-top: 1px dashed #ddd; }}
  .page-row:first-of-type {{ border-top: 0; }}
  .page-label {{ font-weight: bold; color: #555; padding-top: 6px; }}
  .ref, .ren {{ background: #fafafa; padding: 6px; text-align: center; }}
  .ref::before {{ content: "REFERENCE"; display: block; font-size: 9pt; color: #c33; margin-bottom: 4px; }}
  .ren::before {{ content: "RENDERED"; display: block; font-size: 9pt; color: #393; margin-bottom: 4px; }}
  img {{ max-width: 100%; border: 1px solid #ccc; }}
  .missing {{ color: #aaa; padding: 40px; font-style: italic; }}
</style></head>
<body>
<h1>print-pdf visual audit</h1>
<p class="intro">並排比對 templates/reference/ 的範例 PDF（左）vs 目前 print-pdf 輸出（右）。
sample 資料和 reference 的內容不同（reference 是實際表單，sample 是 demo data），
重點看：標題 / 欄位排版 / 表格結構 / 簽章區塊 / 頁碼樣式 是否對齊。</p>
{''.join(rows)}
</body></html>"""


def main() -> int:
    OUT_DIR.mkdir(exist_ok=True)
    report: list[dict] = []
    for tid, ref_name, title in PAIRS:
        ref_path = ROOT / "templates" / "reference" / ref_name
        rendered_path = RENDERED_DIR / f"{tid}.pdf"
        if not ref_path.exists():
            print(f"[skip] {tid}: ref missing — {ref_path}")
            continue
        if not rendered_path.exists():
            print(f"[skip] {tid}: rendered missing — {rendered_path}  (run smoke_test.py first)")
            continue
        print(f"[run] {tid}")
        ref_pngs = rasterize(ref_path, OUT_DIR / f"{tid}_ref", "reference")
        ren_pngs = rasterize(rendered_path, OUT_DIR / f"{tid}_ren", "rendered")
        report.append({"id": tid, "title": title, "ref": ref_pngs, "ren": ren_pngs})

    if not report:
        print("nothing to render")
        return 1
    html = build_html(report)
    out_html = OUT_DIR / "index.html"
    out_html.write_text(html, encoding="utf-8")
    print(f"\nopen: {out_html.as_uri()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
