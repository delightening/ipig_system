// AUP 計劃類型 / 計畫種類的選項 key —— 單一真相（source of truth）。
//
// 前端（ResearchBasicFields 選單、validation）直接 import 此處。
// 後端 print-pdf adapter（services/print-pdf/adapters/aup_protocol.py）的
// ProjectClassification 比對 key 必須與此一致；由 vitest parity 測試
// (aupProjectOptions.parity.test.ts) 在 CI 鎖死，避免再發生 #608 那種
// 前端/後端 key 悄悄漂移（漏勾 PDF 框）的 bug。
//
// 對應 i18n：aup.projectTypes.* / aup.projectCategories.*（zh-TW.json / en.json）。

export const PROJECT_TYPE_KEYS = [
    '1_basic_research',
    '2_applied_research',
    '3_pre_market_testing',
    '4_educational',
    '5_biologics_manufacturing',
    '6_other',
] as const

export const PROJECT_CATEGORY_KEYS = [
    '1_medical',
    '2_agricultural',
    '3_drugs_vaccines',
    '4_supplements',
    '5_food',
    '6_toxics_chemicals',
    '7_medical_materials',
    '8_pesticide',
    '9_animal_drugs_vaccines',
    '10_animal_supplements_feed',
    '11_cosmetics',
    '12_other',
] as const

/** 「其他」哨兵 key（選此項時顯示補充說明輸入框）。 */
export const PROJECT_TYPE_OTHER = '6_other'
export const PROJECT_CATEGORY_OTHER = '12_other'
