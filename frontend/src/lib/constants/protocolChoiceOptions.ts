// 計劃書 HTML 檢視「顯示未選選項」共用：通用 helper + 各 section（Design 以外）選項常數。
//
// Design 段的 enum 選項在 protocolDesignOptions.ts；本檔放跨 section 通用的
// 是/否選項與 selected 推導 helper，以及 Surgery / Guidelines 等的選項清單。
// 詳見 docs/design/protocol-unselected-options-spec.md。
//
// 注意：編輯表單（protocol-edit/*）目前仍各自定義選項，與此處暫時重複；
// 全面鋪開後應收斂為單一來源（spec §6 follow-up）。

import type { ProtocolChoiceOption } from './protocolDesignOptions'

export type { ProtocolChoiceOption }

// 是/否 boolean 欄位共用選項
export const YES_NO_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'yes', labelKey: 'common.yes' },
    { value: 'no', labelKey: 'common.no' },
]

/** boolean → ChoiceList selectedValues（null/undefined → 兩項皆未選，對齊 spec Q2） */
export const boolSelected = (v: boolean | null | undefined): string[] =>
    v === true ? ['yes'] : v === false ? ['no'] : []

/** 單選 enum → ChoiceList selectedValues */
export const oneOf = (v: string | null | undefined): string[] => (v ? [v] : [])

// ── Surgery (§6) ──────────────────────────────
export const SURGERY_ASEPTIC_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'surgical_site_disinfection', labelKey: 'aup.surgery.asepticTechniques.surgical_site_disinfection' },
    { value: 'instrument_disinfection', labelKey: 'aup.surgery.asepticTechniques.instrument_disinfection' },
    { value: 'sterilized_gowns_gloves', labelKey: 'aup.surgery.asepticTechniques.sterilized_gowns_gloves' },
    { value: 'sterilized_drapes', labelKey: 'aup.surgery.asepticTechniques.sterilized_drapes' },
    { value: 'surgical_hand_disinfection', labelKey: 'aup.surgery.asepticTechniques.surgical_hand_disinfection' },
]

export const SURGERY_POSTOP_TYPE_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'orthopedic', labelKey: 'aup.surgery.postOpTypes.orthopedic' },
    { value: 'non_orthopedic', labelKey: 'aup.surgery.postOpTypes.non_orthopedic' },
]

// ── Guidelines (§5) 文獻資料庫 A–L（label 走 aup.guidelines.databases.{code}） ──
export const GUIDELINE_DATABASE_CODES = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'] as const

// ── Purpose (§2) ──────────────────────────────
// 2.2.2 替代方案搜尋平台（複選；value→個別 labelKey，非群組）
export const ALT_SEARCH_PLATFORM_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'altbib', labelKey: 'aup.purpose.altbibLabel', url: 'https://ntp.niehs.nih.gov/whatwestudy/niceatm/altbib' },
    { value: 'db_alm', labelKey: 'aup.purpose.dbAlmLabel', url: 'https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/EURL-ECVAM/datasets/DBALM/LATEST/online/dbalm.html' },
    { value: 're_place', labelKey: 'aup.purpose.rePlaceLabel', url: 'https://www.re-place.be/' },
    { value: 'johns_hopkins', labelKey: 'aup.purpose.johnsHopkinsLabel', url: 'http://altweb.jhsph.edu/resources/searchalt/searchaltdata.html' },
    { value: 'taat', labelKey: 'aup.purpose.taatLabel', url: 'https://taat.nhri.edu.tw/' },
    { value: 'nc3rs_eda', labelKey: 'aup.purpose.nc3rsEdaLabel', url: 'https://nc3rs.org.uk/' },
    { value: 'nc3rs_refinement', labelKey: 'aup.purpose.nc3rsRefinementLabel', url: 'https://refinementdatabase.org/' },
    { value: 'other', labelKey: 'aup.purpose.otherPlatformLabel' },
]

// 2.2.3 重複試驗狀態（單選）
export const DUPLICATE_STATUS_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'no', labelKey: 'aup.purpose.duplicateStatusOptions.no' },
    { value: 'not_applicable', labelKey: 'aup.purpose.duplicateStatusOptions.not_applicable' },
    { value: 'yes_continuation', labelKey: 'aup.purpose.duplicateStatusOptions.yes_continuation' },
    { value: 'yes_duplicate', labelKey: 'aup.purpose.duplicateStatusOptions.yes_duplicate' },
]

// 2.3.2 單獨飼養原因（複選）
export const SINGLE_HOUSING_REASON_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'b1_pregnant_female', labelKey: 'aup.purpose.singleHousingReasonOptions.b1_pregnant_female' },
    { value: 'b1_breeding_male', labelKey: 'aup.purpose.singleHousingReasonOptions.b1_breeding_male' },
    { value: 'b1_post_wean', labelKey: 'aup.purpose.singleHousingReasonOptions.b1_post_wean' },
    { value: 'b2_post_surgery', labelKey: 'aup.purpose.singleHousingReasonOptions.b2_post_surgery' },
    { value: 'b2_single_in_group', labelKey: 'aup.purpose.singleHousingReasonOptions.b2_single_in_group' },
    { value: 'b2_metabolic_cage', labelKey: 'aup.purpose.singleHousingReasonOptions.b2_metabolic_cage' },
    { value: 'b3_aggressive', labelKey: 'aup.purpose.singleHousingReasonOptions.b3_aggressive' },
    { value: 'b3_temporary', labelKey: 'aup.purpose.singleHousingReasonOptions.b3_temporary' },
    { value: 'b4_other', labelKey: 'aup.purpose.singleHousingReasonOptions.b4_other' },
]

// 2.3.3 動物再應用計畫（單選）
export const ANIMAL_REUSE_PLAN_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'no_further_procedure', labelKey: 'aup.purpose.animalReusePlanOptions.no_further_procedure' },
    { value: 'partial_procedure_euthanasia', labelKey: 'aup.purpose.animalReusePlanOptions.partial_procedure_euthanasia' },
    { value: 'teaching_purpose', labelKey: 'aup.purpose.animalReusePlanOptions.teaching_purpose' },
    { value: 'deferred', labelKey: 'aup.purpose.animalReusePlanOptions.deferred' },
    { value: 'other', labelKey: 'aup.purpose.animalReusePlanOptions.other' },
]
