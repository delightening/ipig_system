"""R32-A3: 審查意見回覆表 docxtpl payload schema。

對應模板：`templates/review_reply.docx`
對應 mapping：`pdf-service/scripts/mappings/review_reply.py`
原始檔：`templates/APIG-XXXXXX審查意見回覆表範例.docx`

結構：
- T0 計畫基本資訊（申請編號 / 研究名稱 / PI）
- T1 執行秘書初審紀錄（loop：項次 / 審查意見 / 申請人回覆）
- T2 獸醫師審查紀錄（12 個固定 dimension + 簽名日期）
- T3-T6 委員一二三四審查紀錄（每委員獨立 loop）
"""

from typing import Optional

from pydantic import BaseModel, Field


class SecretaryReviewItem(BaseModel):
    """T1 執行秘書一筆。"""

    item_no: str = ""
    section_no: str = ""   # 計畫書項次（如 4.1.2），優先於序號顯示於「項次」欄
    opinion: str = ""
    reply: str = ""


class VetReviewItem(BaseModel):
    """T2 獸醫師審查 — 一個 dimension。"""

    status: str = ""             # V (符合) / X (不符合) / - (不適用) / "" (未審查)
    opinion: str = ""
    reply: str = ""


class VetReview(BaseModel):
    """T2 獸醫師審查 12 個 fixed dimensions。"""

    basic_info: VetReviewItem = Field(default_factory=VetReviewItem)            # R1 計畫基本資料
    research_purpose: VetReviewItem = Field(default_factory=VetReviewItem)      # R2 簡述研究目的
    necessity: VetReviewItem = Field(default_factory=VetReviewItem)             # R3 動物實驗必要性
    experiment_design: VetReviewItem = Field(default_factory=VetReviewItem)     # R4 動物實驗試驗設計
    endpoint: VetReviewItem = Field(default_factory=VetReviewItem)              # R5 預期結束時機及人道終點
    end_handling: VetReviewItem = Field(default_factory=VetReviewItem)          # R6 結束動物處置方式
    hazardous: VetReviewItem = Field(default_factory=VetReviewItem)             # R7 有無危害性物質實驗
    anesthesia: VetReviewItem = Field(default_factory=VetReviewItem)            # R8 麻醉用藥及方法
    surgery: VetReviewItem = Field(default_factory=VetReviewItem)               # R9 手術操作及術中觀察
    postop_care: VetReviewItem = Field(default_factory=VetReviewItem)           # R10 術後照護及給藥
    animal_info: VetReviewItem = Field(default_factory=VetReviewItem)           # R11 實驗動物資料
    personnel: VetReviewItem = Field(default_factory=VetReviewItem)             # R12 相關人員資料
    signature_date: str = ""                                                    # R13 獸醫師簽名 + 日期


class CommitteeReviewItem(BaseModel):
    """T3-T6 委員審查 — 一個 row。"""

    item_no: str = ""
    section_no: str = ""     # 計畫書項次（如 4.1.2），優先於序號顯示於「項次」欄
    opinion_1st: str = ""    # 一審意見
    reply_1st: str = ""      # 一審回覆
    opinion_2nd: str = ""    # 二審意見
    reply_2nd: str = ""      # 二審回覆


class ReviewReplyPayload(BaseModel):
    """審查意見回覆表完整 payload。"""

    application_no: str = ""
    study_title: str = ""
    pi_name: str = ""
    secretary_items: list[SecretaryReviewItem] = Field(default_factory=list)
    vet_review: VetReview = Field(default_factory=VetReview)
    committee_1: list[CommitteeReviewItem] = Field(default_factory=list)
    committee_2: list[CommitteeReviewItem] = Field(default_factory=list)
    committee_3: list[CommitteeReviewItem] = Field(default_factory=list)
    committee_4: list[CommitteeReviewItem] = Field(default_factory=list)
    export_date: Optional[str] = None
