"""R32-A3 收尾：working_content (DB JSONB) → AupProtocolPayload 轉換。

backend Rust 把 protocols.working_content + protocol_meta 直接 POST 過來，
本 adapter 處理 domain transform（同樣邏輯原本在 scripts/smoke_real_data.py
裡面，搬上來成正式 runtime path）。
"""

from __future__ import annotations

import base64
from datetime import datetime, timedelta, timezone
from typing import Any

from schemas.aup_protocol import (
    AbnormalSigns,
    AnesthesiaPlan,
    AnimalRow,
    AsepticMeasures,
    Attachment,
    AupProtocolPayload,
    ControlledDrugRow,
    ControlledSubstances,
    CoverInfo,
    DietaryRestriction,
    DrugRow,
    EuthanasiaTransfer,
    FacilityInfo,
    FigurePhoto,
    FreeTextSections,
    HazardousMaterialEntry,
    HazardousMaterials,
    MaterialItem,
    MaterialPhoto,
    NonPharmaCheck,
    PainAssessment,
    PainCategoryB,
    PainCategoryC,
    PainCategoryD,
    PainCategoryE,
    PersonnelRow,
    PIInfo,
    ProjectClassification,
    ProtocolMeta,
    ReferenceDatabase,
    ReferenceItem,
    StudyDirector,
    SurgeryType,
    TestArticle,
)


# ===== Frontend → PDF schema field mappings =====

# 5.2 資料庫搜尋紀錄 code → (zh, en) 名稱（對齊前端 i18n aup.guidelines.databases）
_DB_NAMES: dict[str, tuple[str, str]] = {
    "A": ("AGRICOLA 資料庫", "AGRICOLA Database"),
    "B": ("MEDLINE 資料庫", "MEDLINE Database"),
    "C": ("CAB Abstracts 資料庫", "CAB Abstracts Database"),
    "D": ("TOXNET 資料庫", "TOXNET Database"),
    "E": ("BIOSIS 資料庫", "BIOSIS Database"),
    "F": ("實驗動物相關的科學期刊或雜誌", "Scientific Journals or Magazines of Laboratory Animals"),
    "G": ("動物福利資訊中心", "Animal Welfare Information Center"),
    "H": ("實驗動物福利文獻索編", "Laboratory Animal Welfare Bibliography"),
    "I": ("替代毒理方法的基準資料", "Bench Marks: Alternative Methods in Toxicology"),
    "J": ("國際替代與動物使用會議資料",
          "The World Congress on Alternatives and Animal Use in the Life Sciences: Education, Research, Testing"),
    "K": ("與同儕直接聯繫（需記錄來源與資訊內容）", "Direct Contact with Peers (Record source and information)"),
    "L": ("其他", "Other"),
}

# 4.1.3 Pain category sub-items: frontend `category_items[]` (e.g. "c_handling_weighing_transport")
# → PDF PainCategory{B,C,D,E} boolean fields.
_PAIN_C_MAP = {
    "c_handling_weighing_transport": "handling_weighing",
    "c_injection_oral_non_irritant": "injection_oral",
    "c_animal_marking": "marking",
    "c_routine_farming": "routine_husbandry",
    "c_general_anesthesia": "full_anesthesia",
    "c_avma_euthanasia": "avma_euthanasia",
    "c_other": "other",
}
_PAIN_D_MAP = {
    "d_stress_transport_sedation": "transport_with_sedative",
    "d_intubation_under_anesthesia": "intubation",
    "d_survival_surgery_under_anesthesia": "survival_surgery_under_anesthesia",
    "d_non_survival_surgery": "non_survival_surgery_under_anesthesia",
    "d_non_lethal_drug_exposure": "sub_lethal_chemical",
    "d_catheter_implantation": "catheter_implant",
    "d_blood_draw_perfusion": "bleeding_perfusion",
    "d_non_preop_food_water_restrict": "food_water_restriction",
    "d_pain_with_analgesia": "pain_with_analgesia",
    "d_induced_anatomical_physiological": "induced_pathology",
    "d_drug_physiological_damage": "chemical_damage",
    "d_eye_skin_irritation_relievable": "eye_skin_irritation",
    "d_other": "other",
}
_PAIN_E_MAP = {
    "e_severe_drug_damage_death": "chemical_severe_damage",
    "e_paralytic_without_anesthesia": "paralytic_no_anesthesia",
    "e_burn_large_skin_wound": "burns_skin_trauma",
    "e_induced_disease": "induced_disease",
    "e_pain_threshold_procedure": "near_pain_threshold",
    "e_chronic_pain_unrelievable": "chronic_pain_disease",
    "e_excessive_food_water_restrict": "extreme_food_water_restriction",
    "e_extreme_environment": "extreme_environment",
    "e_procedure_may_cause_death": "lethal_procedures",
    "e_pain_distress_study": "pain_distress_research",
    "e_non_avma_euthanasia": "non_avma_euthanasia",
    "e_other": "other",
}
_PAIN_B_MAP = {
    "b_breeding_no_procedure": "breeding_only",
    "b_other": "other",
}

