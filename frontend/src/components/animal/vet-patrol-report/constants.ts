// 獸醫巡場報告 Dialog 常數與工廠函式（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import type { EntryRow } from './types'

/**
 * Category 設定。各 category 有 3 個欄位（觀察/建議/追蹤改善）；
 * - placeholder：使用者沒輸入時顯示的灰字 hint（不影響實際值）
 * - defaultValue：新增條目時的預設值（會塞進 textarea，使用者要刪才會空）
 *
 * 2026-05-11 使用者調整：防疫消毒「觀察內容」預填例行清消、其他類別有自訂 hint。
 */
export const CATEGORIES = [
    {
        key: 'pig_condition',
        label: '豬隻狀況',
        hasAnimal: true,
        placeholders: {
            observation: '例：右後腿輕微跛行...',
            suggestion: '例:休養觀察 3 日...',
            follow_up: '請陪同人員扼要填寫',
        },
        defaults: {},
    },
    {
        key: 'epidemic_prevention',
        label: '防疫及消毒計畫',
        hasAnimal: false,
        placeholders: {
            observation: '',
            suggestion: '',
            follow_up: '請陪同人員扼要填寫',
        },
        defaults: {
            observation: '全場定期清洗消毒（每週一次，週三）。',
        },
    },
    {
        key: 'case_record',
        label: '病歷紀錄',
        hasAnimal: true,
        placeholders: {
            observation: '',
            suggestion: '',
            follow_up: '請陪同人員扼要填寫',
        },
        defaults: {},
    },
    {
        key: 'other',
        label: '其他',
        hasAnimal: false,
        placeholders: {
            observation: '（時間、棟舍、溫度、濕度、帆布狀況）',
            suggestion: '',
            follow_up: '請陪同人員扼要填寫',
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
