"""R32-A8h: 動物血液檢查紀錄 docxtpl payload schema。

對應模板：`templates/blood_test.docx`
對應 backend：`AnimalBloodTestService::list_blood_test_export_rows`

設計：6 欄 per-item 攤平
- 檢查日期 / 項目 / 檢驗值（含單位）/ 參考值 / 異常 / 建立者
- 每筆 blood_test 的 N 個 items 各佔一列；同一 test_date 多筆 items 重複顯示日期
- result_value 由 backend 已合併 unit（"8.2 10^3/uL"）
"""

from typing import Optional

from pydantic import BaseModel, Field


class BloodTestItemRow(BaseModel):
    test_date: str = ""
    item_name: str = ""
    result_value: str = ""        # 含單位
    reference_range: str = ""
    is_abnormal: bool = False     # adapter 轉成 "✓" / "" 時用 abnormal_mark
    abnormal_mark: str = ""       # 模板顯示用：true → "✓"，false → ""
    created_by_name: str = ""


class BloodTestPayload(BaseModel):
    animal_ear_tag: str = ""
    animal_iacuc_no: Optional[str] = None
    export_date: str = ""
    exporter_name: str = ""
    items: list[BloodTestItemRow] = Field(default_factory=list)
