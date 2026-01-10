# AUP 提交與審查系統規格書

## 1. 概述

本文件定義動物使用計畫（Animal Use Protocol, AUP）提交與審查系統的資料結構、驗證規則與 UI 規格。

---

## 2. 章節 1：研究資料 Study Information

### 2.1 GLP 屬性

**欄位定義**
- `section1.is_glp` (boolean, required on submit)
  - UI：單選按鈕
    - GLP
    - 非 GLP

**驗證規則**
- 若 `is_glp = true`，則 `section1.registration_authorities` 必填

---

### 2.2 研究名稱與編號

**欄位定義**
- `section1.study_title` (string, required on submit)
  - 預設值：從 `cover.study_title_zh` 同步，或支援雙語結構
- `section1.apply_study_number` (string, required on submit)
- `section1.iacuc_apply_no` (string, optional in draft, required if policy demands)
- `section1.iacuc_approval_no` (string, readonly)
  - 從 `cover` 衍生，審核通過後自動填入
- `section1.apply_date` (date, required on submit)
- `section1.approval_date` (date, staff only, required when approved)

**一致性規則**
- `cover.study_title_zh` 必須等於 `section1.study_title_zh`
- `cover.testing_facility_name` 必須等於 `section1.facility.title`（可由系統同步）

---

### 2.3 計畫主持人（PI）資料

**欄位定義**
- `section1.pi.name` (string, required)
- `section1.pi.phone` (string, required, pattern 可配置)
- `section1.pi.email` (string, required, email format)
- `section1.pi.address` (string, required, max 300)

**委託單位（Sponsor）資料**
- `section1.sponsor.name` (string, required)
- `section1.sponsor.contact_person` (string, required)
- `section1.sponsor.contact_phone` (string, required)
- `section1.sponsor.contact_email` (string, required, email format)

**UI 規則**
- PI 可能為系統使用者，可從使用者清單帶入
- Sponsor 名稱可從使用者 `organization` 欄位帶入
- 若 Sponsor 為外部單位，支援手動輸入

---

### 2.4 專案主持人（SD）資料

**欄位定義**
- `section1.sd.name` (string, required)
- `section1.sd.email` (string, required, email format)

**規則**
- SD 必須屬於試驗機構端使用者或指定人員

---

### 2.5 試驗機構與動物飼養場所

**欄位定義**
- `section1.facility.title` (string, required)
- `section1.facility.address` (string, required)
- `section1.animal_environment.housing_location` (string, required)

**UI 規則**
- 若系統已有機構設定，可一鍵套用

---

### 2.6 試驗時程 Valid Period

**欄位定義**
- `section1.period.start_date` (date, required)
- `section1.period.end_date` (date, required)

**驗證規則**
- `end_date` 必須大於 `start_date`
- 若 `end_date` 超過機構允許最長期限，提示需分段或續展

---

### 2.7 計畫類型與種類

**計畫類型**
- `section1.project_type` (enum, required)
  - 可選值：
    - `basic_research`（基礎研究）
    - `applied_research`（應用研究）
    - `pre_market_testing`（上市前試驗）
    - `teaching_training`（教學訓練）
    - `biologics_manufacturing`（生物製劑製造）
  - UI：顯示原表單選項 1 至 5

**計畫種類**
- `section1.project_category` (enum, required)
  - 可選值：
    - `medical`（醫藥）
    - `agricultural`（農業）
    - `drug_herbal`（藥用植物）
    - `health_food`（健康食品）
    - `food`（食品）
    - `toxic_chemical`（毒性化學物質）
    - `medical_device`（醫療器材）
    - `pesticide`（農藥）
    - `veterinary_drug_vaccine`（動物用藥品疫苗）
    - `animal_health_feed_additive`（動物保健飼料添加物）
    - `cosmetic_drug`（化粧品）
    - `other`（其他）
- `section1.project_category_other` (string, required if `project_category = other`)

---

### 2.8 試驗物質類型與技術類別

**試驗物質類型**
- `section1.test_item_type` (enum, required)
  - 可選值：
    - `na`（不適用）
    - `drug`（藥品）
    - `pesticide_env_drug`（農藥環境用藥）
    - `veterinary_drug`（動物用藥品）
    - `cosmetic`（化粧品）
    - `food_additive`（食品添加物）
    - `feed_additive`（飼料添加物）
    - `industrial_chemical`（工業化學品）
    - `medical_product`（醫療產品）
    - `other`（其他）
