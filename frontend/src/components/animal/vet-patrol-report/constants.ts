// 獸醫巡場報告 Dialog 常數與工廠函式（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import type { EntryRow } from './types'

/**
 * Category 設定。各 category 有 3 個欄位（觀察/建議/追蹤改善）；
 * - labelKey / placeholderKeys：i18n key（模組頂層不可存翻譯後字串，渲染時才 t()）。
 *   placeholderKeys 的值為 '' 表示該欄位沒有 hint（不顯示灰字）。
 * - defaults：新增條目時的預設值（會塞進 textarea 並隨報告存進 DB，屬資料內容，不翻譯；
 *   使用者要刪才會空）
 *
 * 2026-05-11 使用者調整：防疫消毒「觀察內容」預填例行清消、其他類別有自訂 hint。
 */
export const CATEGORIES = [
    {
        key: 'pig_condition',
        labelKey: 'animalActions.vetPatrol.category.pigCondition',
        hasAnimal: true,
        placeholderKeys: {
            observation: 'animalActions.vetPatrol.placeholder.pigConditionObservation',
            suggestion: 'animalActions.vetPatrol.placeholder.pigConditionSuggestion',
            follow_up: 'animalActions.vetPatrol.placeholder.followUpBrief',
        },
        defaults: {},
    },
    {
        key: 'epidemic_prevention',
        labelKey: 'animalActions.vetPatrol.category.epidemicPrevention',
        hasAnimal: false,
        placeholderKeys: {
            observation: '',
            suggestion: '',
            follow_up: 'animalActions.vetPatrol.placeholder.followUpBrief',
        },
        defaults: {
            observation: '全場定期清洗消毒（每週一次，週三）。',
        },
    },
    {
        key: 'case_record',
        labelKey: 'animalActions.vetPatrol.category.caseRecord',
        hasAnimal: true,
        placeholderKeys: {
            observation: '',
            suggestion: '',
            follow_up: 'animalActions.vetPatrol.placeholder.followUpBrief',
        },
        defaults: {},
    },
    {
        key: 'other',
        labelKey: 'animalActions.common.other',
        hasAnimal: false,
        placeholderKeys: {
            observation: 'animalActions.vetPatrol.placeholder.otherObservation',
            suggestion: '',
            follow_up: 'animalActions.vetPatrol.placeholder.followUpBrief',
        },
        defaults: {},
    },
] as const

export type CategoryKey = typeof CATEGORIES[number]['key']

let _tempKeyCounter = 0
export const newTempKey = () => `tmp-${Date.now()}-${++_tempKeyCounter}`

export const emptyEntry = (category: CategoryKey): EntryRow => {
    const cat = CATEGORIES.find(c => c.key === category)
    const defaults = (cat?.defaults ?? {}) as Partial<Record<'observation' | 'suggestion' | 'follow_up', string>>
    return {
        tempKey: newTempKey(),
        category,
        animal_ids: [],
        observation: defaults.observation ?? '',
        suggestion: defaults.suggestion ?? '',
        follow_up: defaults.follow_up ?? '',
    }
}
