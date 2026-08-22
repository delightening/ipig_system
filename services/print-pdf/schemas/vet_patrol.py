"""R32-A3b: 動物欄位巡視報告（位置圖）xlsx payload schema。

對應模板：`templates/位置圖output.xlsx`
對應 mapping：`pdf-service/scripts/mappings/vet_patrol.py`
工具：`pdf-service/scripts/fill_xlsx.py`（openpyxl-based 位置式填入）

不同於 docx 範本：
- 不走 docxtpl 路線（無 Jinja loop / 條件）
- 純粹 pen_code → fixed cell 對應
- 由 mapping 檔 export 的 PEN_CELLS dict 決定每個 pen 寫到哪格

資料來源（Rust 端）：`Animal` table query by pen_id JOIN，依 pen_location
分組組成 pen → ear_tag list + status。
"""

from typing import Optional

from pydantic import BaseModel, Field


class PenData(BaseModel):
    """單一 pen 的填入資料。

    Attributes:
        ear_tags: 耳號（多筆 join '.'，如 "001.002.003"）
        status: ○ = 全 healthy / 沒動物，● = 含 in_experiment / 特殊狀態
    """

    ear_tags: str = ""
    status: str = "○"


class VetPatrolHeader(BaseModel):
    """報表表頭資訊。"""

    inspector_name: str = ""           # 巡視人
    patrol_date: str = ""              # 巡視日期 YYYY-MM-DD
    period: str = ""                   # AM / PM
    document_no: str = "AD-05-01-02C"  # 文件編號（預設）


class VetPatrolPayload(BaseModel):
    """欄位巡視報告完整 payload。

    pens 為 dict[pen_code, PenData]，pen_code 形如 "A01" / "B12" / "G06" /
    "C20" / "D33" / "E25" / "F03"。
    """

    header: VetPatrolHeader = Field(default_factory=VetPatrolHeader)
    pens: dict[str, PenData] = Field(default_factory=dict)
    export_date: Optional[str] = None
