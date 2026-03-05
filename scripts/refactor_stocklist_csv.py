# -*- coding: utf-8 -*-
"""
耗材藥品庫存清表重構：品名/規格標準化、針筒/針頭分類、廠商與包裝欄位處理。
輸出欄位：項次, 分類, 品名, 規格, 製造廠商, 單位, 包裝規格
"""
import csv
import re
from pathlib import Path

# 輸入輸出路徑
INPUT_CSV = Path(__file__).resolve().parent.parent / "20260305_TU-04-00-01_耗材藥品庫存清表(年)_V.0315(TEST).csv"
OUTPUT_CSV = Path(__file__).resolve().parent.parent / "20260305_TU-04-00-01_耗材藥品庫存清表(年)_V.0315_REFACTORED.csv"


def normalize_manufacturer(val: str) -> str:
    """廠商：空、"0"、"-" 統一為空。"""
    if not val or not str(val).strip():
        return ""
    v = str(val).strip()
    if v in ("0", "-"):
        return ""
    return v


def extract_packaging(spec: str) -> tuple[str, str]:
    """
    從規格中分離「包裝規格」（箱/盒/支等換算）。
    回傳 (主規格, 包裝規格)。
    """
    if not spec or not spec.strip():
        return ("", "")
    spec = spec.strip()
    # 常見模式：數字+單位/盒、數字+支/包、unit/盒、個/盒 等
    packaging_patterns = [
        r"(\d+\s*(?:支|個|枚|張|條|片|雙|瓶|包|捲|件|組|安瓶)/\s*(?:盒|箱|包|袋)(?:\s*[，,]\s*\d+(?:\s*盒|包)/\s*箱)?)",
        r"(\d+\s*(?:unit|pcs|支|個)/\s*(?:盒|箱|包))",
        r"(\d+\s*(?:盒|包|本)/\s*箱)",
    ]
    pack_out = []
    remaining = spec
    for pat in packaging_patterns:
        for m in re.finditer(pat, remaining, re.IGNORECASE):
            pack_out.append(m.group(1).strip())
        remaining = re.sub(pat, " ", remaining, flags=re.IGNORECASE)
    packaging = "，".join(pack_out).strip() if pack_out else ""
    # 主規格：若整段都是包裝描述則留空；否則保留非包裝部分（容量、尺寸等）
    if packaging and packaging == spec:
        main_spec = ""
    elif packaging:
        # 移除已提取的包裝，剩餘當主規格
        for p in pack_out:
            remaining = remaining.replace(p, " ")
        main_spec = re.sub(r"[,，]\s*$", "", re.sub(r"\s+", " ", remaining)).strip()
        main_spec = main_spec.strip("，, ")
    else:
        main_spec = spec.strip("，, ")
    return (main_spec, packaging)


def normalize_needle_spec(text: str) -> str:
    """將針頭規格統一為 G*長度 格式，例如 23G*1\" """
    if not text:
        return ""
    text = re.sub(r"\s+", " ", text.strip())
    # 保留 G 與 inch 描述，簡化為 數字G*數字 或 數字G*數字 1/2 等
    text = re.sub(r'"', '"', text)  # 統一引號
    return text.strip()


def detect_capacity_in_name(name: str) -> str:
    """從品名提取容量，如 50 ml、50mL、2.5 mL、3 ml(帶針) -> 50ml, 2.5ml, 3ml"""
    m = re.search(r"(\d+\.?\d*)\s*(?:ml|mL|cc)(?![a-zA-Z])", name, re.IGNORECASE)
    if m:
        return m.group(1) + "ml"
    return ""


def detect_needle_in_name_or_spec(name: str, spec: str) -> str:
    """從品名或規格提取針頭型號（G + 長度），用於針筒/針頭規格。保留 1 1/2 等分數格式。"""
    combined = f"{name} {spec}"
    # 排除包裝數字（如 10支/盒）被誤當成 inch：inch 後不可緊接 支/盒/個/
    no_pack = r"(?!支|盒|個|/)"
    # 例如 23G*1", 18G 1 1/2", 21G採血針 1 1/2"（G 與長度間可有中文）
    m = re.search(
        rf"(\d+)\s*G\s*[^\d]*(\d+(?:\s+\d+/\d+)?)\s*\"?{no_pack}",
        combined,
        re.IGNORECASE,
    )
    if m:
        g, inch = m.group(1), m.group(2).strip()
        # 排除從「10支/盒」等包裝數字誤取的 inch（避免 1 來自 10）
        end_pos = m.end(2)
        if end_pos < len(combined) and combined[end_pos].isdigit():
            return f"{g}G"
        if inch and int(inch.split()[0].split("/")[0]) <= 3:  # inch 通常 1~3
            if not inch.endswith('"'):
                inch = inch + '"'
            return f"{g}G*{inch}"
        return f"{g}G"
    m = re.search(rf"(\d+)\s*G\s*(?:\*|\s)\s*(\d+(?:\s+\d+/\d+)?)\s*\"?{no_pack}", combined, re.IGNORECASE)
    if m:
        g, inch = m.group(1), m.group(2).strip()
        if inch:
            if not inch.endswith('"'):
                inch = inch + '"'
            return f"{g}G*{inch}"
        return f"{g}G"
    m = re.search(r"(\d+)\s*G(?![a-zA-Z])", combined, re.IGNORECASE)
    if m:
        return f"{m.group(1)}G"
    return ""


