// 疼痛評估共用常數與工具函式（TU-03-05-03B）
// 供 PainAssessmentTab、ObservationPainSection 共用

export interface AssessmentOption {
    score: number
    label: string
}

// ── 傷口狀況 (Incision) ──────────────────────
export const INCISION_OPTIONS: AssessmentOption[] = [
    { score: 0, label: '正常' },
    { score: 1, label: '輕微透明滲出液或鮮紅色血液' },
    { score: 2, label: '不透明滲出液體或暗褐色血液' },
    { score: 3, label: '膿樣分泌物' },
]

// ── 態度/行為 (Attitude/Behavior) ────────────
export const ATTITUDE_OPTIONS: AssessmentOption[] = [
    { score: 0, label: '正常' },
    { score: 1, label: '皮膚外觀的改變（豎毛、體表沾黏分泌物等）' },
    { score: 2, label: '步態改變或姿勢異常（走路或站立或排尿姿勢異常）' },
    { score: 3, label: '反應遲鈍，警覺性降低，當人接近仍持續自殘或持續磨擦或舔舐傷口' },
    { score: 4, label: '人接近時焦慮不安或產生緊張行為' },
    { score: 5, label: '人接近時發出叫聲且迴避甚至出現攻擊行為' },
]

// ── 食慾 (Appetite) ─────────────────────────
export const APPETITE_OPTIONS: AssessmentOption[] = [
    { score: 0, label: '正常' },
    { score: 1, label: '飼料未吃完' },
    { score: 2, label: '飼料不吃且對零食無興趣' },
]

// ── 排便 (Feces) ────────────────────────────
export const FECES_OPTIONS: AssessmentOption[] = [
    { score: 0, label: '正常' },
    { score: 1, label: '排便量減少' },
    { score: 2, label: '排便異常（軟便、下痢、血便）' },
    { score: 3, label: '未排便' },
]

// ── 排尿 (Urine) ────────────────────────────
export const URINE_OPTIONS: AssessmentOption[] = [
    { score: 0, label: '正常' },
    { score: 1, label: '排尿次數增加或減少' },
    { score: 2, label: '尿色異常（深黃、鮮紅色或暗褐色等）' },
    { score: 3, label: '未排尿' },
]

// ── 疼痛分數 (Pain score) ───────────────────
export const PAIN_SCORE_OPTIONS: AssessmentOption[] = [
    { score: 1, label: '第一級：深度觸診手術部位不會直接引起周圍組織的反應，動物對觸診無太大反應。' },
    { score: 2, label: '第二級：深度觸診手術部位會直接引起周圍組織反應（如對側或肢體的反應），動物對觸診反應尚可接受。' },
    { score: 3, label: '第三級：深度觸診手術部位會直接引起周圍組織劇烈反應，動物對觸診反應無法接受。' },
    { score: 4, label: '第四級：深度觸診引起劇烈反應，非手術部位觸診也引起激烈反應，且動物發出叫聲，對觸診反應非常排斥。' },
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
    label: string
    grade: number
    variant: 'default' | 'secondary' | 'outline' | 'destructive'
    advice: string
}

export function getPainGrade(total: number | null): PainGrade | null {
    if (total === null) return null
    if (total <= 5) return { label: `正常（${total}分）`, grade: 1, variant: 'default', advice: '不給藥，仍需持續觀察' }
    if (total <= 10) return { label: `輕度疼痛（${total}分）`, grade: 2, variant: 'secondary', advice: '給予止痛藥' }
    if (total <= 15) return { label: `中度疼痛（${total}分）`, grade: 3, variant: 'outline', advice: '每 8–12 小時給一次止痛藥' }
    return { label: `重度疼痛（${total}分）`, grade: 4, variant: 'destructive', advice: '每 8–12 小時給一次止痛藥並考慮合併用藥' }
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
    injection_ketorolac: boolean
    injection_meloxicam: boolean
    oral_meloxicam: boolean
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
    injection_ketorolac: false,
    injection_meloxicam: false,
    oral_meloxicam: false,
}
