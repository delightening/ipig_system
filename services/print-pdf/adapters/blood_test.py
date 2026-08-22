"""R32-A8h: backend `list_blood_test_export_rows` JSON → `BloodTestPayload`。

每個 blood_test 的 N 個 items 已被 backend 攤平成獨立 row，
adapter 主要做 None / 型別防禦 + abnormal bool → 顯示字串。
"""

from __future__ import annotations

from typing import Any

from schemas.blood_test import BloodTestItemRow, BloodTestPayload


def _row(raw: Any) -> BloodTestItemRow:
    if not isinstance(raw, dict):
        return BloodTestItemRow()
    is_abnormal = bool(raw.get("is_abnormal") or False)
    return BloodTestItemRow(
        test_date=str(raw.get("test_date") or ""),
        item_name=str(raw.get("item_name") or ""),
        result_value=str(raw.get("result_value") or ""),
        reference_range=str(raw.get("reference_range") or ""),
        is_abnormal=is_abnormal,
        abnormal_mark="✓" if is_abnormal else "",
        created_by_name=str(raw.get("created_by_name") or ""),
    )


def from_blood_test_data(data: dict[str, Any]) -> BloodTestPayload:
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")
    return BloodTestPayload(
        animal_ear_tag=str(data.get("animal_ear_tag") or ""),
        animal_iacuc_no=str(data.get("animal_iacuc_no") or "") or None,
        export_date=str(data.get("export_date") or ""),
        exporter_name=str(data.get("exporter_name") or ""),
        items=[_row(it) for it in (data.get("items") or [])],
    )
