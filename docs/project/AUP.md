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
    - `other`（其他）
  - UI：顯示原表單選項 1 至 6
- `section1.project_type_other` (string, required if `project_type = other`)

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

### 3.0 計畫摘要 Abstract

**欄位定義**
- `section2.abstract` (text, required on submit)

**字數限制**
- 中文：500–1000 字
- 英文：250–500 字
- 不超過一個 A4 頁面

**撰寫規則**
- 禁止使用醫學專用字彙或艱深科學概念（或需說明清楚）
- 勿透露與專利相關事宜
- 定義所有縮寫字詞
- 需闡述計畫意義及基本理由（以 IACUC 社會公正人士觀點撰寫）

---

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
    - `altbib`（ALTBIB - https://ntp.niehs.nih.gov/whatwestudy/niceatm/altbib）
    - `db_alm`（DB-ALM 動物試驗替代方法資料庫）
    - `re_place`（歐洲動物替代試驗資源平台 - https://www.re-place.be/）
    - `johns_hopkins`（Johns Hopkins 動物試驗替代中心 - http://altweb.jhsph.edu）
    - `taat`（臺灣非動物性替代方法資訊網 TAAT - https://taat.nhri.edu.tw/）
    - `nc3rs_eda`（實驗設計助理 NC3Rs EDA - https://nc3rs.org.uk/）
    - `nc3rs_refinement`（NC3Rs 精緻化技術與策略資料庫 - https://refinementdatabase.org/）
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
- `section2.duplicate.status` (enum, required)
  - 可選值：
    - `no`（否）
    - `not_applicable`（不適用 — 本案件為委託試驗）
    - `yes_continuation`（是 — 本申請人延續性實驗）
    - `yes_duplicate`（是 — 重複他人試驗）

**條件欄位**
- 若 `status = not_applicable`：
  - `section2.duplicate.regulation_basis` (text, required)（法規依據）
- 若 `status = yes_continuation`：
  - `section2.duplicate.previous_iacuc_no` (string, required)（前次核准 IACUC 編號）
- 若 `status = yes_duplicate`：
  - `section2.duplicate.justification` (text, required)（重複之必要性說明）

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

### 3.5.1 特殊照護 Special Care

> 歸屬於 **2.3 Reduction** 的子項目（對應表單 2.3.1）

**欄位定義**
- `section2.special_care.needed` (boolean, required)
- `section2.special_care.description` (text, required if `needed = true`)

---

### 3.5.2 單獨飼養 Single Housing

> 歸屬於 **2.3 Reduction** 的子項目（對應表單 2.3.2）

**欄位定義**
- `section2.single_housing.required` (boolean, required)

**條件欄位**（若 `required = true`）：
- `section2.single_housing.reasons` (array of enum, required, min 1)
  - 可選值：
    - `b1_pregnant_female`（繁殖需求 - 懷孕雌性）
    - `b1_breeding_male`（繁殖需求 - 繁殖雄性）
    - `b1_post_wean`（繁殖需求 - 斷奶後單獨飼養）
    - `b2_post_surgery`（實驗需求 - 術後護理直至康復）
    - `b2_single_in_group`（實驗需求 - 同組僅單隻個體）
    - `b2_metabolic_cage`（實驗需求 - 使用代謝籠 ≤7 天）
    - `b3_aggressive`（PI 意見 - 攻擊性行為）
    - `b3_temporary`（PI 意見 - 暫時使用）
    - `b4_other`（其他）
- `section2.single_housing.metabolic_cage_duration` (string, required if `b2_metabolic_cage` selected)
- `section2.single_housing.monitoring_method` (text, required)（結束單獨飼養評估的監控方式）
- `section2.single_housing.estimated_duration` (string, required)（預計單獨飼養時間）

---

### 3.5.3 試驗後動物再應用 Animal Reuse

> 歸屬於 **2.3 Reduction** 的子項目（對應表單 2.3.3）

