"""R32-A8b: backend `AnimalSurgeryService::get_surgery_export_data` JSON
→ `SurgeryPayload` 轉換 adapter.

backend 直接 POST 既有 service 回傳的 raw JSON（含 surgery / animal /
source_name / recorded_by_name / pain_assessments），由 pdf-service 端把
JSONB 藥物欄位 flatten 成顯示字串、翻譯 enum、組裝 sub-models 後 render。
"""

from __future__ import annotations

from typing import Iterable

from schemas.surgery import (
    PainAssessmentRow,
    PostOpObservation,
    SurgeryAnimalInfo,
    SurgeryHeader,
    SurgeryPayload,
    VitalSign,
)


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
# 疼痛評估 0-3/0-5 分映射到顯示字串（簡化 — 範本格只放分數即可，UI 端的
# label 由獸醫看 docx 內 footer 的說明文字對照）
_TIME_PERIOD_ZH: dict[str, str] = {
    "morning": "上午",
    "afternoon": "下午",
    "evening": "晚上",
    "AM": "上午",
    "PM": "下午",
}


def _opt_str(v) -> str:
    if v is None:
        return ""
    return str(v)


def _date_str(v) -> str:
    s = _opt_str(v)
    return s.split("T")[0] if "T" in s else s


def _weight_str(v, unit: str = "Kg") -> str:
    """Decimal/float → 「57.0 Kg」整段。空值 → ""。"""
    if v is None or v == "":
        return ""
    try:
        f = float(v)
    except (TypeError, ValueError):
        return _opt_str(v)
    return f"{f:.1f} {unit}"


def _format_drug_jsonb(value) -> str:
    """JSONB 藥物欄位 → 多行展示字串。

    支援多種輸入結構（與 frontend useSurgeryForm.ts 對齊）：
    - dict with sub-keys（如 induction.atropine = {enabled, dose, ...}）→ 列出
      enabled=true 的 sub-key + dose；others list 直接 join
    - list[dict]（MedicationItem[]）→ 每筆 "name dose"
    - 其他 → repr 字串

    回傳多行字串（"\\n" 分隔），方便 docxtpl 在 cell 內呈現。
    """
    if value is None:
        return ""
    lines: list[str] = []
    if isinstance(value, dict):
        for key, sub in value.items():
            if key == "others" and isinstance(sub, list):
                lines.extend(_format_med_item(it) for it in sub if it)
            elif isinstance(sub, dict) and sub.get("enabled"):
                # AnesthesiaDrug 結構：{enabled, dose, ...}
                dose = sub.get("dose") or sub.get("concentration") or ""
                line = f"{key}".strip()
                if dose:
                    line += f" {dose}"
                lines.append(line)
            elif isinstance(sub, list):
                lines.extend(_format_med_item(it) for it in sub if it)
        return "\n".join(filter(None, lines))
    if isinstance(value, list):
        return "\n".join(_format_med_item(it) for it in value if it)
    return _opt_str(value)


def _format_med_item(item) -> str:
    """MedicationItem 單筆 → 「name dose」字串。"""
    if not isinstance(item, dict):
        return _opt_str(item)
    name = (item.get("name") or item.get("drug_name") or "").strip()
    dose = (item.get("dose") or item.get("amount") or "").strip()
    if name and dose:
        return f"{name} {dose}"
    return name or dose or ""


def _build_animal_info(animal_raw: dict) -> SurgeryAnimalInfo:
    breed_raw = (animal_raw.get("breed") or "").lower()
    gender_raw = (animal_raw.get("gender") or "").lower()
    return SurgeryAnimalInfo(
        ear_tag=_opt_str(animal_raw.get("ear_tag")),
        iacuc_no=_opt_str(animal_raw.get("iacuc_no")),
        breed_zh=_BREED_ZH.get(breed_raw, _opt_str(animal_raw.get("breed_other")) or breed_raw),
        gender_zh=_GENDER_ZH.get(gender_raw, gender_raw),
        birth_date=_date_str(animal_raw.get("birth_date")),
    )


def _build_surgery_header(surgery_raw: dict) -> SurgeryHeader:
    return SurgeryHeader(
        surgery_date=_date_str(surgery_raw.get("surgery_date")),
        surgery_site=_opt_str(surgery_raw.get("surgery_site")),
        body_weight="",  # 範本顯示體重欄；surgery row 沒此欄位 → 空（vet 手寫）
        pre_op_observation="",  # 範本「術前觀察記錄」label-only，無 DB 欄位
        induction_anesthesia=_format_drug_jsonb(surgery_raw.get("induction_anesthesia")),
        pre_op_medication=_format_drug_jsonb(surgery_raw.get("pre_surgery_medication")),
        posture=_opt_str(surgery_raw.get("positioning")),
        gas_anesthesia=_format_drug_jsonb(surgery_raw.get("anesthesia_maintenance")),
        anesthesia_observation=_opt_str(surgery_raw.get("anesthesia_observation")),
    )


