"""R32-A8c: backend `ProtocolService::get_review_result_export_data` JSON
→ `ReviewResultPayload` 轉換 adapter.

文件設計（per vet/QA 確認）：每件 protocol 1 份 review_result.docx，
內含召集人彙整後的初審 + 複審結論。

填值策略：
- `revision_opinions`：自動把該 stage 的 review_comments newline-join 後填入
  （召集人列印前可以看到所有委員意見彙整）
- `approved` / `revise_required` / `rejected`：留 False（召集人列印後勾選）
- `reviewer_signature` / `reviewer_date` / `convener_signature` / `convener_date`：
  留空字串（召集人手寫簽名 + 日期）

不自動推導 decision flags 是刻意決策 — 系統 protocol.status 反映「目前狀態」
不必然等於召集人在 docx 上想標記的彙整結論；尊重召集人手寫權威性。
"""

from __future__ import annotations

from typing import Any, Iterable

from schemas.review_result import ReviewDecision, ReviewResultPayload


def _join_comments(comments: Iterable[str]) -> str:
    return "\n".join((c or "").strip() for c in comments if c)


def _normalize_comments(raw: Any) -> list[str]:
    """list → list[str]、str → [str]、其他 → []，避免被當 iterable 拆字元。"""
    if isinstance(raw, list):
        return [str(c) for c in raw if c is not None]
    if isinstance(raw, str):
        return [raw]
    return []


def from_review_data(data: dict) -> ReviewResultPayload:
    """主入口：raw JSON → ReviewResultPayload。

    `data` 結構對齊 `ProtocolService::get_review_result_export_data`：
    `{protocol, initial_comments, final_comments}`。
    """
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    initial_opinions = _join_comments(_normalize_comments(data.get("initial_comments")))
    final_opinions = _join_comments(_normalize_comments(data.get("final_comments")))

    return ReviewResultPayload(
        initial=ReviewDecision(revision_opinions=initial_opinions),
        final=ReviewDecision(revision_opinions=final_opinions),
    )