- `section1.test_item_type_other` (string, required if `test_item_type = other`)

**技術類別**
- `section1.tech_categories` (array of enum, required, min 1)
  - 可複選：
    - `physical_chemical`（物理化學）
    - `toxicity`（毒性）
    - `mutagenicity`（致突變性）
    - `eco_toxicity`（生態毒性）
    - `water_soil_air_behavior`（水、土壤、空氣行為）
    - `residue`（殘留）
    - `ecosystem_simulation`（生態系統模擬）
    - `analytical_clinical_chemistry`（分析臨床化學）
    - `other_biosafety`（其他生物安全）
    - `other_biocompatibility`（其他生物相容性）

---

### 2.9 經費來源

**欄位定義**
- `section1.funding_sources` (array of enum, required, min 1)
  - 可選值：
    - `agriculture_ministry`（農業部）
    - `mohw`（衛生福利部）
    - `nstc`（國家科學及技術委員會）
    - `education_ministry`（教育部）
    - `epa`（環境部）
    - `other`（其他）
- `section1.funding_other` (string, required if `other` selected)

---

### 2.10 預定申請註冊之權責機關（GLP 適用）

**欄位定義**
- `section1.registration_authorities` (array of enum, required if `is_glp = true`)
  - 可選值：
    - `FDA`（美國食品藥物管理局）
    - `CE`（歐盟）
    - `TFDA`（台灣食品藥物管理署）
    - `CFDA`（中國國家食品藥品監督管理總局）
    - `other`（其他）
- `section1.registration_authority_other` (string, required if `other` selected)

---

## 3. 章節 2：研究目的 Study Purpose

### 3.1 研究之目的及重要性

**欄位定義**
- `section2.purpose_significance` (text, required on submit, min 50, max 5000)

**UI 提示**
內容需包含：
- 研究背景
- 臨床或科學重要性
- 預期成果

---

### 3.2 Replacement 替代原則說明

**欄位定義**
- `section2.replacement_rationale` (text, required on submit)

**內容要求**
必須描述：
- 為何需要活體動物
- 為何非動物模型不足
- 物種選擇的解剖、生理、遺傳理由
- 最低系統發生學層級考量

---

### 3.3 非動物替代方案搜尋紀錄

**欄位定義**
- `section2.alt_search.platforms` (array of enum, required, min 1)
  - 可選值：
    - `altbib`
    - `db_alm`
    - `re_place`
    - `other`
- `section2.alt_search.other_name` (string, required if `other` selected)
- `section2.alt_search.keywords` (string, required on submit, max 200)
- `section2.alt_search.conclusion` (text, required on submit)

**UI 規則**
- 提供連結按鈕與關鍵字欄位
- 可上傳搜尋截圖作為附件（type: `reference_paper` 或 `other`）

---

### 3.4 是否重複他人試驗

**欄位定義**
- `section2.duplicate_experiment` (boolean, required)
- `section2.duplicate_justification` (text, required if `duplicate_experiment = true`)

---

### 3.5 Reduction 減量原則

**欄位定義**
- `section2.reduction_design` (text, required on submit)

**內容要求**
需包含：
- 分組方法
- 統計假設與樣本數估算
- 納入排除標準
- 減少變異的方法
- 引用之指南或文獻

**結構化欄位（建議）**
- `section2.sample_size_calculation.method` (enum, optional)
  - 可選值：
    - `power_analysis`（統計檢定力分析）
    - `literature_based`（文獻基礎）
    - `pilot_data`（先導數據）
    - `other`（其他）
- `section2.sample_size_calculation.details` (text, optional)

**分組計畫表**
- `section2.grouping_plan` (table, required on submit)
  - 欄位：
    - `group_name` (string)
    - `n` (integer, min 1)
    - `treatment` (text)
    - `timepoints` (text)

**驗證規則**
- 所有 `group.n` 加總需等於 `section7.total_animals`（若有定義）

---

## 4. 章節 3：試驗物質與對照物質 Testing and Control Item

### 4.1 是否使用試驗物質於動物

**欄位定義**
- `section3.use_test_item` (boolean, required)

---

### 4.2 試驗物質清單 Test Items