**欄位定義**
- `section2.animal_reuse.considered` (boolean, required)
- `section2.animal_reuse.plan` (enum, required if `considered = true`)
  - 可選值：
    - `no_further_procedure`（無進一步操作，後續回歸群養或持續觀察）
    - `partial_procedure_euthanasia`（僅部分操作，後續安樂死）
    - `teaching_purpose`（作為教學或教育訓練用途）
    - `other`（其他，請說明）
- `section2.animal_reuse.plan_other` (text, required if `plan = other`)

---

### 3.6 精緻化原則說明 Refinement

**欄位定義**
- `section2.refinement_description` (text, required on submit)

**內容要求**
需包含：
- 降低動物疼痛、緊迫及不適的具體措施
- 麻醉與止痛措施
- 飼養環境豐富化（如玩具球、鍊條等環境豐富化物件）
- 定期健康觀察方式
- 異常處置流程（包括通報獸醫師的條件）

**系統支援**
- 可提供預設範例文字（可供編輯使用）：
  > 本計畫於動物實驗設計與執行過程中，已充分考量實驗動物福利與精緻化原則（Refinement），以降低動物之疼痛、緊迫及不適情形。實驗操作前將由具相關訓練經驗之人員執行動物處置與監測，並依實驗需求採適當之麻醉與止痛措施，以減輕動物於實驗過程中的不適感。
  >
  > 於飼養管理方面，動物飼養環境依據機構實驗動物照護標準辦理，提供適當空間、通風、溫濕度控制及定期健康觀察，以維持動物良好生理狀態。同時配合機構環境豐富化政策，於飼養欄位內提供玩具球及鍊條等環境豐富化物件，使豬隻可進行探索及互動行為，以降低心理緊迫並促進其自然行為表現。
  >
  > 此外，實驗期間將持續觀察動物之行為與健康狀況，如發現異常或疼痛跡象，將立即通報獸醫師並依建議採取適當處置措施。

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

### 5.4 疼痛緊迫等級評估（對應表單 4.1.3）

> **互動模式**：先單選 Category B–E，再依所選等級展開對應的細項複選清單。

**欄位定義**
- `section4.pain.category` (enum, required)
  - 可選值：`B`, `C`, `D`, `E`
- `section4.pain.category_items` (array of string, required, min 1)
  - 根據所選 `category` 動態顯示對應細項 checkbox：

**Category B** — 不引起疼痛與不適 (No Pain/Distress)
- `b_breeding_no_procedure`（飼養與繁殖，無實驗操作）
- `b_other`（其他，請說明）

**Category C** — 極小的不適，不需用藥緩解
- `c_handling_weighing_transport`（抓取、稱重或運輸動物）
- `c_injection_oral_non_irritant`（注射（肌肉）及口服無刺激性物質）
- `c_animal_marking`（動物標示如刺青或齧齒類動物的耳朵打孔）
- `c_routine_farming`（常規農牧業程序）
- `c_general_anesthesia`（完整的全身麻醉）
- `c_avma_euthanasia`（AVMA 認可的人道安樂死程序）
- `c_other`（其他，請說明）

**Category D** — 有疼痛或不適，須給予適當的藥物緩解
- `d_stress_transport_sedation`（存在潛在壓力運輸，該動物需給予鎮靜劑）
- `d_intubation_under_anesthesia`（麻醉中插管）
- `d_survival_surgery_under_anesthesia`（在全身麻醉下進行存活性手術）
- `d_non_survival_surgery`（全身麻醉下進行非存活性手術）
- `d_non_lethal_drug_exposure`（暴露於不致命性的藥物或化學物下，未對動物造成顯著的身體變化）
- `d_catheter_implantation`（在血管暴露狀況下植入導管）
- `d_blood_draw_perfusion`（在麻醉下放血或進行灌流）
- `d_non_preop_food_water_restrict`（非手術前必要的限食及限水）
- `d_pain_with_analgesia`（任何流程導致明顯的疼痛或不適，但可施以止痛藥物緩解，如減少食慾/活動、觸摸引起不良反應、開放性皮膚病變、膿腫、跛行、結膜炎、角膜浮腫或畏光）
- `d_induced_anatomical_physiological`（誘導解剖學或生理學異常造成的疼痛或緊迫輻射性病痛）
- `d_drug_physiological_damage`（藥物或化學物損害動物體的生理系統）
- `d_eye_skin_irritation_relievable`（眼睛和皮膚刺激性測試所引起的疼痛，該疼痛可以被緩解）
- `d_other`（其他，請說明）

