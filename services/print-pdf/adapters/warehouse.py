"""R32-A8g: backend `WarehouseService::get_report_data` JSON → `WarehouseReportPayload`。

Rust 端 `WarehouseReportData` 結構（serde 後）：
{
  "warehouse": { "code", "name", "address", ... },
  "summary":   { "total_locations", "active_locations", "total_capacity",
                 "total_current_count", "total_inventory_items" },
  "locations": [
    { "code", "name", "location_type", "capacity": Option<i32>, "current_count",
      "inventory": [ { "product_name", "on_hand_qty", "base_uom", "batch_no",
                       "expiry_date", ... }, ... ] },
    ...
  ],
  "generated_at": "<RFC3339>"
}

Adapter 動作：
- locations[].inventory[] → flat InventoryRow list（per location × product × batch）
  以對應「明細」section 的單一 loop 表格設計
- 每 row 都填完整 location_code / location_name（per user 2026-05-07 決策
  「放棄合併」）；不再做 vMerge / 連續 row blank
- 結構類型（wall/door/window）/ 空 inventory 不進 inventory_rows（只出現在平面圖）
- expiry_date null 顯示「—」；數量 / 單位 分開欄位
- generated_at RFC3339 → "YYYY-MM-DD HH:MM"
"""

from __future__ import annotations

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation
from typing import Any
from zoneinfo import ZoneInfo

# 倉庫報表時間一律以台灣時間 (GMT+8) 顯示
_TZ = ZoneInfo("Asia/Taipei")

from schemas.warehouse import (
    InventoryRow,
    WarehouseInfo,
    WarehouseReportPayload,
    WarehouseSummary,
)

_STRUCTURE_TYPES = {"wall", "door", "window"}


def _format_generated_at(raw: Any) -> str:
    """RFC3339 / ISO8601 → 'YYYY-MM-DD HH:MM'；解析失敗回原字串或空。"""
    if not raw:
        return ""
    s = str(raw)
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    try:
        dt = datetime.fromisoformat(s)
    except ValueError:
        return str(raw)
    # 無時區資訊者視為 UTC，統一轉台灣時間 (GMT+8)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(_TZ).strftime("%Y-%m-%d %H:%M")


def _format_qty(qty: Any) -> str:
    """數量整數字串（per user 2026-05-07：DB column numeric(18,4) 但實際使用都是
    整數，列印不顯示小數位）。

    接受：int / float / Decimal string ('4.0000') → 全部 truncate 為 int。
    None / 解析失敗 → 回原值 / 空字串。
    """
    if qty is None:
        return ""
    try:
        return str(int(Decimal(str(qty))))
    except (TypeError, ValueError, InvalidOperation):
        return str(qty)


def _format_expiry(raw: Any) -> str:
    if not raw:
        return "—"
    s = str(raw)
    # date-only or RFC3339
    try:
        dt = datetime.fromisoformat(s.rstrip("Z"))
    except ValueError:
        return s
    return dt.strftime("%Y-%m-%d")


def _build_inventory_rows(locations: Any) -> list[InventoryRow]:
    """攤平 locations[].inventory[] 為 per-product rows。

    每個 row 都填完整 location_code / location_name —— 跨頁時新頁仍顯示位置資訊。
    同頁連續同位置的視覺合併由 post-render 步驟（`merge_consecutive_location_cells`）
    用 vMerge 實作，不在這裡 blank 文字。
    """
    if not isinstance(locations, list):
        return []
    rows: list[InventoryRow] = []
    for loc in locations:
        if not isinstance(loc, dict):
            continue
        loc_type = str(loc.get("location_type") or "").lower()
        if loc_type in _STRUCTURE_TYPES:
            # 牆/門/窗只進平面圖，不列入庫存明細
            continue
        loc_code = str(loc.get("code") or "")
        loc_name = str(loc.get("name") or "")
        inv_list = loc.get("inventory") or []
        if not isinstance(inv_list, list) or not inv_list:
            continue
        for inv in inv_list:
            if not isinstance(inv, dict):
                continue
            qty_raw = inv.get("on_hand_qty") if "on_hand_qty" in inv else inv.get("quantity")
            unit_raw = inv.get("base_uom") if "base_uom" in inv else inv.get("unit")
            rows.append(
                InventoryRow(
                    location_code=loc_code,
                    location_name=loc_name,
                    product_name=str(
                        inv.get("product_name") or inv.get("name") or ""
                    ),
                    # backend 欄位 `product_spec`（與 product_sku / product_name siblings
                    # 命名一致）；保 'spec' fallback 給直接呼 endpoint 的 caller
                    spec=str(inv.get("product_spec") or inv.get("spec") or ""),
                    batch_no=str(inv.get("batch_no") or "—"),
                    quantity=_format_qty(qty_raw),
                    unit=str(unit_raw or ""),
                    expiry_date=_format_expiry(inv.get("expiry_date")),
                )
            )
    return rows


def from_warehouse_report(data: dict[str, Any]) -> WarehouseReportPayload:
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    wh_raw = data.get("warehouse") or {}
    sm_raw = data.get("summary") or {}

    warehouse = WarehouseInfo(
        code=str(wh_raw.get("code") or ""),
        name=str(wh_raw.get("name") or ""),
        address=str(wh_raw.get("address") or ""),
    )
    summary = WarehouseSummary(
        total_locations=int(sm_raw.get("total_locations") or 0),
        active_locations=int(sm_raw.get("active_locations") or 0),
        total_capacity=int(sm_raw.get("total_capacity") or 0),
        total_current_count=int(sm_raw.get("total_current_count") or 0),
        total_inventory_items=int(sm_raw.get("total_inventory_items") or 0),
    )

    return WarehouseReportPayload(
        warehouse=warehouse,
        summary=summary,
        inventory_rows=_build_inventory_rows(data.get("locations")),
        generated_at=_format_generated_at(data.get("generated_at")),
        exporter_name=str(data.get("exporter_name") or ""),
    )