**欄位定義**
- `section3.test_items` (array of object, required if `use_test_item = true`, min 1)

**每筆欄位**
- `name` (string, required)
- `lot_no` (string, optional)
- `expiry_date` (date, optional)
- `is_sterile` (boolean, required)
- `purpose` (text, required)
- `storage_conditions` (text, required)
- `concentration` (string, optional)
- `form` (enum, optional)
  - 可選值：`liquid`, `solid`, `device`, `implant`, `other`
- `hazard_classification` (text, optional)

**UI 規則**
- 可新增多列
- 欄位不足允許加列

---

### 4.3 對照物質清單 Control Items

**欄位定義**
- `section3.control_items` (array of object, required on submit, min 1)
  - 若研究無對照，可填寫一筆：`name = "N/A"`, `purpose = "N/A"`
  - 每筆欄位同 `test_items`

**一致性規則**
- 若 control item 也是試驗物質之一，需標記關係：
  - `section3.control_items[x].is_sham` (boolean, optional)
  - `section3.control_items[x].is_vehicle` (boolean, optional)

---

## 5. 章節 4：研究設計與方法 Study Design and Methods

### 5.1 是否於麻醉下進行試驗

**欄位定義**
- `section4.anesthesia.is_under_anesthesia` (boolean, required)

**條件欄位**
- 若 `is_under_anesthesia = true`，則需填：
  - `section4.anesthesia.plan_type` (enum, required)
    - 可選值：
      - `survival_surgery`（存活性手術）
      - `non_survival_surgery`（非存活性手術）
      - `anesthesia_only_no_surgery`（僅麻醉，無手術）

**驗證規則**
- 若選擇 `survival_surgery` 或 `non_survival_surgery`，則章節 6（手術計畫書）必填

---

### 5.2 麻醉方案選項

**欄位定義**
- `section4.anesthesia.premed_option` (enum, required if `is_under_anesthesia = true`)
  - 可選值：
    - `inhalation_isoflurane_only`（僅吸入性異氟烷）
    - `azeperonum_atropine_then_isoflurane`（阿澤哌隆+阿托品後異氟烷）
    - `custom`（自訂）
- `section4.anesthesia.custom_text` (text, required if `premed_option = custom`)

**規則**
- 若 `is_under_anesthesia = false`，則上述欄位為 `null`

---

### 5.3 動物試驗流程描述

**欄位定義**
- `section4.procedures` (text, required on submit, min 100)

**內容要求**
必須包含：
- 投予物質與途徑
- 途徑選擇理由
- 採血次數與每次量
- 影像檢查類型與頻率
- 保定方式與頻率
- 各時間點操作
- 若有手術內容，需提示改填章節 6

**結構化欄位（建議）**

**途徑選擇理由**
- `section4.route_justifications` (array of object, optional)
  - 欄位：
    - `substance_name` (string)
    - `route` (enum)：`IM`, `IV`, `PO`, `SC`, `ID`, `inhalation`, `topical`, `implant`, `other`
    - `justification` (text)

**採血計畫**
- `section4.blood_withdrawals` (array of object, optional)
  - 欄位：
    - `timepoint` (string)
    - `volume_ml` (number)
    - `frequency` (string)
    - `site` (string)
    - `notes` (text)

**影像檢查**
- `section4.imaging` (array of object, optional)
  - 欄位：
    - `modality` (enum)：`CT`, `MRI`, `Xray`, `ultrasound`, `fluoroscopy`, `endoscopy`, `other`
    - `timepoint` (string)
    - `anesthesia_required` (boolean)
    - `notes` (text)

**保定方式**
- `section4.restraint` (array of object, optional)
  - 欄位：
    - `method` (enum)：`manual`, `sling`, `cage`, `sedation`, `other`
    - `duration_min` (number)
    - `frequency` (string)
    - `welfare_notes` (text)

---

### 5.4 疼痛緊迫等級

**欄位定義**
- `section4.pain_category` (enum, required)
  - 可選值：`B`, `C`, `D`, `E`

**驗證規則**
- 若選擇 `D` 或 `E`，則必須描述疼痛處置策略：
  - `section4.pain_management_plan` (text, required)
- 若選擇 `E`，需提供不給止痛、麻醉、鎮定之科學理由：
  - `section4.no_analgesia_justification` (text, required)

---

### 5.5 飲食飲水限制

