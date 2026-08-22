"""R32-A3: 實驗豬隻病歷總表 docxtpl payload schema。

對應模板：`templates/medical_record.docx`
對應 mapping：`pdf-service/scripts/mappings/medical_record.py`

設計（per CLAUDE.md 高風險決策）：
- D2 (a) 統一 timeline：weights / observations / surgery 由 backend 端 merge
  排序成 `events` array，模板用單一 loop（與範例 docx 同日期混排格式對齊）
- D3 flat：animal 欄位攤平到一層，模板直接 `{{ animal.ear_tag }}`
- D1 vaccination：日期必填，疫苗或驅蟲二選一，空欄填 "無"
"""

from typing import Optional

from pydantic import BaseModel, Field


class AnimalInfo(BaseModel):
    """動物基本資訊（模板 Table 0 對應）。

    來自 Rust `Animal` entity，但 enum 欄位（breed/gender）由 backend
    preprocess 成中文顯示字串；FK 欄位（source_id）JOIN 成名稱。
    """

    ear_tag: str
    iacuc_no: str = ""
    breed_zh: str = ""           # Animal.breed enum → 中文（如「迷你豬」）
    gender_zh: str = ""          # Animal.gender enum → 中文（公/母）
    birth_date: str = ""         # Animal.birth_date 格式化 YYYY-MM-DD
    entry_date: str = ""         # Animal.entry_date 格式化 YYYY-MM-DD
    source_name: str = ""        # Animal.source_id JOIN 取 name
    entry_weight: str = ""       # Animal.entry_weight 格式化字串（含 .0）


class VaccinationRow(BaseModel):
    """疫苗/驅蟲紀錄一筆（Table 1 loop body 對應）。

    per D1：日期必填；疫苗或驅蟲擇一，空欄由 backend 填 "無"。
    """

    administered_date: str
    vaccine: str = "無"
    deworming_dose: str = "無"


class TimelineEvent(BaseModel):
    """體重/手術/觀察混合時間線一筆（Table 2 loop body 對應）。

    per D2(a)：backend handler 端把 AnimalWeight + AnimalObservation +
    AnimalSacrifice 三類資料合併、依日期排序成單一 array。
    - 純體重 row：weight 有值，content 空
    - 純觀察 row：weight 空，content 有值
    - 同日多筆視為多個獨立 event（與範例 docx 一致：T2R10/R11 同 2021-07-23）
    """

    event_date: str
    weight: str = ""             # "" 或 "28.0" 等格式化字串
    content: str = ""            # 觀察/手術文字，含「左眼(手術)」等
    medications: str = ""        # 觀察紀錄用藥/施打摘要（含類別/劑量/途徑），GLP/AAALAC 結構化用藥


class MedicalRecordPayload(BaseModel):
    """病歷總表完整 payload（flat structure per D3）。"""

    animal: AnimalInfo
    vaccinations: list[VaccinationRow] = Field(default_factory=list)
    events: list[TimelineEvent] = Field(default_factory=list)
    export_date: Optional[str] = None
