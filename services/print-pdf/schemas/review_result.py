"""R32-A3: 審核結果 docxtpl payload schema。

對應模板：`templates/review_result.docx`
對應 mapping：`pdf-service/scripts/mappings/review_result.py`
原始檔：`templates/AD-04-01-10B審核結果.docx`

結構：
- 1 張 7×1 表
  - R0 標題
  - R1 初審結果（3 個 checkbox + 審查意見）
  - R2 審查委員簽章 + 日期
  - R3 召集人簽章 + 日期
  - R4 複審結果（同 R1）
  - R5 審查委員簽章 + 日期
  - R6 召集人簽章 + 日期
"""

from typing import Optional

from pydantic import BaseModel, Field


class ReviewDecision(BaseModel):
    """初審 / 複審 共用結構。"""

    approved: bool = False              # 照案通過
    revise_required: bool = False       # 應改善後複審
    rejected: bool = False              # 不通過
    revision_opinions: str = ""         # 需改善或不通過之審查意見
    reviewer_signature: str = ""        # 審查委員簽章
    reviewer_date: str = ""
    convener_signature: str = ""        # 召集人簽章
    convener_date: str = ""


class ReviewResultPayload(BaseModel):
    initial: ReviewDecision = Field(default_factory=ReviewDecision)
    final: ReviewDecision = Field(default_factory=ReviewDecision)
    export_date: Optional[str] = None