**欄位定義**
- `section4.restrictions.is_restricted` (boolean, required)

**條件欄位**
- 若 `is_restricted = true`：
  - `section4.restrictions.types` (array of enum, min 1)
    - 可選值：
      - `fast_before_anesthesia`（麻醉前禁食）
      - `water_restriction`（飲水限制）
      - `diet_restriction`（飲食限制）
      - `other`（其他）
  - `section4.restrictions.other_text` (text, required if `other` selected)

---

### 5.6 實驗終點與人道終點

**欄位定義**
- `section4.endpoints.experimental_endpoint` (text, required)
- `section4.endpoints.humane_endpoint` (text, required)

**規則**
- 系統內建預設文字可引用，但不可完全空白

**驗證規則**
- `humane_endpoint` 必須包含可操作的觸發條件
  - 例如：體重下降比例、食慾、傷口、感染、獸醫判斷

---

### 5.7 動物最終處置

**欄位定義**
- `section4.final_handling.method` (enum, required)
  - 可選值：
    - `euthanasia_kcl_exsanguination`（安樂死：KCl + 放血）
    - `euthanasia_electrocution_exsanguination`（安樂死：電擊 + 放血）
    - `transfer`（轉移）
    - `other`（其他）

**條件欄位**
- 若 `method = transfer`：
  - `section4.final_handling.transfer.recipient_name` (string, required)
  - `section4.final_handling.transfer.recipient_org` (string, required)
  - `section4.final_handling.transfer.project_name` (string, required)
- 若 `method = other`：
  - `section4.final_handling.other_text` (text, required)

---

### 5.8 屍體處理

**欄位定義**
- `section4.carcass_disposal.method` (text, required)
  - 可提供預設值：「委由合格化製廠商處理」
- `section4.carcass_disposal.vendor_name` (string, optional)
- `section4.carcass_disposal.vendor_id` (string, optional)

---

### 5.9 非醫藥級化學品使用

**欄位定義**
- `section4.non_pharma_grade.used` (boolean, required)

**條件欄位**
- 若 `used = true`：
  - `section4.non_pharma_grade.description` (text, required)
    - 必須包含：
      - 物質性質
      - 安全性
      - 科學理由

---

### 5.10 危害性物質

**欄位定義**
- `section4.hazards.used` (boolean, required)

**條件欄位**
- 若 `used = true`：
  - `section4.hazards.materials` (array of object, required, min 1)
    - 欄位：
      - `type` (enum)：`biological`, `radioactive`, `hazardous_chemical_drug`
      - `agent_name` (string, required)
      - `amount` (string, required)
  - `section4.hazards.waste_disposal_method` (text, required)
  - `section4.hazards.operation_location_method` (text, required)
  - `section4.hazards.protection_measures` (text, required)
  - `section4.hazards.waste_and_carcass_disposal` (text, required)

**附件要求**
- `hazard_certificate` 至少 1 份，或由 staff 勾選豁免原因

---

### 5.11 管制藥品

**欄位定義**
- `section4.controlled_substances.used` (boolean, required)

**條件欄位**
- 若 `used = true`：
  - `section4.controlled_substances.items` (array of object, required, min 1)
    - 欄位：
      - `drug_name` (string, required)
      - `approval_no` (string, required)
      - `amount` (string, required)
      - `authorized_person` (string, required)

---

## 6. 章節 5：相關規範及參考文獻 Guidelines and References

### 6.1 規範與文獻

**欄位定義**
- `section5.guidelines` (text, required on submit)

**內容要求**
需包含：
- 法源依據
- 適用指南或標準
- 參考文獻列表
- 若無法源依據，必須至少提供參考文獻

**結構化欄位（建議）**
- `section5.references` (array of object, optional)
  - 欄位：
    - `citation` (string, required)
    - `url` (string, optional)
    - `attachment_id` (optional)

**驗證規則**
- `guidelines` 不得為空白或僅 "N/A"
- 若為 "N/A"，必須至少提供 `references` 一筆

---

## 7. 章節 6：手術計畫書 Animal Surgical Plan

### 7.1 條件必填規則

若 `section4.anesthesia.plan_type` 為 `survival_surgery` 或 `non_survival_surgery`，則本章所有標記為「必填」的欄位必填。

---

### 7.2 手術種類