# 4.1.4 distress signs: frontend `distress_signs[]` → PDF AbnormalSigns boolean fields
_DISTRESS_SIGNS_MAP = {
    "weight_loss": "weight_loss",
    "reduced_food_water": "food_water_decrease",
    "dehydration": "dehydration",
    "unkempt_fur": "hair_disheveled",
    "isolation_hiding": "self_isolation",
    "self_mutilation": "self_mutilation",
    "abnormal_posture": "abnormal_posture",
    "abnormal_breathing": "abnormal_breathing",
    "abnormal_activity": "abnormal_activity",
    "aggression": "aggressive_behavior",
    "lacrimation_no_blink": "tearing_no_blink",
    "muscle_rigidity_weakness": "muscle_stiffness",
    "tremor_convulsion": "tremor_seizure",
    "vocalization": "vocalization",
    "surgical_site_inflammation": "surgical_site_inflammation",
    "teeth_grinding": "teeth_grinding",
    "other": "other",
}

# 2.2.3 duplicate status: frontend enum → PDF Literal["no","n_a","continuing","yes"]
_DUPLICATE_STATUS_MAP = {
    "no": "no",
    "not_applicable": "n_a",
    "yes_continuation": "continuing",
    "yes_duplicate": "yes",
}


def _pain_b_from_items(items: list[str], other_text: str) -> PainCategoryB:
    sel = set(items)
    return PainCategoryB(
        breeding_only="b_breeding_no_procedure" in sel,
        other="b_other" in sel,
        other_text=other_text if "b_other" in sel else "",
    )


def _pain_c_from_items(items: list[str], other_text: str) -> PainCategoryC:
    sel = set(items)
    kwargs = {field: (key in sel) for key, field in _PAIN_C_MAP.items()}
    return PainCategoryC(other_text=other_text if "c_other" in sel else "", **kwargs)


def _pain_d_from_items(items: list[str], other_text: str) -> PainCategoryD:
    sel = set(items)
    kwargs = {field: (key in sel) for key, field in _PAIN_D_MAP.items()}
    return PainCategoryD(other_text=other_text if "d_other" in sel else "", **kwargs)


def _pain_e_from_items(items: list[str], other_text: str) -> PainCategoryE:
    sel = set(items)
    kwargs = {field: (key in sel) for key, field in _PAIN_E_MAP.items()}
    return PainCategoryE(other_text=other_text if "e_other" in sel else "", **kwargs)


def _signs_from_distress(signs: list[str], other_text: str) -> AbnormalSigns:
    sel = set(signs)
    kwargs = {field: (key in sel) for key, field in _DISTRESS_SIGNS_MAP.items()}
    return AbnormalSigns(other_text=other_text if "other" in sel else "", **kwargs)


# §1 GLP 註冊權責機關 code → 名稱（對齊前端 i18n registrationAuthorityOptions）
_REG_AUTHORITIES: dict[str, str] = {
    "FDA": "美國食品藥物管理局 (FDA)",
    "CE": "歐盟 (CE)",
    "TFDA": "台灣食品藥物管理署 (TFDA)",
    "CFDA": "中國國家食品藥品監督管理總局 (CFDA)",
}

# 2.2.2 替代方案檢索平台 code → 名稱（對齊前端 i18n；en 版沿用，名稱含英文縮寫）
_ALT_PLATFORMS: dict[str, str] = {
    "altbib": "ALTBIB 非動物性替代方法參考文獻搜索工具",
    "db_alm": "DB-ALM 動物試驗替代方法資料庫",
    "re_place": "歐洲動物替代試驗資源平台 (RE-Place)",
    "johns_hopkins": "Johns Hopkins 動物試驗替代中心",
    "taat": "臺灣非動物性替代方法資訊網 (TAAT)",
    "nc3rs_eda": "實驗設計助理 NC3Rs EDA",
    "nc3rs_refinement": "NC3Rs 精緻化技術與策略資料庫",
}


# 2.3.2 單獨飼養原因 code → 名稱（對齊前端 i18n singleHousingReasonOptions）
_SINGLE_HOUSING_REASONS: dict[str, str] = {
    "b1_pregnant_female": "繁殖需求 — 懷孕雌性",
    "b1_breeding_male": "繁殖需求 — 繁殖雄性",
    "b1_post_wean": "繁殖需求 — 斷奶後單獨飼養",
    "b2_post_surgery": "實驗需求 — 術後護理（直至康復）",
    "b2_single_in_group": "實驗需求 — 同一測試組中單隻個體",
    "b2_metabolic_cage": "實驗需求 — 使用代謝籠（≤7 天）",
    "b3_aggressive": "PI 意見 — 攻擊性行為",
    "b3_temporary": "PI 意見 — 僅在需要時暫時使用",
    "b4_other": "其他",
}


def _alt_platform_names(codes: list[Any], other_name: str) -> list[str]:
    """2.2.2 平台 code → 可讀名稱；'other' 接 other_name。"""
    out: list[str] = []
    for c in codes:
        if c == "other":
            out.append(f"其他：{other_name}" if (other_name or "").strip() else "其他")
        else:
            out.append(_ALT_PLATFORMS.get(c, str(c)))
    return out


