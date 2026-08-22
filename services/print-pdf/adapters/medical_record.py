"""R32-A8a: backend `AnimalMedicalService::get_animal_medical_data` JSON
→ `MedicalRecordPayload` 轉換 adapter.

backend 直接 POST 既有 service 回傳的 raw JSON（含 animal/observations/
surgeries/weights/vaccinations/sacrifice），由 pdf-service 內部 adapter
合併 timeline + 翻譯 enum + 格式化日期/體重 → docx render。

不在 backend 端組 schema 的好處：
- Rust enum → 中文 i18n 不必撒到 backend handler 各處
- timeline merge 邏輯（weights/observations/surgeries 同日多筆）集中一處
- 對齊 aup_protocol / vet_patrol 既有 from-* adapter pattern
"""

from __future__ import annotations

from typing import Iterable

from schemas.medical_record import (
    AnimalInfo,
    MedicalRecordPayload,
    TimelineEvent,
    VaccinationRow,
)


# 對應 backend models/animal/enums.rs 的 serde rename（snake_case）
_BREED_ZH: dict[str, str] = {
    "minipig": "迷你豬",
    "white": "白豬",
    "lyd": "LYD",
    "other": "其他",
}
_GENDER_ZH: dict[str, str] = {
    "male": "公",
    "female": "母",
}
# 對齊 backend utils/jsonb_validation.rs 的 TREATMENT_CATEGORIES / TREATMENT_ROUTES
_TREATMENT_CATEGORY_ZH: dict[str, str] = {
    "dewormer": "驅蟲藥",
    "antibiotic": "抗生素",
    "other": "其他",
}
_TREATMENT_ROUTE_ZH: dict[str, str] = {
    "IM": "IM 肌肉注射",
    "IV": "IV 靜脈注射",
    "SC": "SC 皮下注射",
    "PO": "PO 口服",
}


def _opt_str(v) -> str:
    """None / empty / 數字 → 統一字串輸出，None 變空字串。"""
    if v is None:
        return ""
    return str(v)


def _date_str(v) -> str:
    """ISO date 字串截掉時間部分 — backend 多用 NaiveDate，已是 'YYYY-MM-DD'。"""
    s = _opt_str(v)
    return s.split("T")[0] if "T" in s else s


def _weight_str(v) -> str:
    """Decimal/float → 帶 1 位小數的字串（如 28.0）；None / 空 → ''。"""
    if v is None or v == "":
        return ""
    try:
        f = float(v)
    except (TypeError, ValueError):
        return _opt_str(v)
    return f"{f:.1f}"


def _build_animal_info(animal_raw: dict) -> AnimalInfo:
    breed_raw = (animal_raw.get("breed") or "").lower()
    gender_raw = (animal_raw.get("gender") or "").lower()
    return AnimalInfo(
        ear_tag=_opt_str(animal_raw.get("ear_tag")),
        iacuc_no=_opt_str(animal_raw.get("iacuc_no")),
        breed_zh=_BREED_ZH.get(breed_raw, _opt_str(animal_raw.get("breed_other")) or breed_raw),
        gender_zh=_GENDER_ZH.get(gender_raw, gender_raw),
        birth_date=_date_str(animal_raw.get("birth_date")),
        entry_date=_date_str(animal_raw.get("entry_date")),
        # source_name 由 backend JOIN 後填入；若沒有則空字串
        source_name=_opt_str(animal_raw.get("source_name")),
        entry_weight=_weight_str(animal_raw.get("entry_weight")),
    )


def _build_vaccinations(rows: Iterable[dict]) -> list[VaccinationRow]:
    out: list[VaccinationRow] = []
    for v in rows or []:
        if not isinstance(v, dict):
            continue
        vaccine = (v.get("vaccine") or "").strip()
        deworming = (v.get("deworming_dose") or "").strip()
        out.append(
            VaccinationRow(
                administered_date=_date_str(v.get("administered_date")),
                vaccine=vaccine or "無",
                deworming_dose=deworming or "無",
            )
        )
    return out


def _format_treatments(treatments) -> str:
    """觀察紀錄 treatments JSONB → 人類可讀用藥摘要（GLP/AAALAC 結構化用藥）。

    每筆格式：`[類別] 藥名 劑量 單位（途徑）`，多筆以「；」串接。
    藥名為空的項目略過。
    """
    if not isinstance(treatments, list):
        return ""
    segments: list[str] = []
    for t in treatments:
        if not isinstance(t, dict):
            continue
        drug = _opt_str(t.get("drug")).strip()
        if not drug:
            continue
        category = _TREATMENT_CATEGORY_ZH.get(_opt_str(t.get("category")).strip(), "")
        dose = " ".join(
            x for x in [_opt_str(t.get("dosage")).strip(), _opt_str(t.get("dosage_unit")).strip()] if x
        )
        route = _TREATMENT_ROUTE_ZH.get(_opt_str(t.get("route")).strip(), "")
        seg = " ".join(x for x in [f"[{category}]" if category else "", drug, dose] if x)
        if route:
            seg = f"{seg}（{route}）"
        segments.append(seg)
    return "；".join(segments)


def _build_events(
    weights: Iterable[dict],
    observations: Iterable[dict],
    surgeries: Iterable[dict],
) -> list[TimelineEvent]:
    """Merge 三類 row 為單一時間線，依日期排序。

    - weights → event_date=measure_date, weight=weight, content=""
    - observations → event_date=event_date, weight="", content=observation.content
    - surgeries → event_date=surgery_date, weight="", content="手術: <site> ..."
    """
    events: list[TimelineEvent] = []

    for w in weights or []:
        if not isinstance(w, dict):
            continue
        events.append(
            TimelineEvent(
                event_date=_date_str(w.get("measure_date")),
                weight=_weight_str(w.get("weight")),
                content="",
            )
        )

    for o in observations or []:
        if not isinstance(o, dict):
            continue
        # 軟刪除 row 不出現在病歷
        if o.get("deleted_at"):
            continue
        events.append(
            TimelineEvent(
                event_date=_date_str(o.get("event_date")),
                weight="",
                content=_opt_str(o.get("content")),
                medications=_format_treatments(o.get("treatments")),
            )
        )

    for s in surgeries or []:
        if not isinstance(s, dict):
            continue
        site = _opt_str(s.get("surgery_site"))
        prefix = f"手術: {site}" if site else "手術"
        remark = _opt_str(s.get("remark")).strip()
        content = f"{prefix} {remark}".strip() if remark else prefix
        events.append(
            TimelineEvent(
                event_date=_date_str(s.get("surgery_date")),
                weight="",
                content=content,
            )
        )

    # 依日期排序；同日依 content 字典序穩定排（避免 Python sort 的不穩定排序）
    events.sort(key=lambda e: (e.event_date, e.content))
    return events


def from_animal_data(data: dict) -> MedicalRecordPayload:
    """主入口：raw JSON → MedicalRecordPayload。

    `data` 結構對齊 `AnimalMedicalService::get_animal_medical_data`：
    `{animal, observations, surgeries, weights, vaccinations, sacrifice}`。
    """
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    animal_raw = data.get("animal") or {}
    if not isinstance(animal_raw, dict) or not animal_raw.get("ear_tag"):
        raise ValueError("medical_record adapter: data.animal.ear_tag 缺失")

    return MedicalRecordPayload(
        animal=_build_animal_info(animal_raw),
        vaccinations=_build_vaccinations(data.get("vaccinations") or []),
        events=_build_events(
            data.get("weights") or [],
            data.get("observations") or [],
            data.get("surgeries") or [],
        ),
    )