**欄位定義**
- `section6.surgery_type` (enum, required if surgery)
  - 可選值：
    - `survival`（存活性手術）
    - `non_survival`（非存活性手術）

---

### 7.3 術前準備 Preoperative Preparation

**欄位定義**
- `section6.preop_preparation` (text, required if surgery)

**系統支援**
- 可提供預設範例文字：
  - 禁食禁水
  - 鎮靜誘導
  - 插管
  - 麻醉維持
  - 術前抗生素止痛
  - 引用 SOP

---

### 7.4 無菌措施 Aseptic Technique

**欄位定義**
- `section6.aseptic.techniques` (array of enum, required if surgery, min 1)
  - 可選值：
    - `surgical_site_disinfection`（手術部位消毒）
    - `instrument_disinfection`（器械消毒）
    - `sterilized_gown_gloves`（無菌衣手套）
    - `sterilized_drapes`（無菌布單）
    - `surgical_hand_disinfection`（手術手部消毒）

---

### 7.5 手術內容說明 Surgery Description

**欄位定義**
- `section6.surgery_description` (text, required if surgery)

**內容要求**
必須包含：
- 手術部位
- 手術方法
- 預估切口長度
- 止血方式
- 縫合方式
- 植入物或器材
- 術中影像或導引

**結構化欄位（建議）**
- `section6.surgery_steps` (array of object, optional)
  - 欄位：
    - `step_no` (integer)
    - `description` (text)
    - `estimated_duration_min` (number)
    - `key_risks` (text)

---

### 7.6 術中監控 Perioperative Monitoring

**欄位定義**
- `section6.monitoring` (text, required if surgery)

**內容要求**
需包含：
- 心跳、呼吸、體溫監視器
- 保溫
- 麻醉深度評估
- 必要時輸液
- 引用 SOP

---

### 7.7 存活性手術術後影響

**欄位定義**
- `section6.postop_expected_impact` (text, required if `surgery_type = survival`)

---

### 7.8 多次手術

**欄位定義**
- `section6.multiple_surgeries.used` (boolean, required if surgery)

**條件欄位**
- 若 `used = true`：
  - `section6.multiple_surgeries.number` (integer, required, min 2)
  - `section6.multiple_surgeries.reason` (text, required)

---

### 7.9 術後照護與止痛

**欄位定義**
- `section6.postop_care` (text, required if `surgery_type = survival`)

**內容要求**
需包含：
- 每日健康評估
- 傷口護理
- 疼痛評估頻率與量表
- 止痛與抗生素方案
- 異常處置與獸醫介入

**表格式用藥**
- `section6.drugs` (array of object, required if surgery, min 1)
  - 欄位：
    - `drug_name` (string, required)
    - `dose` (string, required)
    - `route` (enum, required)：`IM`, `IV`, `PO`, `inhalation`, `SC`, `other`
    - `frequency` (string, required)
    - `purpose` (string, required)
  - UI：可新增多列

**驗證規則**
- 若 `section4.pain_category` 為 `D` 或 `E`，則 `drugs` 需包含至少一個止痛、鎮定或麻醉相關項目
- 若手術類型為 `survival`，則需至少一個術後止痛（postop analgesic）項目，或提供醫學理由

---

### 7.10 預期實驗結束之時機

**欄位定義**
- `section6.expected_end_point` (text, required if surgery)

---

## 8. 章節 7：實驗動物資料 Animal Information

### 8.1 動物清單

**欄位定義**
- `section7.animals` (array of object, required on submit, min 1)

**每筆欄位**
- `species` (enum, required)
  - 可選值：
    - `pig_minipig`（迷你豬）
    - `pig_white`（白豬）
    - `other`（其他）
- `other_species_text` (string, required if `species = other`)
- `sex` (enum, required)
  - 可選值：`male`, `female`, `any`
- `number` (integer, required, min 1)
- `age_range_months` (string, required, allow "不限")
- `weight_range_kg` (string, required, allow "不限")
- `animal_source` (enum, required)
  - 可選值：參考 `pig_sources` 表（如 `TAITUNG`, `QINGXIN`, `PIGMODEL`, `PINGSHUN`）
  - 或 `other`
- `animal_source_other` (text, required if `animal_source = other`)
- `housing_location` (string, required)

**匯總欄位**
- `section7.total_animals` (integer, computed)
  - 計算方式：`sum(animals.number)`