def _signed_date_gmt8(iso: str) -> str:
    """PI 簽署時間 ISO(UTC) → YYYY/MM/DD（GMT+8 台灣本地）。"""
    if not (iso or "").strip():
        return ""
    try:
        dt = datetime.fromisoformat(iso.replace("Z", "+00:00"))
        return dt.astimezone(timezone(timedelta(hours=8))).strftime("%Y/%m/%d")
    except (ValueError, TypeError):
        return ""


def _material_items(raw: list[Any]) -> list[MaterialItem]:
    """3.1.2/3.1.3 試驗/對照物質 list（多筆，過濾無名稱者）。"""
    out: list[MaterialItem] = []
    for x in raw:
        m = x if isinstance(x, dict) else {}
        if not (m.get("name") or "").strip():
            continue
        out.append(MaterialItem(
            name=m.get("name", ""),
            form=m.get("form", "") or "",
            concentration=m.get("concentration", "") or "",
            expiry=m.get("expiry_date") or m.get("expiry") or "",
            sterile=m.get("is_sterile") if "is_sterile" in m else None,
            non_sterile_justification=m.get("non_sterile_justification", "") or "",
            purpose=m.get("purpose", "") or "",
            storage=m.get("storage_conditions") or m.get("storage") or "",
            photos=_material_photos(m.get("photos")),
        ))
    return out


def _material_photos(raw: Any) -> list[MaterialPhoto]:
    """§3 物質照片：datauri 由 backend 內嵌（>2MB/非圖片時為空，僅留檔名）。"""
    out: list[MaterialPhoto] = []
    for x in raw if isinstance(raw, list) else []:
        p = x if isinstance(x, dict) else {}
        datauri = p.get("datauri", "") or ""
        file_name = p.get("file_name", "") or ""
        if datauri or file_name:
            out.append(MaterialPhoto(datauri=datauri, file_name=file_name))
    return out


def _figure_photos(raw: Any) -> list[FigurePhoto]:
    """4.1.2 流程圖 / 6.4 手術示意圖：datauri 由 backend 內嵌 + caption 圖說。
    datauri 為空（>2MB/非圖片）時保留 caption（模板僅顯示「圖N：說明」一行）。"""
    out: list[FigurePhoto] = []
    for x in raw if isinstance(raw, list) else []:
        p = x if isinstance(x, dict) else {}
        datauri = p.get("datauri", "") or ""
        caption = p.get("caption", "") or ""
        if datauri or caption:
            out.append(FigurePhoto(datauri=datauri, caption=caption))
    return out


def _compose_test_article_explanation(test_items: list[dict[str, Any]]) -> str:
    """Combine frontend items.test_items[] details into PDF §3.1.2 free-text."""
    lines = []
    for i, t in enumerate(test_items, start=1):
        if not isinstance(t, dict):
            continue
        name = str(t.get("name") or "").strip()
        if not name:
            continue
        bits = [f"{i}. {name}"]
        if t.get("concentration"):
            bits.append(f"濃度：{t['concentration']}")
        if t.get("form"):
            bits.append(f"劑型：{t['form']}")
        if t.get("purpose"):
            bits.append(f"用途：{t['purpose']}")
        if t.get("storage_conditions"):
            bits.append(f"保存：{t['storage_conditions']}")
        lines.append("；".join(bits))
    return "\n".join(lines)


def _compose_pain_mitigation(pain: dict[str, Any]) -> str:
    """Combine pain.relief_measures + drug_name + no_relief_justification."""
    measures = pain.get("relief_measures") or []
    parts: list[str] = []
    if measures:
        labels = {
            "alternative_painless_procedure": "改用不造成疼痛/痛苦的替代程序",
            "anesthesia_analgesia": "投予麻醉或止痛藥",
            "humane_euthanasia": "執行人道安樂死",
            "no_relief_with_justification": "未採取緩解措施（有合理科學依據）",
        }
        parts.append("、".join(labels.get(m, m) for m in measures))
    if pain.get("relief_drug_name"):
        parts.append(f"藥品：{pain['relief_drug_name']}")
    if pain.get("no_relief_justification"):
        parts.append(f"未緩解之科學依據：{pain['no_relief_justification']}")
    return "；".join(parts)


def _compose_carcass_disposal(cd: dict[str, Any]) -> str:
    """Combine frontend object {method, vendor_name, vendor_id} into single
    descriptive string for PDF rendering.

    Frontend stores 3 structured fields, but PDF schema expects free-text under
    sections.carcass_disposal (matches IACUC paper form). Previously the
    adapter took only `.method` and dropped vendor info — losing data shown to
    committee in the system UI. This composes all 3 in a readable form.
    """
    method = str(cd.get("method") or "").strip()
    vendor = str(cd.get("vendor_name") or "").strip()
    vid = str(cd.get("vendor_id") or "").strip()
    if not method and not vendor and not vid:
        return ""
    parts = [method] if method else []
    if vendor:
        suffix = f"（管編 {vid}）" if vid else ""
        parts.append(f"委由{vendor}{suffix}執行")
    elif vid:
        parts.append(f"（管編 {vid}）")
    return "；".join(parts) if len(parts) > 1 else parts[0]


