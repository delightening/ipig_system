// 疼痛評估共用常數與工具函式（TU-03-05-03B）
// 供 PainAssessmentTab、ObservationPainSection 共用
//
// i18n：模組級常數只存「i18n 鍵」（labelKey / adviceKey），不在頂層呼叫 t()；
// 由元件於渲染時再 t(labelKey)，語言切換後才會即時更新。

export interface AssessmentOption {
    score: number
    /** i18n 鍵（渲染時才 t()） */
    labelKey: string
}

// ── 傷口狀況 (Incision) ──────────────────────
export const INCISION_OPTIONS: AssessmentOption[] = [
    { score: 0, labelKey: 'animalRecords.painAssessment.options.incision.normal' },
    { score: 1, labelKey: 'animalRecords.painAssessment.options.incision.mildExudate' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.incision.opaqueExudate' },
    { score: 3, labelKey: 'animalRecords.painAssessment.options.incision.purulent' },
]

// ── 態度/行為 (Attitude/Behavior) ────────────
export const ATTITUDE_OPTIONS: AssessmentOption[] = [
    { score: 0, labelKey: 'animalRecords.painAssessment.options.attitude.normal' },
    { score: 1, labelKey: 'animalRecords.painAssessment.options.attitude.skinChange' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.attitude.gaitChange' },
    { score: 3, labelKey: 'animalRecords.painAssessment.options.attitude.dullSelfInjury' },
    { score: 4, labelKey: 'animalRecords.painAssessment.options.attitude.anxious' },
    { score: 5, labelKey: 'animalRecords.painAssessment.options.attitude.aggressive' },
]

// ── 食慾 (Appetite) ─────────────────────────
export const APPETITE_OPTIONS: AssessmentOption[] = [
    { score: 0, labelKey: 'animalRecords.painAssessment.options.appetite.normal' },
    { score: 1, labelKey: 'animalRecords.painAssessment.options.appetite.leftoverFeed' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.appetite.noEating' },
]

// ── 排便 (Feces) ────────────────────────────
export const FECES_OPTIONS: AssessmentOption[] = [
    { score: 0, labelKey: 'animalRecords.painAssessment.options.feces.normal' },
    { score: 1, labelKey: 'animalRecords.painAssessment.options.feces.reduced' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.feces.abnormal' },
    { score: 3, labelKey: 'animalRecords.painAssessment.options.feces.none' },
]

// ── 排尿 (Urine) ────────────────────────────
export const URINE_OPTIONS: AssessmentOption[] = [
    { score: 0, labelKey: 'animalRecords.painAssessment.options.urine.normal' },
    { score: 1, labelKey: 'animalRecords.painAssessment.options.urine.frequencyChange' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.urine.colorAbnormal' },
    { score: 3, labelKey: 'animalRecords.painAssessment.options.urine.none' },
]

// ── 疼痛分數 (Pain score) ───────────────────
export const PAIN_SCORE_OPTIONS: AssessmentOption[] = [
    { score: 1, labelKey: 'animalRecords.painAssessment.options.painScore.level1' },
    { score: 2, labelKey: 'animalRecords.painAssessment.options.painScore.level2' },
    { score: 3, labelKey: 'animalRecords.painAssessment.options.painScore.level3' },
    { score: 4, labelKey: 'animalRecords.painAssessment.options.painScore.level4' },
]

// ── 總分計算 ────────────────────────────────
export function calcTotal(
    incision: string, attitude: string, appetite: string,
    feces: string, urine: string, painScore: string
): number | null {
    if (!incision || !attitude || !appetite || !feces || !urine || !painScore) return null
    return parseInt(incision) + parseInt(attitude) + parseInt(appetite) +
        parseInt(feces) + parseInt(urine) + parseInt(painScore)
}

// ── 疼痛分級 ────────────────────────────────
export interface PainGrade {
    /** 分級名稱 i18n 鍵（不含分數），例如「正常」「輕度疼痛」 */
    labelKey: string
    /** 原始總分（供組合「名稱（N 分）」顯示） */
    total: number
    grade: number
    variant: 'default' | 'secondary' | 'outline' | 'destructive'
    /** 建議處置 i18n 鍵 */
    adviceKey: string
}

export function getPainGrade(total: number | null): PainGrade | null {
    if (total === null) return null
    if (total <= 5) return { labelKey: 'animalRecords.painAssessment.grade.normal', total, grade: 1, variant: 'default', adviceKey: 'animalRecords.painAssessment.advice.normal' }
    if (total <= 10) return { labelKey: 'animalRecords.painAssessment.grade.mild', total, grade: 2, variant: 'secondary', adviceKey: 'animalRecords.painAssessment.advice.mild' }
    if (total <= 15) return { labelKey: 'animalRecords.painAssessment.grade.moderate', total, grade: 3, variant: 'outline', adviceKey: 'animalRecords.painAssessment.advice.moderate' }
    return { labelKey: 'animalRecords.painAssessment.grade.severe', total, grade: 4, variant: 'destructive', adviceKey: 'animalRecords.painAssessment.advice.severe' }
}

// ── 術後給藥項目型別 ─────────────────────────
export interface MedicationItem {
    name: string
    dose: string
    drug_option_id?: string
    dosage_unit?: string
}

// ── 疼痛評估表單項目型別 ─────────────────────
export interface PainAssessmentEntry {
    id?: string
    post_op_days: string
    time_period: string
    incision: string
    attitude_behavior: string
    appetite: string
    feces: string
    urine: string
    pain_score: string
    post_medications: MedicationItem[]
}

export const emptyPainEntry: PainAssessmentEntry = {
    post_op_days: '',
    time_period: 'AM',
    incision: '',
    attitude_behavior: '',
    appetite: '',
    feces: '',
    urine: '',
    pain_score: '',
    post_medications: [],
}
