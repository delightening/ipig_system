"""抽取 PDF 版面結構：每頁字型使用、字級分佈、表格偵測。"""
import sys
from collections import Counter

import fitz


def main(path):
    doc = fitz.open(path)
    print(f"pages={doc.page_count}")
    page0 = doc[0]
    w, h = page0.rect.width, page0.rect.height
    print(f"page_size_pt={w}x{h}  page_size_cm={w/72*2.54:.2f}x{h/72*2.54:.2f}\n")

    global_font_counter = Counter()
    global_size_counter = Counter()

    for pno, page in enumerate(doc):
        print(f"{'='*70}")
        print(f"PAGE {pno+1}")
        print(f"{'='*70}")
        d = page.get_text("dict")
        page_fonts = Counter()
        page_sizes = Counter()
        for block in d["blocks"]:
            if block["type"] != 0:
                continue
            for line in block["lines"]:
                for span in line["spans"]:
                    font = span["font"]
                    size = round(span["size"], 1)
                    txt = span["text"]
                    if txt.strip():
                        page_fonts[font] += len(txt)
                        page_sizes[size] += len(txt)
                        global_font_counter[font] += len(txt)
                        global_size_counter[size] += len(txt)

        print(f"Fonts: {dict(page_fonts.most_common())}")
        print(f"Sizes(pt): {dict(page_sizes.most_common())}")

        # Extract text preview (first 500 chars)
        text = page.get_text("text")
        print(f"\n--- Text Preview (first 600 chars) ---")
        print(text[:600])

        # Detect tables
        try:
            tables = page.find_tables()
            if tables.tables:
                print(f"\n--- Tables on this page: {len(tables.tables)} ---")
                for ti, tbl in enumerate(tables.tables):
                    print(f"  Table {ti}: rows={tbl.row_count}  cols={tbl.col_count}  bbox={tbl.bbox}")
        except Exception as e:
            print(f"  (table detection skipped: {e})")

        print()

    print(f"\n{'='*70}")
    print(f"GLOBAL SUMMARY")
    print(f"{'='*70}")
    print(f"Fonts used (chars total):")
    for font, cnt in global_font_counter.most_common():
        print(f"  {font:30s} {cnt}")
    print(f"\nFont sizes used (chars total):")
    for sz, cnt in global_size_counter.most_common():
        print(f"  {sz}pt   {cnt}")

    doc.close()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: python {sys.argv[0]} <path_to_pdf>", file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1])