**Category E** — 劇烈疼痛且無法以藥物緩解（需 IACUC + 獸醫師謹慎監督）
- `e_severe_drug_damage_death`（使用藥物或化學物嚴重損害動物生理系統而造成動物死亡、劇烈疼痛或極度緊迫）
- `e_paralytic_without_anesthesia`（未麻醉情形下使用麻痺或肌肉鬆弛劑）
- `e_burn_large_skin_wound`（燒燙傷或大規模皮膚創傷）
- `e_induced_disease`（實驗性誘發疾病，包括代謝干擾和營養性疾病或接觸會引起疾病有毒物質）
- `e_pain_threshold_procedure`（任何會造成接近疼痛閥值且無法以止痛劑解除的操作步驟，如關節炎模式、眼睛/皮膚刺激性試驗、強烈炎症反應模式、視覺剝奪、電擊/加熱試驗等）
- `e_chronic_pain_unrelievable`（突變或患有慢性疼痛的疾病，且無法用止痛藥或適當處置緩解）
- `e_excessive_food_water_restrict`（超出常規術前必要的限食及限水且對動物產生壓力）
- `e_extreme_environment`（暴露於異常或極端環境中情況）
- `e_procedure_may_cause_death`（實驗操作可能會導致動物死亡）
- `e_pain_distress_study`（允許測試項目為疼痛或緊迫的研究，如未經治療就戒斷成癮的藥物或疼痛研究）
- `e_non_avma_euthanasia`（未經 AVMA 認可的安樂死方法）
- `e_other`（其他，請說明）

- `section4.pain.category_item_other_text` (string, required if 對應 `*_other` selected)

---

### 5.5 飲食飲水限制（對應表單 4.1.4）

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

### 5.6 疼痛或痛苦症狀勾選（對應表單 4.1.5）

> 「請勾選此試驗可能會對動物造成的疼痛或痛苦（可複選，請勾選所有適用者）」

**欄位定義**
- `section4.pain.distress_signs` (array of string, required on submit, min 1)
  - 可選值（複選）：
    - `weight_loss`（體重減輕）
    - `reduced_food_water`（食物與水攝取量減少）
    - `dehydration`（脫水／皮膚無彈性／眼眶下陷）
    - `unkempt_fur`（毛髮蓬亂、打結或失去光澤）
    - `isolation_hiding`（自行隔離或躲藏）
    - `self_mutilation`（自殘，如咬肢體）
    - `abnormal_posture`（姿勢或體位異常，如頂頭、背拱）
    - `abnormal_breathing`（呼吸異常，如急促呼吸、張口呼吸、腹式呼吸）
    - `abnormal_activity`（活動量異常，增加或減少）
    - `aggression`（咬人、咆哮或具攻擊性行為）
    - `lacrimation_no_blink`（流淚（包括紅色淚液）、無眨眼反射）
    - `muscle_rigidity_weakness`（肌肉僵硬或無力）
    - `tremor_convulsion`（顫抖、顫動或抽搐）
    - `vocalization`（發出叫聲（哀鳴））
    - `surgical_site_inflammation`（手術部位紅腫或發炎）
    - `teeth_grinding`（磨牙）
    - `other`（其他，請註明）
- `section4.pain.distress_signs_other_text` (string, required if `other` selected)

---

### 5.7 疼痛緩解措施（對應表單 4.1.6）

> 「請說明將採取何種措施以緩解或減輕超過輕微的疼痛或痛苦，或說明為何未採取緩解措施（可複選，請勾選所有適用者）」

