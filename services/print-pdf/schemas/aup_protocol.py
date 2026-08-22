"""R32-A3: 動物試驗研究計畫書（AUP）docxtpl payload schema。

對應模板：`templates/aup_protocol.docx`
對應 mapping：`pdf-service/scripts/mappings/aup_protocol.py`

設計：
- v1 範圍：cover / PI / SD / facility / protocol meta / 30+ 自由文字段落 +
  3 個 loops（drugs / animals / personnel）
- **不在 v1 範圍**：複雜 checkbox 多選表（T3 計畫類型、T13-T20 試驗物質與
  疼痛評估、T23 安樂死方法、T25-T30 危害物質與管制藥品、T32 手術類型、
  T34 無菌措施）— 這些每格 checkbox 需要 Jinja `{% if %}` 條件式，且 schema
  需逐 option boolean，工程量大。v1 保留 demo 內容，待 v2 條列展開。
"""

from typing import Literal, Optional

from pydantic import BaseModel, Field


class CoverInfo(BaseModel):
    """T0 封面資訊。"""

    study_title_zh: str = ""
    study_title_en: str = ""
    sponsor: str = ""
    testing_facility: str = ""


class PIInfo(BaseModel):
    """T2 計畫主持人資料。"""

    name: str = ""
    title: str = ""
    phone: str = ""
    email: str = ""
    address: str = ""
    sponsor_name: str = ""           # 試驗單位名稱
    contact_person: str = ""         # 聯絡人
    contact_phone: str = ""
    contact_email: str = ""


class StudyDirector(BaseModel):
    """T2 專案主持人資料。"""

    name: str = ""
    email: str = ""


class FacilityInfo(BaseModel):
    """T2 試驗機構及動物飼養場所。"""

    title: str = ""
    address: str = ""


class ProtocolMeta(BaseModel):
    """T2 申請/核准編號 + 試驗時程。"""

    iacuc_apply_no: str = ""
    iacuc_approval_no: str = ""
    apply_date: str = ""
    approval_date: str = ""
    valid_from: str = ""             # 生效起日
    valid_to: str = ""               # 生效迄日
    is_glp: bool = False             # 是否 GLP 動物試驗


class FigurePhoto(BaseModel):
    """4.1.2 流程圖 / 6.4 手術示意圖一張：圖檔 + 說明。

    datauri 由 backend 內嵌（>2MB 或非圖片時為空，僅顯示說明文字）；
    caption 為圖說（模板自動加「圖N：」前綴、全文連續編號）。
    """

    datauri: str = ""
    caption: str = ""


class FreeTextSections(BaseModel):
    """各 1×1 自由文字段落（vet 在 UI 編輯後存為單一 string）。"""

    # §1 GLP 註冊權責機關（GLP 模式才填）
    registration_authorities: str = ""
    # 計畫摘要 / 替代原則 / 減量原則
    abstract: str = ""                       # T4
    alternatives_search: str = ""            # T6 已搜尋之資料庫說明（legacy free-text，保留向後相容）
    duplicate_experiment: str = ""           # T7 (legacy free-text，保留向後相容)
    reduction_rationale: str = ""            # T8 動物分組 / 數量
    special_care: str = ""                   # T9
    individual_housing: str = ""             # T10
    reuse_after_study: str = ""              # T11
    refinement_measures: str = ""            # T12
    live_animal_necessity: str = ""          # 2.2.1 活體動物試驗必要性 + 物種選擇理由
    personnel_other_role: str = ""           # 試驗人員「i.其他」說明
    # 2.2.2 替代方案檢索 — 結構化（databases + keywords + conclusion）
    alternatives_databases: list[str] = Field(default_factory=list)  # ["ALTBIB", "DB-ALM", "TAAT", ...]
    alternatives_keywords: str = ""          # "porcine valve xenograft, decellularized heart valve"
    alternatives_conclusion: str = ""        # "未發現可行非動物性替代方案，本試驗具不可替代性。"
    # 2.2.3 重複試驗 — enum
    duplicate_status: Literal["no", "n_a", "continuing", "yes"] = "no"
    duplicate_prev_iacuc: str = ""           # 延續性實驗：前次 IACUC 編號
    duplicate_n_a_basis: str = ""            # 不適用時的法規依據
    duplicate_yes_note: str = ""             # "是" 的補充說明
    # 試驗物質
    test_article_explanation: str = ""       # T17 試驗物質詳細說明
    # 研究設計
    animal_procedures: str = ""              # 4.1.2 動物試驗內容及流程詳細敘述（design.procedures）
    procedure_photos: list[FigurePhoto] = Field(default_factory=list)  # 4.1.2 流程圖（圖N）
    pain_distress_mitigation: str = ""       # T21 緩解措施
    endpoint_humane: str = ""                # T22 實驗終點 + 人道終點
    carcass_disposal: str = ""               # T24 屍體處理
    non_pharma_grade: str = ""               # T27 非醫藥級化學品
    hazardous_disposal: str = ""             # T28 危害物質廢棄物處理 (4.5.3)
    hazardous_application: str = ""          # T29 危害物質施用方式 (4.5.1)
    hazardous_protection: str = ""           # 4.5.2 試驗人員/動物/環境保護措施
    references: str = ""                     # T31 文獻搜尋結果
    # 手術相關
    pre_surgery_prep: str = ""               # T33 動物術前準備
    surgery_description: str = ""            # T35 手術內容
    surgery_photos: list[FigurePhoto] = Field(default_factory=list)    # 6.4 手術示意圖（圖N）
    intraop_monitoring: str = ""             # T36 術中監控
    postop_impact: str = ""                  # T37 術後影響
    multiple_surgery_yes: bool = False       # 6.7 是否會接受一次以上的手術
    multiple_surgery: str = ""               # T38 多次手術說明
    postop_care_type: str = ""               # 6.8 術後照護分類（骨科 / 非骨科手術）
    postop_care: str = ""                    # T39 術後照護
    surgery_endpoint: str = ""               # T40 手術終點


