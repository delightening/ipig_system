"""R32-A3: 實驗豬隻手術紀錄表 docxtpl payload schema。

對應模板：`templates/surgery.docx`
對應 mapping：`pdf-service/scripts/mappings/surgery.py`

設計（沿用 medical_record 已驗證模式）：
- D3 flat：animal / header / post_op 各成一個 sub-model，模板用 `{{ animal.* }}`
- 一份 docx = 一次手術（1 surgery → 1 vital_signs timeline → N 天 pain_assessments）
- 所有欄位以 str 為主：保留 vet 自由填寫空間 + "無" / "N/A" 一致；數值由 frontend
  / backend 端決定是否做 validator
- 對應 Rust 端：`AnimalObservation` (record_type=手術) + 後續多日觀察事件
"""

from typing import Optional

from pydantic import BaseModel, Field


class SurgeryAnimalInfo(BaseModel):
    """動物基本資訊（T0/T3 共用）。"""

    ear_tag: str
    iacuc_no: str = ""
    breed_zh: str = ""
    gender_zh: str = ""
    birth_date: str = ""


class SurgeryHeader(BaseModel):
    """手術主資訊（T0 後半段）。"""

    surgery_date: str
    surgery_site: str = ""           # 左眼 / 右眼 / 雙眼 等
    body_weight: str = ""            # 「57.0 Kg」整段（含單位）
    pre_op_observation: str = ""     # 術前觀察記錄
    induction_anesthesia: str = ""   # 誘導麻醉用藥（多行 join "\n"）
    pre_op_medication: str = ""      # 術前給藥（多行 join "\n"）
    posture: str = ""                # 固定姿勢（如「右側臥」）
    gas_anesthesia: str = ""         # 氣體麻醉描述
    anesthesia_observation: str = ""


class VitalSign(BaseModel):
    """麻醉時間軸生理數值一筆（T1 loop body）。"""

    respiration_method: str = ""     # 呼吸方式
    time: str = ""                   # HH:MM
    heart_rate: str = ""
    respiration_rate: str = ""
    temperature: str = ""            # 體溫（攝氏）
    spo2: str = ""                   # 血氧 SPO2


class PostOpObservation(BaseModel):
    """術後觀察記錄（T2 5×2 label-value）。"""

    reflex_recovery: str = ""              # 反射恢復觀察
    spontaneous_respiration_rate: str = "" # 自主呼吸 次/分鐘
    post_op_medication: str = ""           # 術後給藥
    remark: str = ""                       # 備註


class PainAssessmentRow(BaseModel):
    """術後疼痛評估照護給藥紀錄一筆（T5 loop body）。

    對應 docx 表頭 15 欄：
    日期 / 術後天數 / 時段 / 禁食 / 活動力站立 / 活動力行走 / 手術部位
    / 傷口狀況 / 態度行為 / 食慾 / 排便 / 排尿 / 疼痛分數 / 總分 / 註記
    """

    record_date: str = ""
    post_op_day: str = ""
    time_period: str = ""             # 上午 / 下午
    fasted: str = "否"
    standing: str = ""                # 活動力 / 站立
    walking: str = ""                 # 活動力 / 行走
    wound_site: str = ""              # 手術部位
    wound_condition: str = ""         # 傷口狀況 0-3 分
    behavior: str = ""                # 態度行為 0-5 分
    appetite: str = ""                # 食慾 0-2 分
    defecation: str = ""              # 排便 0-3 分
    urination: str = ""               # 排尿 0-3 分
    pain_score: str = ""              # 疼痛分數 1-4 級
    total_score: str = ""
    medication_or_remark: str = ""    # 給藥（給藥行）/ 註記（紀錄行）


class SurgeryPayload(BaseModel):
    """手術紀錄表完整 payload（flat structure）。"""

    animal: SurgeryAnimalInfo
    surgery: SurgeryHeader
    recorded_by: str = ""              # 記錄人員
    vital_signs: list[VitalSign] = Field(default_factory=list)
    post_op: PostOpObservation
    pain_assessments: list[PainAssessmentRow] = Field(default_factory=list)
    export_date: Optional[str] = None