**欄位定義**
- `section4.pain.relief_measures` (array of string, required on submit, min 1)
  - 可選值（複選）：
    - `alternative_painless_procedure`（改用不造成疼痛／痛苦的替代程序）
    - `anesthesia_analgesia`（投予麻醉或止痛藥）
    - `humane_euthanasia`（執行人道安樂死）
    - `no_relief_with_justification`（未採取麻醉、止痛或鎮靜劑等緩解措施，但有合理科學依據）

**條件欄位**
- 若選擇 `anesthesia_analgesia`：
  - `section4.pain.relief_drug_name` (string, required)（藥品名稱）
- 若選擇 `no_relief_with_justification`：
  - `section4.pain.no_relief_justification` (text, required)（科學依據說明）

---

### 5.8 實驗終點與人道終點（對應表單 4.1.7）

**欄位定義**
- `section4.endpoints.experimental_endpoint` (text, required)
- `section4.endpoints.humane_endpoint` (text, required)

**系統支援**
- 人道終點可提供預設範例文字（可供編輯使用）：
  > 實驗過程中如果動物體重下降超過原體重的 20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，或其他經獸醫師評估不宜持續實驗之情形，則提早結束實驗，以符合動物福祉。

**驗證規則**
- `humane_endpoint` 必須包含可操作的觸發條件
  - 例如：體重下降比例、食慾、傷口、感染、獸醫判斷

---

### 5.9 動物最終處置（對應表單 4.1.8）

**欄位定義**
- `section4.final_handling.method` (enum, required)
  - 可選值：
    - `euthanasia_kcl_exsanguination`（安樂死：麻醉下 Zoletil®-50 4.4 mg/kg + KCl 注射後放血，依 AD-04-03-00）
    - `euthanasia_electrocution_exsanguination`（安樂死：麻醉下 Zoletil®-50 4.4 mg/kg + 220V 電擊後放血，依 AD-04-03-00）
    - `transfer`（轉移）
    - `other`（其他）

> **SOP 參照**：安樂死程序依「AD-04-03-00 試驗豬隻安樂死規範標準作業程序書」執行。

**條件欄位**
- 若 `method = transfer`：
  - `section4.final_handling.transfer.recipient_name` (string, required)
  - `section4.final_handling.transfer.recipient_org` (string, required)
  - `section4.final_handling.transfer.project_name` (string, required)
- 若 `method = other`：
  - `section4.final_handling.other_text` (text, required)

---

### 5.10 屍體處理

**欄位定義**
- `section4.carcass_disposal.method` (text, required)
  - 可提供預設值：「委由簽約之合格化製廠商進行化製處理」
- `section4.carcass_disposal.vendor_name` (string, optional)
  - 預設廠商：**金海龍生物科技股份有限公司**
- `section4.carcass_disposal.vendor_id` (string, optional)
  - 預設化製廠管編：**P6001213**

---

### 5.11 非醫藥級化學品使用

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

### 5.12 危害性物質

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

### 5.13 管制藥品

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
- 可提供預設範例文字（參考 `TU-03-09-00 試驗豬隻外科手術標準作業程序書`）：
  > 1. 實驗動物術前禁食至少 12 小時，不禁水。
  > 2. 試驗豬隻清洗擦乾後，以畜舒坦（Azeperonum 40 mg/mL）3–5 mg/kg 和 0.03–0.05 mg/kg 阿托平（Atropine® 1 mg/mL）肌肉注射鎮靜，仔細觀察豬隻呼吸頻率。
  > 3. 經 20–30 分鐘後，以 4.4 mg/kg 舒泰-50（Zoletil®-50）肌肉注射誘導麻醉。
  > 4. 經 5–10 分鐘後，將豬隻移至手術檯，以趴姿進行氣管插管，接上氣體麻醉機，以 2–3 L/min 流速氧氣混合 0.5–2% 異氟烷（Isoflurane）維持麻醉。
  > 5. 術前肌肉注射抗生素 Cefazolin 15 mg/kg 及止痛劑 Meloxicam 0.4 mg/kg，並以電剪於術部剃毛。
  > 6. 手術部位消毒：碘酒由中心向外畫圓擦拭，再以 70–75% 酒精同法擦拭，重複三次，最後一次酒精不須擦掉。
  > 依「TU-03-09-00 試驗豬隻外科手術標準作業程序書」進行。

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
- 麻醉深度評估（依麻醉深度、呼吸頻率調整氧氣流速與麻醉氣體濃度）
- 必要時輸液
- 記錄頻率：**每 30 分鐘**記錄一次心跳、呼吸及體溫
- 引用 SOP：`TU-03-09-00 試驗豬隻外科手術標準作業程序書`

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
- 傷口護理（依術後狀況）
- 疼痛評估：術後 7 日內每日進行
- 止痛與抗生素方案（依豬隻狀況給予）
- 異常處置與獸醫介入
- 引用 SOP：`TU-03-09-00 試驗豬隻外科手術標準作業程序書`