def is_syringe_row(name: str, unit: str, sub_item: str) -> bool:
    """是否為針筒類：細項為針筒/針桶，或品名含針具/針筒/針桶。"""
    name = (name or "").strip()
    sub_item = (sub_item or "").strip()
    if sub_item in ("針筒", "針桶"):
        return True
    if "針具" in name or "針筒" in name or "針桶" in name:
        return True
    return False


def is_needle_only_row(name: str, sub_item: str, spec: str) -> bool:
    """是否為針頭類：僅細項為「針頭」時歸類為針頭（排除縫線、留置針等）。"""
    sub_item = (sub_item or "").strip()
    return sub_item == "針頭"


def needle_spec_to_display(needle: str) -> str:
    """將針頭規格改為顯示格式：26G*1\" → 26G 1\"（空格分隔，符合匯入需求）。"""
    if not needle:
        return ""
    return needle.replace("*", " ").strip()


def capacity_to_cc(capacity_ml: str) -> str:
    """容量改為 cc 顯示（1ml=1cc，僅改單位字樣）。"""
    if not capacity_ml or not capacity_ml.endswith("ml"):
        return capacity_ml or ""
    return capacity_ml[:-2] + "cc"


def format_syringe_spec(name: str, spec: str) -> tuple[str, str]:
    """
    針筒：回傳 (品名, 規格)。
    需求：針筒 3cc 支 / 帶針針筒 3cc 24G 1" 支（品名含「帶針」或規格有針頭時用帶針針筒）
    """
    has_帶針 = "帶針" in (name or "")
    capacity = detect_capacity_in_name(name)
    needle = detect_needle_in_name_or_spec(name, spec)
    capacity_cc = capacity_to_cc(capacity) if capacity else ""
    needle_display = needle_spec_to_display(needle) if needle else ""
    if capacity_cc and needle_display:
        return ("帶針針筒", f"{capacity_cc} {needle_display}")
    if capacity_cc and has_帶針:
        return ("帶針針筒", capacity_cc)
    if capacity_cc:
        return ("針筒", capacity_cc)
    if needle_display:
        return ("針筒", needle_display)
    return ("針筒", (spec or "").strip() or "")


def format_needle_spec(name: str, spec: str) -> str:
    """針頭規格：顯示格式為 26G 1\"（空格分隔，符合匯入需求）。"""
    needle = detect_needle_in_name_or_spec(name, spec)
    if needle:
        return needle_spec_to_display(needle)
    return (spec or "").strip() or ""


def standardize_unit_for_needle_syringe(unit: str) -> str:
    """針/針筒/帶針針筒單位統一為 支。"""
    return "支"


def main():
    with open(INPUT_CSV, "r", encoding="utf-8-sig", newline="") as f:
        reader = csv.reader(f)
        rows = list(reader)

    # 找表頭列（含「品名」）
    header_idx = None
    for i, row in enumerate(rows):
        if len(row) > 4 and "品名" in (row[4] or ""):
            header_idx = i
            break
    if header_idx is None:
        raise SystemExit("找不到品名欄位表頭")

    # 欄位索引：請購=0, 存放區=1, 項目=2, 細項=3, 品名=4, 數量=5, 單位=6, 規格=7, 廠商=8
    col_品名, col_單位, col_規格, col_廠商 = 4, 6, 7, 8
    col_項目, col_細項 = 2, 3

    out_rows = []
    out_rows.append(["項次", "分類", "品名", "規格", "製造廠商", "單位", "包裝規格"])

    idx = 0
    for row in rows[header_idx + 1 :]:
        if len(row) <= col_品名:
            continue
        name = (row[col_品名] or "").strip()
        if not name:
            continue
        idx += 1
        項目 = (row[col_項目] or "").strip()
        細項 = (row[col_細項] or "").strip()
        單位 = (row[col_單位] or "").strip()
        規格原始 = (row[col_規格] or "").strip()
        廠商 = normalize_manufacturer(row[col_廠商] if len(row) > col_廠商 else "")

        # 分類：用細項（針筒、針頭、輸液等）
        分類 = 細項 or 項目 or ""

        # 包裝從原始規格中分離（先做，後面規格可能被覆寫）
        main_spec, 包裝規格 = extract_packaging(規格原始)

        if is_syringe_row(name, 單位, 細項):
            品名, 規格 = format_syringe_spec(name, 規格原始)
            if not 規格 and main_spec:
                規格 = main_spec
            單位 = standardize_unit_for_needle_syringe(單位)
        elif is_needle_only_row(name, 細項, 規格原始):
            品名 = "針"
            規格 = format_needle_spec(name, 規格原始)
            if not 規格 and main_spec:
                規格 = main_spec
            單位 = standardize_unit_for_needle_syringe(單位)
        else:
            # 非針具：保持原品名與規格，簡潔化
            品名 = name
            規格 = main_spec if main_spec else 規格原始

        out_rows.append([str(idx), 分類, 品名, 規格, 廠商, 單位, 包裝規格])

    with open(OUTPUT_CSV, "w", encoding="utf-8-sig", newline="") as f:
        writer = csv.writer(f)
        writer.writerows(out_rows)

    print(f"已寫入 {len(out_rows)-1} 筆至 {OUTPUT_CSV}")


if __name__ == "__main__":
    main()