**驗證規則**
- 若 `species = pig_white` 且研究期間超過 3 個月，則提示建議使用 `pig_minipig`
- 提示不阻擋送審，但需使用者確認（acknowledgment）

---

## 9. 章節 8：試驗人員資料 Personnel Working on Animal Study

### 9.1 人員清單

**欄位定義**
- `section8.personnel` (array of object, required on submit, min 1)

**每筆欄位**
- `name` (string, required)
- `position` (string, required)
- `roles` (array of enum, required, min 1)
  - 可選值：
    - `a_supervision`（監督）
    - `b_animal_care`（動物照護）
    - `c_restraint`（保定）
    - `d_anesthesia_analgesia`（麻醉止痛）
    - `e_surgery`（手術）
    - `f_surgery_assistance`（手術協助）
    - `g_monitoring`（監控）
    - `h_euthanasia`（安樂死）
    - `i_other`（其他）
- `roles_other_text` (string, required if `i_other` selected)
- `years_experience` (number, required, min 0)
- `trainings` (array of object, required, min 1)
  - 欄位：
    - `code` (enum, required)
      - 可選值：
        - `A_iacuc_member_training`（IACUC 委員訓練）
        - `B_iacuc_education_seminar`（IACUC 教育研習）
        - `C_radiation_safety`（輻射安全）
        - `D_biomed_industry_livestock_training`（生醫產業畜牧訓練）
        - `E_animal_law_care_management`（動物法規照護管理）
    - `certificate_no` (string, optional)
    - `received_date` (date, optional)

---

### 9.2 驗證規則

**手術相關人員訓練要求**
- 若 `roles` 包含 `e_surgery` 或 `f_surgery_assistance`，則此人員必須具備至少一個相關訓練：
  - `A_iacuc_member_training` 或
  - `D_biomed_industry_livestock_training` 或
  - `E_animal_law_care_management`

**輻射安全訓練要求**
- 若研究包含放射線（`section4.hazards.materials[].type = radioactive`），則 `roles` 中參與者需包含訓練 `C_radiation_safety`

**管制藥品授權人員要求**
- 若 `section4.controlled_substances.used = true`，則需指定 `authorized_person`，且此人員必須存在於 `personnel` 名單中

---

## 10. 資料驗證與提交規則

### 10.1 提交前驗證

所有章節在提交前需通過以下驗證：
1. 必填欄位完整性檢查
2. 資料格式驗證（email、日期、數字範圍等）
3. 條件必填欄位驗證
4. 跨章節一致性檢查
5. 業務邏輯驗證（如動物數量、訓練要求等）

### 10.2 草稿儲存

- 所有章節支援草稿儲存
- 草稿狀態下，部分必填欄位可為空
- 提交時進行完整驗證

### 10.3 版本控制

- 每次提交建立新版本
- 審查過程中的修改記錄於版本歷史
- 已審核通過的版本不可修改，需建立新版本進行變更

---

## 11. 附件管理

### 11.1 附件類型

- `reference_paper`（參考文獻）
- `hazard_certificate`（危害性物質證書）
- `other`（其他）

### 11.2 附件上傳規則

- 支援多檔案上傳
- 檔案大小限制：依系統設定
- 允許的檔案格式：PDF、DOC、DOCX、JPG、PNG 等
- 附件與特定章節或欄位關聯

---

## 12. 審查流程整合

### 12.1 狀態管理

AUP 狀態與 `protocols` 表狀態同步：
- `DRAFT`：草稿中
- `SUBMITTED`：已提交
- `UNDER_REVIEW`：審查中
- `APPROVED`：已核准
- `REJECTED`：已駁回
- `REVISION_REQUESTED`：要求修正

### 12.2 審查意見

- IACUC_STAFF 可於各章節添加審查意見
- 意見可標記為「必須修正」或「建議修正」
- PI 可回應審查意見並提交修正版本

---

## 13. 資料模型參考

本文件定義的欄位結構應對應至以下資料表：
- `protocols`：主要計畫資料
- `protocol_sections`：章節資料（JSON 結構）
- `protocol_attachments`：附件資料
- `protocol_reviews`：審查意見
- `users`：使用者資料（PI、SD、人員清單）
- `pig_sources`：豬隻來源資料

詳細資料庫結構請參考主規格文件。
