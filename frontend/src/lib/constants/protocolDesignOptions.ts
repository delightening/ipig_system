// 計劃書 Design (§4) 選擇欄位的「完整選項」正規清單（顯示端 single source）。
//
// 用於 HTML 唯讀檢視「顯示未選選項」功能（見 docs/design/protocol-unselected-options-spec.md）。
// 每個選項帶 i18n labelKey；注意部分欄位 stored value ≠ i18n key（如麻醉類型、限制類型），
// 故以明確 value→labelKey 對照，避免兩者錯位。
//
// 注意：目前編輯表單（protocol-edit/components/*）仍各自定義選項；全面鋪開時應收斂至此檔
// （DRY single source）。本示範階段暫不改動編輯表單，詳見 spec §6。

export interface ProtocolChoiceOption {
    /** workingContent 內實際儲存的值 */
    value: string
    /** i18n key（完整路徑） */
    labelKey: string
    /** 選填：外部參考連結（目前僅替代方案搜尋平台使用） */
    url?: string
}

export const ANESTHESIA_TYPE_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'survival_surgery', labelKey: 'aup.design.anesthesiaTypes.survival' },
    { value: 'non_survival_surgery', labelKey: 'aup.design.anesthesiaTypes.non_survival' },
    { value: 'gas_only', labelKey: 'aup.design.anesthesiaTypes.gas_only' },
    { value: 'azaperonum_atropine', labelKey: 'aup.design.anesthesiaTypes.azaperonum_atropine' },
]

export const PAIN_CATEGORY_OPTIONS: ProtocolChoiceOption[] = (['B', 'C', 'D', 'E'] as const).map(
    (c) => ({ value: c, labelKey: `aup.design.painCategories.${c}` }),
)

export const RELIEF_MEASURE_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'alternative_painless_procedure', labelKey: 'aup.design.reliefMeasures.alternative_painless_procedure' },
    { value: 'anesthesia_analgesia', labelKey: 'aup.design.reliefMeasures.anesthesia_analgesia' },
    { value: 'humane_euthanasia', labelKey: 'aup.design.reliefMeasures.humane_euthanasia' },
    { value: 'no_relief_with_justification', labelKey: 'aup.design.reliefMeasures.no_relief_with_justification' },
]

export const RESTRICTION_TYPE_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'fasting_before_anesthesia', labelKey: 'aup.design.restrictionTypes.fasting' },
    { value: 'other', labelKey: 'aup.design.restrictionTypes.other' },
]

export const HANDLING_METHOD_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'euthanasia', labelKey: 'aup.design.handlingMethods.euthanasia' },
    { value: 'transfer', labelKey: 'aup.design.handlingMethods.transfer' },
    { value: 'other', labelKey: 'aup.design.handlingMethods.other' },
]

export const EUTHANASIA_TYPE_OPTIONS: ProtocolChoiceOption[] = [
    { value: 'kcl', labelKey: 'aup.design.euthanasiaTypes.kcl' },
    { value: 'electrocution', labelKey: 'aup.design.euthanasiaTypes.electrocution' },
    { value: 'other', labelKey: 'aup.design.euthanasiaTypes.other' },
]
