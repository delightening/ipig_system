"""R32-A8g: 倉庫現況報表 docxtpl payload schema。

對應模板：`templates/warehouse.docx`
對應 Rust DTO：`backend/src/models/warehouse.rs::WarehouseReportData`

設計：
- `warehouse` / `summary` 攤平到 schema 兩個子物件，模板用 `{{ warehouse.code }}` 取值
- `inventory_rows`：per-product flat list（每筆 row 對應 location × product × batch
  的組合）；模板用 `{%tr for inv in inventory_rows %}` row-loop。每 row 都填完整
  location_code / location_name（per user 2026-05-07 決策「放棄合併」）
- 結構類型儲位（wall/door/window）只進倉庫平面圖，不出現在 inventory_rows
- 所有 Optional 欄位 default 字串 ""，讓模板無條件 render 不爆 KeyError
"""

from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field


class WarehouseInfo(BaseModel):
    """倉庫基本資料（對應 Rust `Warehouse` entity 部分欄位）。"""

    code: str = ""
    name: str = ""
    address: str = ""


class WarehouseSummary(BaseModel):
    """彙總統計（對應 Rust `WarehouseReportSummary`）。"""

    total_locations: int = 0
    active_locations: int = 0
    total_capacity: int = 0
    total_current_count: int = 0
    total_inventory_items: int = 0


class InventoryRow(BaseModel):
    """庫存明細一筆（per-product flat row）。

    一筆 row 對應 location × product × batch 的組合。Adapter 把 raw
    locations[].inventory[] 攤平成這個 list，模板用單一 loop 渲染。
    結構類型儲位（wall/door/window）與無庫存儲位不出現在此 list — 只在平面圖呈現。
    """

    location_code: str = ""
    location_name: str = ""
    product_name: str = ""
    spec: str = ""            # products.spec（規格）；可能空
    batch_no: str = ""
    quantity: str = ""        # 純數量字串（不含單位）
    unit: str = ""            # 單位（products.base_uom）
    expiry_date: str = ""     # 格式化 YYYY-MM-DD；無到期日顯示「—」


class WarehouseReportPayload(BaseModel):
    warehouse: WarehouseInfo = Field(default_factory=WarehouseInfo)
    summary: WarehouseSummary = Field(default_factory=WarehouseSummary)
    inventory_rows: list[InventoryRow] = Field(default_factory=list)
    generated_at: str = ""        # adapter 格式化 YYYY-MM-DD HH:MM
    exporter_name: str = ""       # 由 handler 注入 current_user.display_name