class DrugRow(BaseModel):
    """T41 手術用藥資訊一筆。"""

    name: str = ""
    dose: str = ""
    route: str = ""           # IM / PO / IV / 吸入 等
    frequency: str = ""
    purpose: str = ""


class AnimalRow(BaseModel):
    """T42 實驗動物資料一筆。"""

    species_breed: str = ""   # 「□豬/迷你豬」等（含 checkbox 文字）
    sex_count: str = ""       # 「公 5 頭/母 3 頭」等
    age: str = ""
    weight: str = ""
    source: str = ""
    housing: str = ""


class PersonnelRow(BaseModel):
    """T43 試驗人員資料一筆。"""

    seq: str = ""
    name: str = ""
    position: str = ""
    roles: str = ""           # 「b, c, d, f」等工作內容代號
    years: str = ""           # 參與動物試驗年數
    trainings: str = ""       # 訓練/資格代號 + 證照編號


# ===== v2: Checkbox 多選表 sub-models =====

class ProjectClassification(BaseModel):
    """T3 計畫類型/種類/經費（多選）。"""

    # 類型 (6 + 其他)
    type_basic_research: bool = False
    type_applied_research: bool = False
    type_premarket: bool = False
    type_educational: bool = False
    type_biologics: bool = False
    type_other: bool = False
    type_other_text: str = ""

    # 種類 (11 + 其他) — 對齊 reference AD-04-01-01F.pdf 1-12 完整項目
    cat_medical: bool = False
    cat_agricultural: bool = False
    cat_drug_vaccine: bool = False
    cat_health_food: bool = False
    cat_food: bool = False
    cat_toxic_chemical: bool = False
    cat_medical_material: bool = False
    cat_pesticide: bool = False
    cat_animal_drug: bool = False
    cat_animal_health: bool = False
    cat_cosmetics: bool = False
    cat_other: bool = False
    cat_other_text: str = ""

    # 經費來源 (6 + 其他)
    funding_moa: bool = False
    funding_mohw: bool = False
    funding_nstc: bool = False
    funding_moe: bool = False
    funding_environment: bool = False
    funding_other: bool = False
    funding_other_text: str = ""


class TestArticle(BaseModel):
    """T13/T14 試驗物質 / T15 對照物質。"""

    use_article: bool = False              # T13/T15 是/否
    name: str = ""
    expiry: str = ""
    sterile: Optional[bool] = None         # 三態：是/否/未填
    purpose: str = ""
    storage: str = ""


class AnesthesiaPlan(BaseModel):
    """T16 麻醉下進行試驗 + 麻醉方法（5 個 row 各自 是/否）。"""

    survival_surgery: bool = False                   # R0
    non_survival_surgery: bool = False               # R1
    isoflurane_only: bool = False                    # R2
    azaperonum_atropine_isoflurane: bool = False     # R3
    none: bool = False                               # R4 否


class PainCategoryB(BaseModel):
    breeding_only: bool = False
    other: bool = False
    other_text: str = ""


