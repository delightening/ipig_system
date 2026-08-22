"""R39: 獸醫巡場報告 adapter — backend JSON → VetPatrolReportPayload + InlineImage 注入.

Photos 從 backend 端帶 base64 data URL 字串進來（src 欄位），
本 adapter 解碼為 bytes → BytesIO → `docxtpl.InlineImage` 物件，
回填 ctx 後 docxtpl 範本即可直接 render（範本用
`{%p for pair in cat.photos | batch(2) %}` nested loop 排版）。

不在 backend 端做 InlineImage：docxtpl InlineImage 需綁定特定 DocxTemplate
實例，無法 serialize via JSON；只能在 pdf-service 端 render 時即時建立。
"""

from __future__ import annotations

import base64
import logging
import re
from io import BytesIO
from typing import Any

from PIL import Image, UnidentifiedImageError

from schemas.vet_patrol_report import (
    EntryRow,
    PhotoEntry,
    PhotoGroup,
    SubEntry,
    VetPatrolReportPayload,
)

logger = logging.getLogger(__name__)


_DATA_URL_RE = re.compile(r"^data:([^;]+);base64,(.+)$", re.DOTALL)


def _opt_str(v: Any) -> str:
    if v is None:
        return ""
    return str(v)


def _parse_data_url(src: str) -> bytes | None:
    """解 'data:image/jpeg;base64,...' → bytes；非 data URL 或解碼失敗回 None."""
    if not src:
        return None
    m = _DATA_URL_RE.match(src)
    if not m:
        return None
    try:
        return base64.b64decode(m.group(2), validate=False)
    except (base64.binascii.Error, ValueError):
        return None


_MAX_PX = 1200
_JPEG_QUALITY = 75


def _compress_data_url(src: str) -> str:
    """壓縮 data URL 內的圖片：max 1200px 寬 + JPEG quality 75。"""
    raw = _parse_data_url(src)
    if raw is None:
        return src
    try:
        img = Image.open(BytesIO(raw))
        if img.mode in ("RGBA", "P"):
            img = img.convert("RGB")
        w, h = img.size
        if w > _MAX_PX:
            ratio = _MAX_PX / w
            img = img.resize((_MAX_PX, int(h * ratio)), Image.LANCZOS)
        buf = BytesIO()
        img.save(buf, format="JPEG", quality=_JPEG_QUALITY, optimize=True)
        b64 = base64.b64encode(buf.getvalue()).decode()
        return f"data:image/jpeg;base64,{b64}"
    except (UnidentifiedImageError, Exception):
        logger.warning("photo compression failed, using original")
        return src


def from_report_data(data: dict) -> VetPatrolReportPayload:
    """主入口：backend JSON → VetPatrolReportPayload (尚未轉 InlineImage)."""
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    categories_raw = data.get("categories") or []
    if not isinstance(categories_raw, list):
        raise ValueError("categories 必須為 list")

    categories: list[EntryRow] = []
    for cat_idx, cat in enumerate(categories_raw):
        if not isinstance(cat, dict):
            continue
        photos_raw = cat.get("photos") or []
        if not isinstance(photos_raw, list):
            raise ValueError(f"categories[{cat_idx}].photos 必須為 list")
        photos = [
            PhotoEntry(src=_compress_data_url(_opt_str(p.get("src"))), caption=_opt_str(p.get("caption")))
            for p in photos_raw
            if isinstance(p, dict)
        ]
        sub_entries_raw = cat.get("entries") or []
        sub_entries = [
            SubEntry(
                observation=_opt_str(se.get("observation")),
                suggestion=_opt_str(se.get("suggestion")),
                follow_up=_opt_str(se.get("follow_up")),
            )
            for se in sub_entries_raw
            if isinstance(se, dict)
        ]
        categories.append(
            EntryRow(
                label=_opt_str(cat.get("label")),
                observation=_opt_str(cat.get("observation")),
                suggestion=_opt_str(cat.get("suggestion")),
                follow_up=_opt_str(cat.get("follow_up")),
                entries=sub_entries,
                photos=photos,
            )
        )

    # report-level 接受 `photos` 為主、`report_photos` 為相容；統一存入 payload.photos
    photos_raw = data.get("photos")
    if photos_raw is None:
        photos_raw = data.get("report_photos") or []
    if not isinstance(photos_raw, list):
        raise ValueError("photos/report_photos 必須為 list")
    report_photos = [
        PhotoEntry(src=_compress_data_url(_opt_str(p.get("src"))), caption=_opt_str(p.get("caption")))
        for p in photos_raw
        if isinstance(p, dict)
    ]

    # R39-D2：同隻豬一組（backend 已分組好）
    groups_raw = data.get("photo_groups") or []
    if not isinstance(groups_raw, list):
        raise ValueError("photo_groups 必須為 list")
    photo_groups: list[PhotoGroup] = []
    for g_idx, g in enumerate(groups_raw):
        if not isinstance(g, dict):
            continue
        srcs_raw = g.get("srcs") or []
        if not isinstance(srcs_raw, list):
            raise ValueError(f"photo_groups[{g_idx}].srcs 必須為 list")
        photos_raw_g = g.get("photos") or []
        if not isinstance(photos_raw_g, list):
            raise ValueError(f"photo_groups[{g_idx}].photos 必須為 list")

        # 有 `photos`（新版 backend）就以它為準，並由它回填 `srcs`——
        # 兩邊各壓一次會把每張照片重新編碼兩遍（CPU + 記憶體翻倍，且 srcs 那份沒人用）。
        group_photos = [
            PhotoEntry(src=_compress_data_url(_opt_str(p.get("src"))), caption=_opt_str(p.get("caption")))
            for p in photos_raw_g
            if isinstance(p, dict) and p.get("src")
        ]
        if group_photos:
            srcs = [p.src for p in group_photos]
        else:
            srcs = [_compress_data_url(_opt_str(s)) for s in srcs_raw if s]

        photo_groups.append(
            PhotoGroup(
                caption=_opt_str(g.get("caption")),
                description=_opt_str(g.get("description")),
                srcs=srcs,
                photos=group_photos,
            )
        )

    return VetPatrolReportPayload(
        vet_name=_opt_str(data.get("vet_name")),
        companion=_opt_str(data.get("companion")),
        patrol_date=_opt_str(data.get("patrol_date") or "draft"),
        patrol_date_display=_opt_str(data.get("patrol_date_display")),
        categories=categories,
        photos=report_photos,
        photo_groups=photo_groups,
    )

# docxtpl InlineImage / build_render_context 已移除 — HTML 路徑直接用 photo.src (data URL).
