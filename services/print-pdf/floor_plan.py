"""R32-A8g 倉庫平面圖：把 locations[] 的 (row_index, col_index, width, height,
color, name) 渲染成 PNG，透過 docxtpl `InlineImage` 注入 warehouse.docx 模板的
`{{ floor_plan }}` placeholder。

對齊 frontend `WarehouseReportPage.tsx::LayoutDiagram`：
- 0-indexed grid
- maxCol = max(col_index + width)；maxRow = max(row_index + height)
- 每個 location 占 (col_index..+width, row_index..+height) 的矩形
- 結構類型（wall/door/window）特殊配色；其他用 location.color 或預設藍
- text label = name || code

設計選擇：用 PNG 而非 Word table（Subdoc 合併 cell + 背景色）：
- 圖片不會被 Word 的 page break / 表格自動拆 row 邏輯打散
- 比例固定（aspectRatio = maxCol:maxRow），與前端視覺一致
- 字型不依 Word 渲染環境
"""

from __future__ import annotations

import re
from io import BytesIO
from typing import Any

from PIL import Image, ImageDraw, ImageFont

# 結構類型固定配色（對齊前端 getStructureColor）
_STRUCTURE_COLORS: dict[str, tuple[int, int, int]] = {
    "wall": (0x99, 0x99, 0x99),
    "door": (0x8B, 0x5A, 0x2B),
    "window": (0xB3, 0xD9, 0xEC),
}

_DEFAULT_COLOR = (0x3B, 0x82, 0xF6)  # frontend default tailwind blue
_BORDER_COLOR = (0x33, 0x33, 0x33)
_BG_COLOR = (0xF5, 0xF5, 0xF5)
_TEXT_COLOR = (0xFF, 0xFF, 0xFF)


def _hex_to_rgb(raw: Any) -> tuple[int, int, int] | None:
    """`#abc` / `abcdef` → (r,g,b)；無效回 None。"""
    if not raw:
        return None
    s = str(raw).lstrip("#").strip()
    if len(s) not in (3, 6):
        return None
    if not all(c in "0123456789abcdefABCDEF" for c in s):
        return None
    if len(s) == 3:
        s = "".join(c * 2 for c in s)
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16))


def _resolve_font(size_px: int, bold: bool = False) -> ImageFont.ImageFont:
    """挑一個能渲染 CJK 的字型。Linux container 上有 Noto CJK；本機 Windows 走
    微軟正黑體 / 標楷體；無都不可用 → fallback PIL default (英文 only)。

    bold=True 優先走 Bold 變體（NotoSansCJK-Bold / msjhbd），fallback Regular。
    """
    if bold:
        candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",
            "C:\\Windows\\Fonts\\msjhbd.ttc",  # 微軟正黑體 Bold
            "C:\\Windows\\Fonts\\msyhbd.ttc",
            "/System/Library/Fonts/PingFang.ttc",
        ]
    else:
        candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "C:\\Windows\\Fonts\\msjh.ttc",
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\kaiu.ttf",
            "/System/Library/Fonts/PingFang.ttc",
        ]
    for path in candidates:
        try:
            return ImageFont.truetype(path, size_px)
        except OSError:
            continue
    return ImageFont.load_default()


# CJK Unicode ranges：CJK Unified, Ext-A, 中日韓常用、台灣/香港字
_CJK_PATTERN = r"一-鿿㐀-䶿豈-﫿"
_RE_CJK_BEFORE_ASCII = re.compile(rf"([{_CJK_PATTERN}])([A-Za-z0-9])")
_RE_ASCII_BEFORE_CJK = re.compile(rf"([A-Za-z0-9])([{_CJK_PATTERN}])")


def _add_cjk_ascii_spacing(text: str) -> str:
    """半形英數字 ↔ 全形 CJK 之間補半形空格（例：'藥品架1' → '藥品架 1'）。
    若兩側已經有空白則跳過（regex 用單字元 lookahead 不會疊加）。
    """
    if not text:
        return text
    out = _RE_CJK_BEFORE_ASCII.sub(r"\1 \2", text)
    out = _RE_ASCII_BEFORE_CJK.sub(r"\1 \2", out)
    return out


# 倉庫平面圖網格安全上限（防範極端輸入導致 canvas 記憶體耗盡）
# 一般倉庫網格 < 50×50；400 已遠超實際需求，但仍防止 100k×100k 之類惡意輸入。
_MAX_GRID_EDGE = 400