def _hazard_entry_from_materials(materials: list[dict[str, Any]], hazard_type: str) -> HazardousMaterialEntry:
    """Filter frontend materials[] by type and collapse into single PDF entry.

    Frontend stores `materials: [{type, agent_name, amount}, ...]` (multi-type
    multi-row), but PDF schema has 3 fixed slots (biological/radioactive/chemical),
    each with a single agent + amount. When multiple materials share a type,
    join names with 、 and amounts with 、 for readable display.

    Returns empty entry (used=False) if no material of this type.
    """
    of_type = [m for m in materials if str(m.get("type") or "").strip() == hazard_type
               and str(m.get("agent_name") or "").strip()]
    if not of_type:
        return HazardousMaterialEntry()
    # Keep agent/amount lists same length so order is preserved — empty amounts
    # render as empty slots between 、 separators, not silently dropped (which
    # would misalign e.g. "Agent1、Agent2" → "10mg" implying Agent1=10mg).
    agents = "、".join(str(m.get("agent_name") or "").strip() for m in of_type)
    amounts = "、".join(str(m.get("amount") or "").strip() for m in of_type)
    return HazardousMaterialEntry(used=True, agent=agents, amount=amounts)


def from_working_content(working_content: dict[str, Any]) -> AupProtocolPayload:
    """把 protocols.working_content JSONB 轉成 AupProtocolPayload。

    `working_content.basic` 內若已含 IACUC 編號 / 日期 / is_glp 直接用；否則保留空字串。
    若 backend 想覆寫 protocol_meta 欄位，可在 POST 前自行 merge 進 working_content.basic。
    """
    if not isinstance(working_content, dict):
        raise TypeError(f"Expected dict for working_content, got {type(working_content).__name__}")
    wc = working_content

    def _d(v: Any) -> dict[str, Any]:
        return v if isinstance(v, dict) else {}

    def _l(v: Any) -> list[Any]:
        return v if isinstance(v, list) else []

    def _sl(v: Any) -> list[str]:
        """複選欄位正規化：陣列原樣、單一字串轉單元素陣列（向後相容舊資料）、其餘空陣列。"""
        if isinstance(v, list):
            return [str(x) for x in v]
        if isinstance(v, str) and v:
            return [v]
        return []

    def _reduction_cell(flag: Any, detail: str) -> str:
        """2.3.x 減量子題顯示（以申請書為準）：是→明細（空白則「是」）、否→「否」、未答(None)→空（模板退化為 —）。"""
        if flag is True:
            return detail if detail.strip() else "是"
        if flag is False:
            return "否"
        return ""

    basic = _d(wc.get("basic"))
    pi_data = _d(basic.get("pi"))
    sd_data = _d(basic.get("sd"))
    sponsor = _d(basic.get("sponsor"))
    facility = _d(basic.get("facility"))
    design = _d(wc.get("design"))
    purpose = _d(wc.get("purpose"))
    animals_data = _d(wc.get("animals"))
    items = _d(wc.get("items"))
    surgery = _d(wc.get("surgery"))
    personnel = _l(wc.get("personnel"))
    guidelines = _d(wc.get("guidelines"))

    # 計劃類型 / 計畫種類為複選（string[]）；舊資料為單一字串時由 _sl 正規化。
    # 比對 key 對齊前端選項與 i18n（真相），修正先前單數/縮寫不符導致勾選框漏勾。
    proj_types = _sl(basic.get("project_type"))
    proj_cats = _sl(basic.get("project_category"))
    funding = _l(basic.get("funding_sources"))

    classification = ProjectClassification(
        type_basic_research=("1_basic_research" in proj_types),
        type_applied_research=("2_applied_research" in proj_types),
        type_premarket=("3_pre_market_testing" in proj_types),
        type_educational=("4_educational" in proj_types),
        type_biologics=("5_biologics_manufacturing" in proj_types),
        type_other=("6_other" in proj_types),
        type_other_text=basic.get("project_type_other", ""),
        cat_medical=("1_medical" in proj_cats),
        cat_agricultural=("2_agricultural" in proj_cats),
        cat_drug_vaccine=("3_drugs_vaccines" in proj_cats),
        cat_health_food=("4_supplements" in proj_cats),
        cat_food=("5_food" in proj_cats),
        cat_toxic_chemical=("6_toxics_chemicals" in proj_cats),
        cat_medical_material=("7_medical_materials" in proj_cats),
        cat_pesticide=("8_pesticide" in proj_cats),
        cat_animal_drug=("9_animal_drugs_vaccines" in proj_cats),
        cat_animal_health=("10_animal_supplements_feed" in proj_cats),
        cat_cosmetics=("11_cosmetics" in proj_cats),
        cat_other=("12_other" in proj_cats),
        cat_other_text=basic.get("project_category_other", ""),
        # 對齊前端表單實際存值（ResearchBasicFields.tsx: 'moa'/'mohw'/'nstc'/'moe'/'env'/'other'）
        funding_moa="moa" in funding,
        funding_mohw="mohw" in funding,
        funding_nstc="nstc" in funding,
        funding_moe="moe" in funding,
        funding_environment="env" in funding,
        funding_other="other" in funding,
        funding_other_text=basic.get("funding_other", ""),
    )

    use_test = items.get("use_test_item") or False
    test_items_list = _l(items.get("test_items"))
    ctrl_items_list = _l(items.get("control_items"))
    first_test = _d(test_items_list[0]) if test_items_list else {}
    first_ctrl = _d(ctrl_items_list[0]) if ctrl_items_list else {}
    # M3: 補上 sterile 欄位（前端 items.test_items[i].is_sterile）
    test_article = TestArticle(
        use_article=bool(use_test),
        name=first_test.get("name", ""),
        expiry=first_test.get("expiry_date") or first_test.get("expiry") or "",
        sterile=first_test.get("is_sterile") if "is_sterile" in first_test else None,
        purpose=first_test.get("purpose", ""),
        storage=first_test.get("storage_conditions") or first_test.get("storage") or "",
    )
    control_article = TestArticle(
        use_article=bool(ctrl_items_list),
        name=first_ctrl.get("name", ""),
        expiry=first_ctrl.get("expiry_date") or first_ctrl.get("expiry") or "",
        sterile=first_ctrl.get("is_sterile") if "is_sterile" in first_ctrl else None,
        purpose=first_ctrl.get("purpose", ""),
        storage=first_ctrl.get("storage_conditions") or first_ctrl.get("storage") or "",
    )

    # C3: 修 plan_type → anesthesia_type（前端 type 沒 plan_type 欄位）
    anes = _d(design.get("anesthesia"))
    anes_type = anes.get("anesthesia_type", "")
    anesthesia = AnesthesiaPlan(
        survival_surgery=(anes_type == "survival_surgery"),
        non_survival_surgery=(anes_type == "non_survival_surgery"),
        isoflurane_only=(anes_type == "gas_only"),
        azaperonum_atropine_isoflurane=(anes_type in ("azaperonum_atropine", "azeperonum_atropine")),
        none=not (anes.get("is_under_anesthesia") or False),
    )

    # C4/C5: pain.category_items → PainCategory{B,C,D,E} 全部 sub-items 接齊
    pain_d = _d(design.get("pain"))
    pain_cat = pain_d.get("category", "")
    pain_items = _l(pain_d.get("category_items"))
    pain_other_text = pain_d.get("category_item_other_text", "")
    pain = PainAssessment(
        cat_b=_pain_b_from_items(pain_items if pain_cat == "B" else [], pain_other_text),
        cat_c=_pain_c_from_items(pain_items if pain_cat == "C" else [], pain_other_text),
        cat_d=_pain_d_from_items(pain_items if pain_cat == "D" else [], pain_other_text),
        cat_e=_pain_e_from_items(pain_items if pain_cat == "E" else [], pain_other_text),
    )

    # C4: AbnormalSigns（4.1.5 痛苦症狀 grid）— 之前完全沒進 payload
    signs = _signs_from_distress(
        _l(pain_d.get("distress_signs")),
        pain_d.get("distress_signs_other_text", ""),
    )

    # M2: dietary 加 other + other_text
    restr = _d(design.get("restrictions"))
    dietary = DietaryRestriction(
        has_restriction=bool(restr.get("is_restricted") or False),
        fast_before_anesthesia=restr.get("restriction_type") == "fasting_before_anesthesia",
        other=restr.get("restriction_type") == "other",
        other_text=restr.get("other_description", "") if restr.get("restriction_type") == "other" else "",
    )

    # M1: end_handling 加 eutha_other / other / other_text
    final = _d(design.get("final_handling"))
    transfer = _d(final.get("transfer"))
    end_handling = EuthanasiaTransfer(
        eutha_kcl=final.get("euthanasia_type") == "kcl",
        eutha_electrocution=final.get("euthanasia_type") == "electrocution",
        eutha_other=final.get("euthanasia_type") == "other",
        eutha_other_text=final.get("euthanasia_other_description", "") if final.get("euthanasia_type") == "other" else "",
        transfer=final.get("method") == "transfer",
        transfer_recipient_name=transfer.get("recipient_name", ""),
        transfer_recipient_org=transfer.get("recipient_org", ""),
        transfer_project=transfer.get("project_name", ""),
        other=final.get("method") == "other",
        other_text=(final.get("other_description") or final.get("other_text") or "") if final.get("method") == "other" else "",
    )

    haz = _d(design.get("hazards"))
    haz_materials = [_d(m) for m in _l(haz.get("materials")) if isinstance(m, dict)]
    hazardous = HazardousMaterials(
        use_any=haz.get("used") or False,
        biological=_hazard_entry_from_materials(haz_materials, "biological"),
        radioactive=_hazard_entry_from_materials(haz_materials, "radioactive"),
        chemical=_hazard_entry_from_materials(haz_materials, "chemical"),
    )

    # C2: 前端 drug_name vs PDF schema name 不對齊 — 顯式 map 避免 silent empty
    ctrl = _d(design.get("controlled_substances"))
    ctrl_items = [_d(c) for c in _l(ctrl.get("items")) if isinstance(c, dict)]
    controlled = ControlledSubstances(
        use_any=ctrl.get("used") or False,
        drugs=[
            ControlledDrugRow(
                name=c.get("drug_name") or c.get("name") or "",
                approval_no=c.get("approval_no", ""),
                amount=c.get("amount", ""),
                authorized_person=c.get("authorized_person", ""),
            )
            for c in ctrl_items
        ],
    )

    np_d = _d(design.get("non_pharma_grade"))
    non_pharma = NonPharmaCheck(
        use=np_d.get("used") or False, explanation=np_d.get("description", "")
    )

    animals = []
    for a in _l(animals_data.get("animals")):
        if not isinstance(a, dict):
            continue
        # A17: 正確處理 species/strain（原本非豬也硬印「☑豬/」、strain_other/species_other 丟失）
        species = a.get("species", "")
        strain = a.get("strain", "")
        if species == "pig":
            sb = {"mini_pig": "迷你豬", "white_pig": "白豬"}.get(strain) or (a.get("strain_other") or "")
            species_breed = f"☑豬 / {sb}" if sb else "☑豬"
        elif species == "other":
            species_breed = f"☑其他：{a.get('species_other') or ''}"
        else:
            species_breed = ""
        sex = a.get("sex", "")
        num = a.get("number", "")
        sex_count = (
            f"公 {num} 頭" if sex == "male"
            else f"母 {num} 頭" if sex == "female"
            else f"不限 {num} 頭"
        )
        age = "不限" if a.get("age_unlimited") else f"{a.get('age_min','')}~{a.get('age_max','')}"
        weight = "不限" if a.get("weight_unlimited") else f"{a.get('weight_min','')}~{a.get('weight_max','')}"
        # C1: 用 frontend source_name，不再硬寫死
        animals.append(AnimalRow(
            species_breed=species_breed,
            sex_count=sex_count,
            age=age,
            weight=weight,
            source=a.get("source_name") or "",
            housing=a.get("housing_location") or basic.get("housing_location", ""),
        ))

    # M4: personnel 加 roles_other_text + trainings_other_text 到對應 string
    people = []
    for p in personnel:
        if not isinstance(p, dict):
            continue
        roles = _l(p.get("roles"))
        roles_str = ", ".join(roles) if isinstance(roles, list) else str(roles)
        roles_other = p.get("roles_other_text", "").strip()
        if "i" in roles and roles_other:
            roles_str = f"{roles_str}（i. {roles_other}）"
        certs = [_d(c) for c in _l(p.get("training_certificates"))]
        cert_str = ", ".join(
            f"{c.get('training_code','')} ({c.get('certificate_no','')})" for c in certs
        )
        trainings_other = p.get("trainings_other_text", "").strip()
        trainings_list = _l(p.get("trainings"))
        if "F" in trainings_list and trainings_other:
            cert_str = f"{cert_str}; F: {trainings_other}" if cert_str else f"F: {trainings_other}"
        people.append(PersonnelRow(
            seq=str(p.get("id", "")),
            name=p.get("name", ""),
            position=p.get("position", ""),
            roles=roles_str,
            years=str(p.get("years_experience", "")),
            trainings=cert_str,
        ))

    drug_items = [_d(d) for d in _l(surgery.get("drugs"))]
    drugs = [
        DrugRow(
            name=d.get("drug_name") or d.get("name", ""),
            dose=d.get("dose", ""),
            route=d.get("route", ""),
            frequency=d.get("frequency", ""),
            purpose=d.get("purpose", ""),
        )
        for d in drug_items
    ]

    # H1-H9: sections 自由文字大面積補齊
    replacement = _d(purpose.get("replacement"))
    alt_search = _d(replacement.get("alt_search"))
    duplicate = _d(purpose.get("duplicate"))
    reduction = _d(purpose.get("reduction"))
    special_care_d = _d(reduction.get("special_care"))
    single_housing_d = _d(reduction.get("single_housing"))
    animal_reuse_d = _d(reduction.get("animal_reuse"))
    endpoints = _d(design.get("endpoints"))
    haz_for_sec = _d(design.get("hazards"))
    multi_surg = _d(surgery.get("multiple_surgeries"))

    # H4: duplicate enum mapping（R60-2 新路徑）
    dup_status_raw = duplicate.get("status", "")
    dup_status = _DUPLICATE_STATUS_MAP.get(dup_status_raw, "no")

    sections = FreeTextSections(
        # E: §1 GLP 註冊權責機關（原本填了完全不輸出）
        registration_authorities=(
            "、".join(
                (f"其他：{basic.get('registration_authority_other', '')}"
                 if a == "other" else _REG_AUTHORITIES.get(a, str(a)))
                for a in _sl(basic.get("registration_authorities"))
            ) if basic.get("is_glp") else ""
        ),
        abstract=(purpose.get("abstract") or purpose.get("significance") or "")[:2000],
        # H3: 結構化 alternatives 路徑（R60-2）
        alternatives_databases=_alt_platform_names(_l(alt_search.get("platforms")), alt_search.get("other_name", "")),
        alternatives_keywords=alt_search.get("keywords", ""),
        alternatives_conclusion=alt_search.get("conclusion", ""),
        alternatives_search=alt_search.get("conclusion", ""),  # legacy fallback
        # H4: duplicate 結構化
        duplicate_status=dup_status,
        duplicate_prev_iacuc=duplicate.get("previous_iacuc_no", ""),
        duplicate_n_a_basis=duplicate.get("regulation_basis", ""),
        duplicate_yes_note=duplicate.get("justification", ""),
        duplicate_experiment=duplicate.get("justification", "") or ("否" if dup_status in ("no", "n_a") else "是"),
        # H2: 2.2.1 活體必要性
        live_animal_necessity=replacement.get("rationale", ""),
        # 2.3 減量原則
        reduction_rationale=reduction.get("design", ""),
        # H5: 2.3.1/2/3 sub-fields
        special_care=_reduction_cell(
            special_care_d.get("needed"),
            special_care_d.get("description", ""),
        ),
        # 修 gemini HIGH: 不 silent drop estimated_duration 或 metabolic_cage_duration —
        # 三欄全部 join，避免「兩個都填 → 後者被 or 短路掉」
        individual_housing=_reduction_cell(
            single_housing_d.get("required"),
            "；".join(filter(None, [
                # A16: 單獨飼養原因（原本整批漏印）
                "、".join(_SINGLE_HOUSING_REASONS.get(r, str(r))
                          for r in _l(single_housing_d.get("reasons"))),
                single_housing_d.get("monitoring_method", "").strip(),
                single_housing_d.get("estimated_duration", "").strip(),
                single_housing_d.get("metabolic_cage_duration", "").strip(),
            ])),
        ),
        # 修 gemini HIGH: plan=="other" 時 plan_other 才是實際內容；非 "other" 才顯示 plan enum
        reuse_after_study=_reduction_cell(
            animal_reuse_d.get("considered"),
            (animal_reuse_d.get("plan_other", "") if animal_reuse_d.get("plan") == "other"
             else animal_reuse_d.get("plan", "")),
        ),
        # H6: 2.4 精緻化（修 mapping source）
        refinement_measures=purpose.get("refinement_description", ""),
        endpoint_humane=(
            f"實驗終點：{endpoints.get('experimental_endpoint', '')}\n\n"
            f"人道終點：{endpoints.get('humane_endpoint', '')}"
        ),
        carcass_disposal=_compose_carcass_disposal(_d(design.get("carcass_disposal"))),
        # H9: 3.1 試驗物質詳細
        test_article_explanation=_compose_test_article_explanation(_l(items.get("test_items"))),
        # 4.1.2 動物試驗內容及流程（vet 於表單 design.procedures 填寫）
        animal_procedures=design.get("procedures", ""),
        procedure_photos=_figure_photos(design.get("procedure_photos")),
        # H1: 4.1.6 緩解措施
        pain_distress_mitigation=_compose_pain_mitigation(pain_d),
        # 4.2 屍體處理已由 carcass_disposal 涵蓋
        non_pharma_grade=_d(design.get("non_pharma_grade")).get("description", ""),
        # H8 / C7: 4.5.1-3 危害性物質處理（之前完全漏）
        hazardous_application=haz_for_sec.get("operation_location_method", ""),
        hazardous_protection=haz_for_sec.get("protection_measures", ""),
        hazardous_disposal=haz_for_sec.get("waste_and_carcass_disposal", ""),
        references=(guidelines.get("content") or "")[:2000],
        # §6 手術相關
        surgery_description=surgery.get("surgery_description", ""),
        surgery_photos=_figure_photos(surgery.get("surgery_photos")),
        intraop_monitoring=surgery.get("monitoring", ""),
        # H7: §6 missing fields
        postop_impact=surgery.get("postop_expected_impact", ""),
        multiple_surgery_yes=bool(multi_surg.get("used") or False),
        # A10: 多次手術 含次數 + 原因
        multiple_surgery=(
            ((f"預計手術次數：{multi_surg.get('number')}\n" if multi_surg.get("number") else "")
             + (multi_surg.get("reason", "") or ""))
            if multi_surg.get("used") else ""
        ),
        # A11: 術後照護分類（骨科 / 非骨科手術）
        postop_care_type={"orthopedic": "骨科手術", "non_orthopedic": "非骨科手術"}.get(
            surgery.get("postop_care_type", ""), ""
        ),
        postop_care=surgery.get("postop_care", ""),
        surgery_endpoint=surgery.get("expected_end_point", ""),
        pre_surgery_prep=surgery.get("preop_preparation", ""),
        # personnel_other_role 多人填寫合併（dict.fromkeys 去重保序）
        personnel_other_role="；".join(dict.fromkeys(
            p.get("roles_other_text", "").strip()
            for p in personnel
            if isinstance(p, dict) and "i" in _l(p.get("roles")) and p.get("roles_other_text")
        )),
    )

    _aseptic = _l(surgery.get("aseptic_techniques"))

    # A12/A13: §5.2 資料庫搜尋（已勾選）+ §5.3 引用文獻列表
    reference_databases = [
        ReferenceDatabase(
            name_zh=_DB_NAMES.get(d.get("code", ""), (d.get("code", ""), d.get("code", "")))[0],
            name_en=_DB_NAMES.get(d.get("code", ""), (d.get("code", ""), d.get("code", "")))[1],
            keywords=d.get("keywords", "") or "",
            note=d.get("note", "") or "",
        )
        for d in (_d(x) for x in _l(guidelines.get("databases")))
        if d.get("checked")
    ]
    reference_list = [
        ReferenceItem(citation=r.get("citation", "") or "", url=r.get("url", "") or "")
        for r in (_d(x) for x in _l(guidelines.get("references")))
        if (r.get("citation") or "").strip()
    ]

    # §9 附件：佐證文件 PDF 檔名清單（內容不內嵌）
    attachments = [
        Attachment(file_name=a.get("file_name", "") or "")
        for a in (_d(x) for x in _l(wc.get("attachments")))
        if (a.get("file_name") or "").strip()
    ]

    # A21: PI 電子簽名（手寫 SVG 內嵌 data URI；上傳圖檔僅顯示聲明）
    _sig_svg = wc.get("handwriting_svg") or ""
    pi_signature_datauri = (
        "data:image/svg+xml;base64," + base64.b64encode(_sig_svg.encode("utf-8")).decode("ascii")
        if _sig_svg.strip() else ""
    )
    pi_signed_date = _signed_date_gmt8(wc.get("signed_at", ""))

    return AupProtocolPayload(
        cover=CoverInfo(
            study_title_zh=basic.get("study_title", ""),
            study_title_en=basic.get("study_title_en", ""),
            sponsor=sponsor.get("name", ""),
            testing_facility=facility.get("title", ""),
        ),
        pi=PIInfo(
            name=pi_data.get("name", ""),
            title=pi_data.get("position", ""),
            phone=pi_data.get("phone", "") + (" 分機 " + pi_data["phone_ext"] if pi_data.get("phone_ext") else ""),
            email=pi_data.get("email", ""),
            address=pi_data.get("address", ""),
            sponsor_name=sponsor.get("name", ""),
            contact_person=sponsor.get("contact_person", ""),
            contact_phone=sponsor.get("contact_phone", "") + (" 分機 " + sponsor["contact_phone_ext"] if sponsor.get("contact_phone_ext") else ""),
            contact_email=sponsor.get("contact_email", ""),
        ),
        sd=StudyDirector(name=sd_data.get("name", ""), email=sd_data.get("email", "")),
        facility=FacilityInfo(
            title=facility.get("title", ""),
            address=facility.get("address", "") or basic.get("housing_location", ""),
        ),
        protocol=ProtocolMeta(
            iacuc_apply_no=basic.get("iacuc_apply_no") or basic.get("apply_study_number", ""),
            iacuc_approval_no=basic.get("iacuc_approval_no", ""),
            apply_date=basic.get("apply_date", ""),
            approval_date=basic.get("approval_date", ""),
            valid_from=basic.get("start_date", ""),
            valid_to=basic.get("end_date", ""),
            is_glp=bool(basic.get("is_glp", False)),
        ),
        sections=sections,
        drugs=drugs,
        animals=animals,
        personnel=people,
        classification=classification,
        test_article=test_article,
        control_article=control_article,
        anesthesia=anesthesia,
        pain=pain,
        signs=signs,        # C4: AbnormalSigns (4.1.5 grid) — 之前完全沒進 payload
        dietary=dietary,
        end_handling=end_handling,
        non_pharma=non_pharma,
        hazardous=hazardous,
        controlled=controlled,
        # A7: 6.1 手術種類（前端 surgery.surgery_type 為單值字串）
        surgery_type=SurgeryType(
            survival=(surgery.get("surgery_type") == "survival"),
            non_survival=(surgery.get("surgery_type") == "non_survival"),
        ),
        # A8: 6.3 無菌措施（前端 surgery.aseptic_techniques[]；注意 sterilized_* → sterile_*）
        aseptic=AsepticMeasures(
            surgical_site_disinfection="surgical_site_disinfection" in _aseptic,
            instrument_disinfection="instrument_disinfection" in _aseptic,
            sterile_gowns_gloves="sterilized_gowns_gloves" in _aseptic,
            sterile_drapes="sterilized_drapes" in _aseptic,
            surgical_hand_disinfection="surgical_hand_disinfection" in _aseptic,
        ),
        test_materials=_material_items(test_items_list),
        control_materials=_material_items(ctrl_items_list),
        reference_databases=reference_databases,
        reference_list=reference_list,
        attachments=attachments,
        pi_signature_datauri=pi_signature_datauri,
        pi_signature_uploaded=bool(_l(wc.get("signature"))) and not pi_signature_datauri,
        pi_signed_date=pi_signed_date,
    )