def _build_vital_signs(value) -> list[VitalSign]:
    """`animal_surgeries.vital_signs` JSONB → list[VitalSign]。

    對齊 frontend `VitalSign` interface：
    {time, breathing_method, heart_rate, respiration_rate, temperature, spo2}
    """
    if not isinstance(value, list):
        return []
    out: list[VitalSign] = []
    for row in value:
        if not isinstance(row, dict):
            continue
        out.append(
            VitalSign(
                respiration_method=_opt_str(row.get("breathing_method")),
                time=_opt_str(row.get("time")),
                heart_rate=_opt_str(row.get("heart_rate")),
                respiration_rate=_opt_str(row.get("respiration_rate")),
                temperature=_opt_str(row.get("temperature")),
                spo2=_opt_str(row.get("spo2")),
            )
        )
    return out


def _build_post_op(surgery_raw: dict) -> PostOpObservation:
    return PostOpObservation(
        reflex_recovery=_opt_str(surgery_raw.get("reflex_recovery")),
        spontaneous_respiration_rate=_opt_str(surgery_raw.get("respiration_rate")),
        post_op_medication=_format_drug_jsonb(surgery_raw.get("post_surgery_medication")),
        remark=_opt_str(surgery_raw.get("remark")),
    )


def _build_pain_assessments(rows: Iterable[dict]) -> list[PainAssessmentRow]:
    out: list[PainAssessmentRow] = []
    for r in rows or []:
        if not isinstance(r, dict):
            continue
        # 給藥欄位：injection_ketorolac 等三個 boolean + post_medications JSONB list
        meds: list[str] = []
        if r.get("injection_ketorolac"):
            meds.append("Ketorolac 注射")
        if r.get("injection_meloxicam"):
            meds.append("Meloxicam 注射")
        if r.get("oral_meloxicam"):
            meds.append("Meloxicam 口服")
        post_meds = r.get("post_medications") or []
        if isinstance(post_meds, list):
            meds.extend(_format_med_item(it) for it in post_meds if it)
        med_str = "\n".join(filter(None, meds))

        # total_score = 各分項加總（incision + attitude + appetite + feces + urine + pain）
        # 任一分項有值就顯示 total（包含全 0 = 完全無異常的合法紀錄）
        # per coderabbit PR #332
        scores = [
            r.get("incision"), r.get("attitude_behavior"), r.get("appetite"),
            r.get("feces"), r.get("urine"), r.get("pain_score"),
        ]
        numeric_scores = [s for s in scores if isinstance(s, (int, float))]
        total = sum(numeric_scores)

        out.append(
            PainAssessmentRow(
                record_date=_date_str(r.get("created_at")),
                post_op_day=_opt_str(r.get("post_op_days")),
                time_period=_TIME_PERIOD_ZH.get(
                    _opt_str(r.get("time_period")), _opt_str(r.get("time_period"))
                ),
                fasted="否",  # care_medication_records 無禁食欄位 — 預設「否」
                standing="",  # 同上 — 範本欄位但 DB 無
                walking="",
                wound_site="",
                wound_condition=_opt_str(r.get("incision")),
                behavior=_opt_str(r.get("attitude_behavior")),
                appetite=_opt_str(r.get("appetite")),
                defecation=_opt_str(r.get("feces")),
                urination=_opt_str(r.get("urine")),
                pain_score=_opt_str(r.get("pain_score")),
                total_score=str(total) if numeric_scores else "",
                medication_or_remark=med_str,
            )
        )
    return out


def from_surgery_data(data: dict) -> SurgeryPayload:
    """主入口：raw JSON → SurgeryPayload。

    `data` 結構對齊 `AnimalSurgeryService::get_surgery_export_data`：
    `{surgery, animal, source_name, recorded_by_name, pain_assessments}`。
    """
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict, got {type(data).__name__}")

    surgery_raw = data.get("surgery") or {}
    animal_raw = data.get("animal") or {}
    if not isinstance(surgery_raw, dict) or not surgery_raw.get("id"):
        raise ValueError("surgery adapter: data.surgery 缺失或無 id")
    if not isinstance(animal_raw, dict) or not animal_raw.get("ear_tag"):
        raise ValueError("surgery adapter: data.animal.ear_tag 缺失")

    return SurgeryPayload(
        animal=_build_animal_info(animal_raw),
        surgery=_build_surgery_header(surgery_raw),
        recorded_by=_opt_str(data.get("recorded_by_name")),
        vital_signs=_build_vital_signs(surgery_raw.get("vital_signs")),
        post_op=_build_post_op(surgery_raw),
        pain_assessments=_build_pain_assessments(data.get("pain_assessments") or []),
    )