def _normalize_locations(raw_locations: Any) -> list[dict[str, Any]]:
    if not isinstance(raw_locations, list):
        return []
    out: list[dict[str, Any]] = []
    for loc in raw_locations:
        if not isinstance(loc, dict):
            continue
        try:
            ri = int(loc.get("row_index", 0))
            ci = int(loc.get("col_index", 0))
            h = max(1, int(loc.get("height", 1)))
            w = max(1, int(loc.get("width", 1)))
        except (TypeError, ValueError):
            continue
        if ri < 0 or ci < 0:
            continue
        if ri + h > _MAX_GRID_EDGE or ci + w > _MAX_GRID_EDGE:
            # 單筆超過上限 → 丟棄，不讓它撐爆 canvas
            continue
        out.append({"loc": loc, "ri": ri, "ci": ci, "h": h, "w": w})
    return out


def render_floor_plan_png(
    raw_locations: Any,
    *,
    target_width_px: int = 1200,
    cell_min_px: int = 40,
) -> bytes:
    """產生倉庫平面圖 PNG bytes。空 locations[] 回 placeholder image。"""
    valid = _normalize_locations(raw_locations)
    if not valid:
        # 空圖：純背景 + 文字
        img = Image.new("RGB", (target_width_px, 200), _BG_COLOR)
        draw = ImageDraw.Draw(img)
        font = _resolve_font(20, bold=True)
        draw.text(
            (target_width_px // 2, 100),
            "（無儲位資料）",
            fill=(0x66, 0x66, 0x66),
            font=font,
            anchor="mm",
        )
        buf = BytesIO()
        img.save(buf, format="PNG")
        return buf.getvalue()

    max_row = max(v["ri"] + v["h"] for v in valid)
    max_col = max(v["ci"] + v["w"] for v in valid)
    # _normalize_locations 已過濾單筆 >_MAX_GRID_EDGE 的 row；再守一道，
    # 防範累積極端值（多筆 ri+h 接近上限的組合）
    if max_row > _MAX_GRID_EDGE or max_col > _MAX_GRID_EDGE:
        raise ValueError(f"floor plan grid {max_col}x{max_row} exceeds safety limit {_MAX_GRID_EDGE}")

    # 維持與前端 aspectRatio = maxCol:maxRow
    cell_px = max(cell_min_px, target_width_px // max_col)
    canvas_w = cell_px * max_col
    canvas_h = cell_px * max_row
    # 留 8px 內外 padding 給陰影
    pad = 8
    img = Image.new("RGB", (canvas_w + pad * 2, canvas_h + pad * 2), _BG_COLOR)
    draw = ImageDraw.Draw(img)

    # 全域底色 + 邊框（對齊前端 .border .rounded .bg-muted）
    draw.rectangle(
        [pad, pad, pad + canvas_w, pad + canvas_h],
        fill=(0xEE, 0xEE, 0xEE),
        outline=_BORDER_COLOR,
        width=1,
    )

    occupied: set[tuple[int, int]] = set()
    font_label = _resolve_font(max(10, cell_px // 4), bold=True)

    for v in valid:
        ri, ci, h, w, loc = v["ri"], v["ci"], v["h"], v["w"], v["loc"]
        if ri + h > max_row or ci + w > max_col:
            continue
        rect_cells = [(r, c) for r in range(ri, ri + h) for c in range(ci, ci + w)]
        if any(cc in occupied for cc in rect_cells):
            continue
        occupied.update(rect_cells)

        # 配色
        loc_type = str(loc.get("location_type") or "").lower()
        rgb = (
            _STRUCTURE_COLORS.get(loc_type)
            or _hex_to_rgb(loc.get("color"))
            or _DEFAULT_COLOR
        )

        x0 = pad + ci * cell_px
        y0 = pad + ri * cell_px
        x1 = x0 + w * cell_px
        y1 = y0 + h * cell_px

        # 矩形 + 半透明白邊（與前端 border-white-30 一致）
        draw.rectangle([x0, y0, x1, y1], fill=rgb, outline=(0xFF, 0xFF, 0xFF), width=2)

        label = _add_cjk_ascii_spacing(
            str(loc.get("name") or loc.get("code") or "").strip()
        )
        if label:
            cx = (x0 + x1) // 2
            cy = (y0 + y1) // 2
            # 簡單 contrast：底色亮 → 黑字；底色暗 → 白字
            r, g, b = rgb
            luminance = 0.299 * r + 0.587 * g + 0.114 * b
            text_color = (0, 0, 0) if luminance > 160 else _TEXT_COLOR
            draw.text((cx, cy), label, fill=text_color, font=font_label, anchor="mm")

    buf = BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


# ----------------------------------------------------------------------
# docxtpl InlineImage / OOXML wrap-square 後處理已移除 — HTML 路徑用 data URI 直接內嵌 PNG，不需 docxtpl 介面。
