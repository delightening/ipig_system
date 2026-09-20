// 施打/用藥結構化選項（GLP/AAALAC 用藥分類 + 施打途徑）
// value = 儲存標準碼（與後端 jsonb_validation TREATMENT_CATEGORIES / TREATMENT_ROUTES 對齊）
// labelKey = 顯示用 i18n 鍵（渲染時才 t()，模組頂層不翻譯）

import type { TFunction } from 'i18next'

export interface TreatmentOption {
    value: string
    /** 顯示用 i18n 鍵 */
    labelKey: string
}

/** 藥品類別 */
export const TREATMENT_CATEGORY_OPTIONS: readonly TreatmentOption[] = [
    { value: 'dewormer', labelKey: 'animalRecords.treatment.category.dewormer' },
    { value: 'antibiotic', labelKey: 'animalRecords.treatment.category.antibiotic' },
    { value: 'other', labelKey: 'animalRecords.treatment.category.other' },
]

/** 施打途徑 */
export const TREATMENT_ROUTE_OPTIONS: readonly TreatmentOption[] = [
    { value: 'IM', labelKey: 'animalRecords.treatment.route.IM' },
    { value: 'IV', labelKey: 'animalRecords.treatment.route.IV' },
    { value: 'SC', labelKey: 'animalRecords.treatment.route.SC' },
    { value: 'PO', labelKey: 'animalRecords.treatment.route.PO' },
]

// 寫入資料庫病歷內容用的固定繁中全稱（ManualWeightEntry 組進 observation.content /
// deworming_dose）。這是存進 DB 的資料而非畫面文字，不可隨 UI 語言變動，故刻意不走 i18n。
const STORED_CATEGORY_LABELS: Readonly<Record<string, string>> = {
    dewormer: '驅蟲藥',
    antibiotic: '抗生素',
    other: '其他',
}
const STORED_ROUTE_LABELS: Readonly<Record<string, string>> = {
    IM: 'IM 肌肉注射',
    IV: 'IV 靜脈注射',
    SC: 'SC 皮下注射',
    PO: 'PO 口服',
}

/** 藥品類別碼 → 繁中全稱（供寫入 DB 的病歷內容用，查無回原字串；畫面顯示請用 treatmentCategoryDisplayLabel） */
export function treatmentCategoryLabel(value?: string): string {
    if (!value) return ''
    return STORED_CATEGORY_LABELS[value] ?? value
}

/** 藥品類別碼 → 目前語言的顯示名稱（查無回原字串） */
export function treatmentCategoryDisplayLabel(value: string | undefined, t: TFunction): string {
    if (!value) return ''
    const opt = TREATMENT_CATEGORY_OPTIONS.find((o) => o.value === value)
    return opt ? t(opt.labelKey) : value
}

/**
 * 施打紀錄藥品類別碼 → 藥物庫分類（`DRUG_CATEGORIES`）對照表。
 * 兩份分類清單並非一對一，但刻意窄化為一對一：藥物庫的「麻醉/止痛/鎮靜」屬其他用途
 * （如手術），與量體重順便施打的情境不符，`other` 只對應藥物庫的「其他」分類，
 * 避免右側藥物下拉把麻醉/止痛/鎮靜也混進候選清單（2026-08 使用者要求收窄）。
 * 用途：右側藥物下拉（DrugCombobox）依左側選的施打類別過濾藥物庫清單。
 * （值為後端藥物庫的分類資料，與 API 比對用，不翻譯。）
 */
export const TREATMENT_CATEGORY_DRUG_CATEGORIES: Record<string, readonly string[]> = {
    dewormer: ['驅蟲'],
    antibiotic: ['抗生素'],
    other: ['其他'],
}

/** 施打途徑碼 → 繁中全稱（供寫入 DB 的病歷內容用，查無回原字串；畫面顯示請用 treatmentRouteDisplayLabel） */
export function treatmentRouteLabel(value?: string): string {
    if (!value) return ''
    return STORED_ROUTE_LABELS[value] ?? value
}

/** 施打途徑碼 → 目前語言的顯示名稱（查無回原字串） */
export function treatmentRouteDisplayLabel(value: string | undefined, t: TFunction): string {
    if (!value) return ''
    const opt = TREATMENT_ROUTE_OPTIONS.find((o) => o.value === value)
    return opt ? t(opt.labelKey) : value
}
