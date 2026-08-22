"""IACUC 審查同意書 (PIG-XXXXXX) schema — 對應 `templates/source/PIG-XXXXXX審查同意書.docx`。

非 GLP 受控文件編號（不在 AD-XX-XX-XXX 體系中）；單頁同意書，召集人手簽。
"""
from typing import Optional

from pydantic import BaseModel, Field


class ApprovedAnimal(BaseModel):
    species: str = "豬/迷你豬"
    amount: str = ""           # 「7頭」
    period_from: str = ""      # YYYY/MM/DD
    period_to: str = ""        # YYYY/MM/DD


class PigApprovalPayload(BaseModel):
    """IACUC 同意書。"""

    approval_no: str = ""                 # PIG-XXXXXX
    pi_name: str = ""
    sponsor: str = ""                     # 委託單位
    study_title_zh: str = ""
    study_title_en: str = ""
    housing_location: str = "豬博士動物科技股份有限公司(豬博士畜牧場)"
    animals: list[ApprovedAnimal] = Field(default_factory=list)
    approval_date: str = ""               # YYYY/MM/DD（封面 + 召集人簽署旁）
    chair_name: str = ""                  # 召集人名字（可選；通常列印後手簽）
    export_date: Optional[str] = None