**表格式用藥**
- `section6.drugs` (array of object, required if surgery, min 1)
  - 欄位：
    - `drug_name` (string, required)
    - `dose` (string, required)
    - `route` (enum, required)：`IM`, `IV`, `PO`, `inhalation`, `SC`, `other`
    - `frequency` (string, required)
    - `purpose` (string, required)
  - UI：可新增多列

**標準手術用藥參考表**（來源：AD-04-01-01F 動物試驗研究計畫書範例）

| 藥品名稱 | 劑量 | 途徑 | 頻率 | 用途 |
|---------|------|------|------|------|
| Atropine（阿托品） | 1 mg/mL；0.03–0.05 mg/kg | IM | 術前 1 次 | 麻醉誘導前導 |
| 畜舒坦（Azaperonum / Azeperonum 40 mg/mL） | 3–5 mg/kg | IM | 術前 1 次 | 麻醉誘導鎮靜 |
| 舒泰-50（Zoletil®-50） | 3–5 mg/kg（安樂死：4.4 mg/kg） | IM | 術前 1 次 | 麻醉誘導 |
| Cefazolin | 15–30 mg/kg | IM | 術前 1 次 / 術後 SID | 術前術後抗生素 |
| Meloxicam | 0.1–0.4 mg/kg | IM | 術前 1 次 / 術後 SID | 術前術後止痛 |
| Isoflurane（異氟烷） | 0.5–2%（O₂ 2–3 L/min） | 吸入 | 術中持續 | 麻醉維持 |
| Ketoprofen | 1–3 mg/kg | IM | SID | 術後止痛 |
| Penicillin（青黴素） | 0.1–1 mL/kg | IM | SID | 術後抗生素 |
| Cephalexin（頭孢力新） | 30–60 mg/kg | PO | BID | 術後抗生素 |
| Amoxicillin（阿莫西林） | 20–40 mg/kg | PO | BID | 術後抗生素 |
| Meloxicam | 0.1–0.4 mg/kg | PO | SID | 術後止痛 |

> 以上為參考預設值，實際用藥依計畫書內容填寫。

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
- `animal_sources`：動物來源資料

詳細資料庫結構請參考主規格文件。

---

## 14. 相關 SOP 文件參照

以下 SOP 文件在 AUP 表單中被引用，系統應在對應欄位提供連結或參照說明：

| SOP 編號 | 名稱 | 相關章節 |
|---------|------|---------|
| `TU-03-09-00` | 試驗豬隻外科手術標準作業程序書 | 術前準備（7.3）、術中監控（7.6）、術後照護（7.9） |
| `AD-04-03-00` | 試驗豬隻安樂死規範標準作業程序書 | 動物最終處置（5.7） |

---

## 15. 表單來源與版本

| 欄位 | 內容 |
|------|------|
| 表單編號 | `AD-04-01-01F` |
| 表單名稱 | 動物試驗研究計畫書（Animal Use Protocol） |
| 現行版本 | F 版 |
| 管理機構 | IACUC（實驗動物照護及使用委員會） |