class PainCategoryC(BaseModel):
    handling_weighing: bool = False
    injection_oral: bool = False
    marking: bool = False
    routine_husbandry: bool = False
    full_anesthesia: bool = False
    avma_euthanasia: bool = False
    other: bool = False
    other_text: str = ""


class PainCategoryD(BaseModel):
    transport_with_sedative: bool = False
    intubation: bool = False
    survival_surgery_under_anesthesia: bool = False
    non_survival_surgery_under_anesthesia: bool = False
    sub_lethal_chemical: bool = False
    catheter_implant: bool = False
    bleeding_perfusion: bool = False
    food_water_restriction: bool = False
    pain_with_analgesia: bool = False
    induced_pathology: bool = False
    chemical_damage: bool = False
    eye_skin_irritation: bool = False
    other: bool = False
    other_text: str = ""


class PainCategoryE(BaseModel):
    chemical_severe_damage: bool = False
    paralytic_no_anesthesia: bool = False
    burns_skin_trauma: bool = False
    induced_disease: bool = False
    near_pain_threshold: bool = False
    chronic_pain_disease: bool = False
    extreme_food_water_restriction: bool = False
    extreme_environment: bool = False
    lethal_procedures: bool = False
    pain_distress_research: bool = False
    non_avma_euthanasia: bool = False
    other: bool = False
    other_text: str = ""


class PainAssessment(BaseModel):
    """T18 疼痛分級 Cat B/C/D/E（多選）。"""

    cat_b: PainCategoryB = Field(default_factory=PainCategoryB)
    cat_c: PainCategoryC = Field(default_factory=PainCategoryC)
    cat_d: PainCategoryD = Field(default_factory=PainCategoryD)
    cat_e: PainCategoryE = Field(default_factory=PainCategoryE)


class DietaryRestriction(BaseModel):
    """T19 飲食/飲水限制。"""

    has_restriction: bool = False             # R0 否 / R1+R2 是
    fast_before_anesthesia: bool = False
    other: bool = False
    other_text: str = ""


class AbnormalSigns(BaseModel):
    """T20 異常徵兆 grid（5×4 = 16 個 + 其他）。"""

    weight_loss: bool = False
    food_water_decrease: bool = False
    dehydration: bool = False
    hair_disheveled: bool = False
    self_isolation: bool = False
    self_mutilation: bool = False
    abnormal_posture: bool = False
    abnormal_breathing: bool = False
    abnormal_activity: bool = False
    aggressive_behavior: bool = False
    tearing_no_blink: bool = False
    muscle_stiffness: bool = False
    tremor_seizure: bool = False
    vocalization: bool = False
    surgical_site_inflammation: bool = False
    teeth_grinding: bool = False
    other: bool = False
    other_text: str = ""


class EuthanasiaTransfer(BaseModel):
    """T23 安樂死 / 轉讓 / 其他。"""

    eutha_kcl: bool = False                  # 麻醉下 KCl 安樂死
    eutha_electrocution: bool = False        # 麻醉下 220V 電擊
    eutha_other: bool = False
    eutha_other_text: str = ""
    transfer: bool = False
    transfer_recipient_name: str = ""
    transfer_recipient_org: str = ""
    transfer_project: str = ""
    other: bool = False
    other_text: str = ""


class NonPharmaCheck(BaseModel):
    """T25 非醫藥級化學品。"""

    use: bool = False
    explanation: str = ""


class HazardousMaterialEntry(BaseModel):
    """T26 一筆危害物質。"""

    used: bool = False
    agent: str = ""
    amount: str = ""


class HazardousMaterials(BaseModel):
    """T26 危害物質（3 種類 + 否）。"""

    use_any: bool = False
    biological: HazardousMaterialEntry = Field(default_factory=HazardousMaterialEntry)
    radioactive: HazardousMaterialEntry = Field(default_factory=HazardousMaterialEntry)
    chemical: HazardousMaterialEntry = Field(default_factory=HazardousMaterialEntry)


class ControlledDrugRow(BaseModel):
    """T30 管制藥品一筆。"""

    name: str = ""
    approval_no: str = ""
    amount: str = ""
    authorized_person: str = ""


class ControlledSubstances(BaseModel):
    """T30 管制藥品 yes/no + drug list。"""

    use_any: bool = False
    drugs: list[ControlledDrugRow] = Field(default_factory=list)


class SurgeryType(BaseModel):
    """T32 手術類型。"""

    survival: bool = False
    non_survival: bool = False


