"""Visual-audit helper: extract text from reference vs rendered PDFs,
print side-by-side outline to help identify layout drift.

Usage: python services/print-pdf/_tools/compare_pdf.py
Writes UTF-8 output to _tools/_compare_out/<id>.txt
"""

import sys
from pathlib import Path

from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[3]
PAIRS = [
    ("review_result", "templates/reference/審核結果範例.pdf"),
    ("review_reply", "templates/reference/審查意見回覆表範例.pdf"),
    ("medical_record", "templates/reference/實驗豬隻病歷總表範例.pdf"),
    ("aup_protocol", "templates/reference/AD-04-01-01F動物試驗研究計畫書.pdf"),
]
OUT_DIR = Path(__file__).parent / "_compare_out"
RENDERED_DIR = Path(__file__).parent / "_smoke_out"


def extract(p: Path) -> tuple[int, list[str]]:
    r = PdfReader(p)
    pages = [(page.extract_text() or "").strip() for page in r.pages]
    return len(r.pages), pages


def main() -> int:
    OUT_DIR.mkdir(exist_ok=True)
    for tid, ref_rel in PAIRS:
        ref = ROOT / ref_rel
        rendered = RENDERED_DIR / f"{tid}.pdf"
        if not ref.exists():
            print(f"[skip] {tid}: ref missing {ref}")
            continue
        if not rendered.exists():
            print(f"[skip] {tid}: rendered missing {rendered}")
            continue
        ref_n, ref_pages = extract(ref)
        ren_n, ren_pages = extract(rendered)
        lines: list[str] = []
        lines.append(f"=== {tid} ===")
        lines.append(f"reference : {ref.name}  ({ref_n} pages)")
        lines.append(f"rendered  : {rendered.name}  ({ren_n} pages)")
        lines.append("")
        max_n = max(ref_n, ren_n)
        for i in range(max_n):
            lines.append(f"--- page {i + 1} REFERENCE ---")
            lines.append(ref_pages[i] if i < ref_n else "(no page)")
            lines.append("")
            lines.append(f"--- page {i + 1} RENDERED ---")
            lines.append(ren_pages[i] if i < ren_n else "(no page)")
            lines.append("")
        out_path = OUT_DIR / f"{tid}.txt"
        out_path.write_text("\n".join(lines), encoding="utf-8")
        print(f"[ok] {tid}: ref {ref_n}p  vs  rendered {ren_n}p  →  {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
