"""R32-A3b 收尾：animals + header → VetPatrolPayload 轉換。

backend Rust 把 animals query 結果（以 pen_location group） + 巡視人/日期 直接 POST
過來。本 adapter 把 list[AnimalRow] 依 pen_location 分組，組成 pens dict 與
ear_tags 字串（zone-aware 格式化）。
"""

from __future__ import annotations

from collections import defaultdict
from typing import Iterable

from schemas.vet_patrol import PenData, VetPatrolHeader, VetPatrolPayload


def _format_ear_tags(zone: str, tags: list[str]) -> str:
    """依 zone 組合 ear tags 字串，避免 Excel wrap 把 3-digit 數字拆斷。

    - G zone：與 A/B/C/D/E 一樣輸出純 `.` 連字串。每 3 號換行由 template
      `pen_tags` macro 負責（split('.') → batch(3) → 插 `<br>`）。**不在此處
      插 `\\n`**：macro 以 `.` 為 split 點，殘留的 `\\n` 會被當成 token 一部分，
      最終在 HTML cell（無 `white-space: pre`）render 成空白而非點，造成
      「277.278.279 760.」這種分隔錯位（per-user 回報）。
    - F zone：1 個一行（cell 較窄），無分隔
    - 其他 (A/B/C/D/E)：全部用 `.` 連，shrink_to_fit 處理長度
    """
    if not tags:
        return ""
    if zone == "F":
        return "\n".join(tags)
    return ".".join(tags)


# 所有已知 pen ID — 範本中每個 pen 都有 status marker cell，必須預填 ○ 讓
# 空欄位也顯示圈圈（沒動物時 → ○；有 in_experiment 動物 → ●；其他狀態 → ○）。
# 對應 scripts/mappings/vet_patrol.py PEN_CELLS keys 的全集。
_ALL_PENS: list[str] = (
    [f"A{i:02d}" for i in range(1, 21)]
    + [f"B{i:02d}" for i in range(1, 21)]
    + [f"C{i:02d}" for i in range(1, 21)]
    + [f"D{i:02d}" for i in range(1, 34)]
    + [f"E{i:02d}" for i in range(1, 26)]
    + [f"F{i:02d}" for i in range(1, 7)]
    + [f"G{i:02d}" for i in range(1, 7)]
)


def from_animals(
    animals: Iterable[dict],
    inspector_name: str = "",
    patrol_date: str = "",
    period: str = "",
) -> VetPatrolPayload:
    """`animals` 為 list of dict，每筆需含 pen_location / ear_tag / status。

    status: "in_experiment" → ●，其他（含空 / 無動物）→ ○
    """
    grouped: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for a in animals or []:
        if not isinstance(a, dict):
            continue
        pen = (a.get("pen_location") or "").strip()
        if not pen:
            continue
        ear_tag = str(a.get("ear_tag") or "").strip()
        grouped[pen].append((ear_tag, str(a.get("status") or "")))

    # 預填所有 pen ○ — 空欄位也要顯示圈圈
    pens: dict[str, PenData] = {p: PenData(ear_tags="", status="○") for p in _ALL_PENS}
    for pen, animal_list in grouped.items():
        zone = pen[:1]
        ear_tags = _format_ear_tags(zone, [t for t, _ in animal_list if t])
        has_inexp = any(s == "in_experiment" for _, s in animal_list)
        pens[pen] = PenData(ear_tags=ear_tags, status="●" if has_inexp else "○")

    return VetPatrolPayload(
        header=VetPatrolHeader(
            inspector_name=inspector_name,
            patrol_date=patrol_date,
            period=period,
        ),
        pens=pens,
    )