class AsepticMeasures(BaseModel):
    """T34 無菌措施。"""

    surgical_site_disinfection: bool = False
    instrument_disinfection: bool = False
    sterile_gowns_gloves: bool = False
    sterile_drapes: bool = False
    surgical_hand_disinfection: bool = False


class MaterialPhoto(BaseModel):
    """§3 物質照片一張。datauri 由 backend 內嵌（>2MB 或非圖片時為空，僅顯示檔名）。"""

    datauri: str = ""
    file_name: str = ""


class MaterialItem(BaseModel):
    """3.1.2/3.1.3 試驗 / 對照物質一筆（支援多筆列印）。"""

    name: str = ""
    form: str = ""                 # 劑型
    concentration: str = ""        # 濃度
    expiry: str = ""               # 效期
    sterile: Optional[bool] = None
    non_sterile_justification: str = ""   # 非無菌製備之說明
    purpose: str = ""
    storage: str = ""
    photos: list[MaterialPhoto] = Field(default_factory=list)   # §3 物質照片縮圖


class ReferenceDatabase(BaseModel):
    """5.2 資料庫搜尋紀錄一筆（已勾選）。name 雙語由 adapter 解析。"""

    name_zh: str = ""
    name_en: str = ""
    keywords: str = ""
    note: str = ""


class ReferenceItem(BaseModel):
    """5.3 引用文獻一筆。"""

    citation: str = ""
    url: str = ""


class Attachment(BaseModel):
    """§9 附件一筆（佐證文件 PDF）。列印僅列檔名清單，不內嵌內容。"""

    file_name: str = ""


class AupProtocolPayload(BaseModel):
    """AUP 完整 payload（v1 + v2）。"""

    lang: Literal["zh", "en"] = "zh"        # R60-2b: 渲染語系（zh=中文預設 / en=英文版）
    cover: CoverInfo = Field(default_factory=CoverInfo)
    pi: PIInfo = Field(default_factory=PIInfo)
    sd: StudyDirector = Field(default_factory=StudyDirector)
    facility: FacilityInfo = Field(default_factory=FacilityInfo)
    protocol: ProtocolMeta = Field(default_factory=ProtocolMeta)
    sections: FreeTextSections = Field(default_factory=FreeTextSections)
    drugs: list[DrugRow] = Field(default_factory=list)
    animals: list[AnimalRow] = Field(default_factory=list)
    personnel: list[PersonnelRow] = Field(default_factory=list)
    export_date: Optional[str] = None
    # A21: PI 電子簽名（手寫 SVG → data URI；上傳圖檔僅聲明；簽署日 GMT+8）
    pi_signature_datauri: str = ""
    pi_signature_uploaded: bool = False
    pi_signed_date: str = ""

    # v2 checkbox 多選表
    classification: ProjectClassification = Field(default_factory=ProjectClassification)
    test_article: TestArticle = Field(default_factory=TestArticle)
    control_article: TestArticle = Field(default_factory=TestArticle)
    anesthesia: AnesthesiaPlan = Field(default_factory=AnesthesiaPlan)
    pain: PainAssessment = Field(default_factory=PainAssessment)
    dietary: DietaryRestriction = Field(default_factory=DietaryRestriction)
    signs: AbnormalSigns = Field(default_factory=AbnormalSigns)
    end_handling: EuthanasiaTransfer = Field(default_factory=EuthanasiaTransfer)
    non_pharma: NonPharmaCheck = Field(default_factory=NonPharmaCheck)
    hazardous: HazardousMaterials = Field(default_factory=HazardousMaterials)
    controlled: ControlledSubstances = Field(default_factory=ControlledSubstances)
    surgery_type: SurgeryType = Field(default_factory=SurgeryType)
    aseptic: AsepticMeasures = Field(default_factory=AsepticMeasures)
    # 3.1.2 / 3.1.3 多筆試驗/對照物質（A3：原本只印第一筆，劑型/濃度/非無菌說明漏印）
    test_materials: list[MaterialItem] = Field(default_factory=list)
    control_materials: list[MaterialItem] = Field(default_factory=list)
    # 5.2 / 5.3 參考文獻（A12/A13：原本只印 5.1 自由文字，databases/references 整段漏印）
    reference_databases: list[ReferenceDatabase] = Field(default_factory=list)
    reference_list: list[ReferenceItem] = Field(default_factory=list)
    # §9 附件清單（佐證文件 PDF 檔名；內容不內嵌）
    attachments: list[Attachment] = Field(default_factory=list)
