"""R32-A8e: backend `ProtocolService::get_review_reply_export_data` JSON
→ `ReviewReplyPayload` 轉換 adapter.

對應範本 templates/review_reply.docx（schema 詳見 schemas/review_reply.py）：
- 計畫 metadata（application_no / study_title / pi_name）
- secretary_items[]：執行秘書 PRE_REVIEW 意見 + 申請人回覆
- vet_review.{12 fixed dimensions}：獸醫師審查 12 項
- committee_1/2/3/4：UNDER_REVIEW 委員意見（前 4 位）
"""

from __future__ import annotations

from schemas.review_reply import (
    CommitteeReviewItem,
    ReviewReplyPayload,
    SecretaryReviewItem,
    VetReview,
    VetReviewItem,
)


# 12 dimensions 在 VetReviewForm.items list 內的固定 index → schema 欄位 mapping
# 對齊 backend services/protocol/review.rs 內 default_items 順序
_VET_DIMENSION_FIELDS: list[str] = [
    "basic_info",          # 0 計畫基本資料
    "research_purpose",    # 1 簡述研究目的
    "necessity",           # 2 動物實驗必要性
    "experiment_design",   # 3 動物實驗試驗設計
    "endpoint",            # 4 預期結束時機及人道終點
    "end_handling",        # 5 結束動物處置方式
    "hazardous",           # 6 有無危害性物質實驗
    "anesthesia",          # 7 麻醉用藥及方法
    "surgery",             # 8 手術操作及術中觀察
    "postop_care",         # 9 術後照護及給藥
    "animal_info",         # 10 實驗動物資料
    "personnel",           # 11 相關人員資料
]


def _build_secretary_items(rows) -> list[SecretaryReviewItem]:
    out: list[SecretaryReviewItem] = []
    for r in rows or []:
        if not isinstance(r, dict):
            continue
        out.append(
            SecretaryReviewItem(
                item_no=str(r.get("item_no") or ""),
                section_no=str(r.get("section_no") or ""),
                opinion=str(r.get("opinion") or ""),
                reply=str(r.get("reply") or ""),
            )
        )
    return out


def _build_vet_review(items, signed_date: str) -> VetReview:
    """Vet review form items list → VetReview 12 named dimensions。

    Backend 端 form.items 順序與 _VET_DIMENSION_FIELDS 對齊；超過 12 項忽略，
    不足則對應 dimension 留預設空 VetReviewItem。
    """
    vr = VetReview(signature_date=signed_date or "")
    if not isinstance(items, list):
        return vr
    for idx, raw in enumerate(items):
        if idx >= len(_VET_DIMENSION_FIELDS):
            break
        if not isinstance(raw, dict):
            continue
        item = VetReviewItem(
            status=str(raw.get("status") or ""),
            opinion=str(raw.get("opinion") or ""),
            reply=str(raw.get("reply") or ""),
        )
        setattr(vr, _VET_DIMENSION_FIELDS[idx], item)
    return vr


def _build_committee(rows) -> list[CommitteeReviewItem]:
    out: list[CommitteeReviewItem] = []
    for r in rows or []:
        if not isinstance(r, dict):
            continue
        out.append(
            CommitteeReviewItem(
                item_no=str(r.get("item_no") or ""),
                section_no=str(r.get("section_no") or ""),
                opinion_1st=str(r.get("opinion_1st") or ""),
                reply_1st=str(r.get("reply_1st") or ""),
                opinion_2nd=str(r.get("opinion_2nd") or ""),
                reply_2nd=str(r.get("reply_2nd") or ""),
            )
        )
    return out


def from_review_reply_data(data: dict) -> ReviewReplyPayload:
    """主入口：raw JSON → ReviewReplyPayload。

    `data` 結構對齊 `ProtocolService::get_review_reply_export_data`。
    """
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    return ReviewReplyPayload(
        application_no=str(data.get("application_no") or ""),
        study_title=str(data.get("study_title") or ""),
        pi_name=str(data.get("pi_name") or ""),
        secretary_items=_build_secretary_items(data.get("secretary_items")),
        vet_review=_build_vet_review(
            data.get("vet_items"), str(data.get("vet_signed_date") or "")
        ),
        committee_1=_build_committee(data.get("committee_1")),
        committee_2=_build_committee(data.get("committee_2")),
        committee_3=_build_committee(data.get("committee_3")),
        committee_4=_build_committee(data.get("committee_4")),
    )
