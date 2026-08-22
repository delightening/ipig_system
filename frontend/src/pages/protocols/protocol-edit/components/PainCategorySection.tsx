// 4.1.3 疼痛等級評估 + 4.1.5 疼痛症狀 + 4.1.6 緩解措施
import { AutoGrowTextarea } from '@/components/ui/autoGrowTextarea'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { PAIN_CATEGORY_OPTIONS, RELIEF_MEASURE_OPTIONS } from '@/lib/constants/protocolDesignOptions'
import { fieldVisibleForVersion } from '@/lib/constants/protocolVersionManifests'
import type { SectionProps } from '../types'

// 每個 category 對應的細項 enum
const CATEGORY_ITEMS: Record<string, string[]> = {
    B: ['b_breeding_no_procedure', 'b_other'],
    C: [
        'c_handling_weighing_transport', 'c_injection_oral_non_irritant',
        'c_animal_marking', 'c_routine_farming', 'c_general_anesthesia',
        'c_avma_euthanasia', 'c_other',
    ],
    D: [
        'd_stress_transport_sedation', 'd_intubation_under_anesthesia',
        'd_survival_surgery_under_anesthesia', 'd_non_survival_surgery',
        'd_non_lethal_drug_exposure', 'd_catheter_implantation',
        'd_blood_draw_perfusion', 'd_non_preop_food_water_restrict',
        'd_pain_with_analgesia', 'd_induced_anatomical_physiological',
        'd_drug_physiological_damage', 'd_eye_skin_irritation_relievable',
        'd_other',
    ],
    E: [
        'e_severe_drug_damage_death', 'e_paralytic_without_anesthesia',
        'e_burn_large_skin_wound', 'e_induced_disease',
        'e_pain_threshold_procedure', 'e_chronic_pain_unrelievable',
        'e_excessive_food_water_restrict', 'e_extreme_environment',
        'e_procedure_may_cause_death', 'e_pain_distress_study',
        'e_non_avma_euthanasia', 'e_other',
    ],
}

const DISTRESS_SIGNS = [
    'weight_loss', 'reduced_food_water', 'dehydration', 'unkempt_fur',
    'isolation_hiding', 'self_mutilation', 'abnormal_posture', 'abnormal_breathing',
    'abnormal_activity', 'aggression', 'lacrimation_no_blink', 'muscle_rigidity_weakness',
    'tremor_convulsion', 'vocalization', 'surgical_site_inflammation', 'teeth_grinding',
    'other',
]

// 4.1.6「投予麻醉或止痛藥」時的可選藥品（沿用既有手術藥單之麻醉/止痛子集）
// 多選結果與「其他」自由輸入共同以「、」join 存入 pain.relief_drug_name（維持 string，PDF 不變）
const RELIEF_DRUG_GROUPS: { group: 'anesthesia' | 'analgesic'; drugs: string[] }[] = [
    { group: 'anesthesia', drugs: ['Atropine', 'Azaperonum', 'Zoletil-50', 'Isoflurane'] },
    { group: 'analgesic', drugs: ['Ketorolac', 'meloxicam', 'ketoprofen'] },
]
const ALL_RELIEF_DRUGS = RELIEF_DRUG_GROUPS.flatMap(g => g.drugs)

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't' | 'formVersion'>

export function PainCategorySection({ formData, updateWorkingContent, t, formVersion }: Props) {
    const { pain } = formData.working_content.design
    const selectedCategory = pain.category

    // 4.1.6 藥品多選 + 其他：由 relief_drug_name 字串衍生勾選狀態與「其他」文字（不額外開 state）
    // 比對時 trim（容錯既有「、」前後空格），但「其他」保留原始 token（避免控制元件吃掉使用者輸入的空格）
    const reliefTokens = (pain.relief_drug_name || '').split('、').filter(Boolean)
    const checkedReliefDrugs = reliefTokens.map(tk => tk.trim()).filter(tk => ALL_RELIEF_DRUGS.includes(tk))
    const reliefDrugOther = reliefTokens.filter(tk => !ALL_RELIEF_DRUGS.includes(tk.trim())).join('、')
    const rebuildReliefDrugName = (drugs: string[], other: string) =>
        updateWorkingContent('design', 'pain.relief_drug_name', [...drugs, other].filter(Boolean).join('、'))
    const toggleReliefDrug = (drug: string) =>
        rebuildReliefDrugName(
            checkedReliefDrugs.includes(drug)
                ? checkedReliefDrugs.filter(d => d !== drug)
                : [...checkedReliefDrugs, drug],
            reliefDrugOther,
        )

    const toggleReliefMeasure = (measure: string) => {
        const isRemoving = pain.relief_measures.includes(measure)
        toggleArrayItem('design', 'pain.relief_measures', pain.relief_measures, measure)
        // 取消「投予麻醉或止痛藥」時一併清空已選藥品，避免殘留字串被 PDF / 唯讀頁渲染
        if (measure === 'anesthesia_analgesia' && isRemoving) {
            updateWorkingContent('design', 'pain.relief_drug_name', '')
        }
    }

    const toggleArrayItem = (section: string, path: string, current: string[], item: string) => {
        const updated = current.includes(item)
            ? current.filter(i => i !== item)
            : [...current, item]
        updateWorkingContent(section as keyof typeof formData.working_content, path, updated)
    }

    return (
        <>
            {/* 4.1.3 疼痛等級：單選 + 細項複選 */}
            <div className="space-y-4">
                <div className="space-y-2">
                    <Label>{t('aup.design.painCategoryLabel')}</Label>
                    <p className="text-sm text-muted-foreground">{t('aup.design.painCategorySubtitle')}</p>
                    <Select
                        value={selectedCategory}
                        onValueChange={(val) => {
                            updateWorkingContent('design', 'pain.category', val)
                            // 切換等級時清空細項
                            updateWorkingContent('design', 'pain.category_items', [])
                            updateWorkingContent('design', 'pain.category_item_other_text', '')
                        }}
                    >
                        <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                        <SelectContent>
                            {PAIN_CATEGORY_OPTIONS.map(opt => (
                                <SelectItem key={opt.value} value={opt.value}>{t(opt.labelKey)}</SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>

                {/* 依所選等級展開細項 checkbox（F 版才有細項勾選；C/D/E 只選等級） */}
                {fieldVisibleForVersion('design.painCategoryItems', formVersion) && selectedCategory && CATEGORY_ITEMS[selectedCategory] && (
                    <div className="space-y-2 pl-4 border-l-2 border-muted">
                        {CATEGORY_ITEMS[selectedCategory].map(item => (
                            <div key={item} className="flex items-start space-x-3 py-1">
                                <Checkbox
                                    id={`pain_item_${item}`}
                                    checked={pain.category_items.includes(item)}
                                    onCheckedChange={() =>
                                        toggleArrayItem('design', 'pain.category_items', pain.category_items, item)
                                    }
                                    className="mt-0.5"
                                />
                                <Label htmlFor={`pain_item_${item}`} className="font-normal leading-relaxed cursor-pointer">
                                    {t(`aup.design.painCategoryItems.${item}`)}
                                </Label>
                            </div>
                        ))}
                        {/* *_other 被勾選時顯示文字欄位 */}
                        {pain.category_items.some(i => i.endsWith('_other')) && (
                            <Input
                                className="mt-2 ml-7"
                                value={pain.category_item_other_text}
                                onChange={(e) => updateWorkingContent('design', 'pain.category_item_other_text', e.target.value)}
                                placeholder={t('aup.design.painCategoryItemOtherPlaceholder')}
                            />
                        )}
                    </div>
                )}
            </div>

            {/* 4.1.4 痛苦症狀 + 4.1.5 緩解措施＝F 版新增勾選清單；C/D/E 版不顯示 = 版本忠實 */}
            {fieldVisibleForVersion('design.painDistressSigns', formVersion) && (
            <>
            <div className="h-px bg-border my-4" />

            {/* 4.1.5 疼痛或痛苦症狀（複選） */}
            <div className="space-y-4">
                <Label>{t('aup.design.distressSignsLabel')}</Label>
                <div className="grid gap-2 md:grid-cols-2 pl-4">
                    {DISTRESS_SIGNS.map(sign => (
                        <div key={sign} className="flex items-start space-x-3 py-1">
                            <Checkbox
                                id={`distress_${sign}`}
                                checked={pain.distress_signs.includes(sign)}
                                onCheckedChange={() =>
                                    toggleArrayItem('design', 'pain.distress_signs', pain.distress_signs, sign)
                                }
                                className="mt-0.5"
                            />
                            <Label htmlFor={`distress_${sign}`} className="font-normal leading-relaxed cursor-pointer">
                                {t(`aup.design.distressSigns.${sign}`)}
                            </Label>
                        </div>
                    ))}
                </div>
                {pain.distress_signs.includes('other') && (
                    <Input
                        className="ml-4"
                        value={pain.distress_signs_other_text}
                        onChange={(e) => updateWorkingContent('design', 'pain.distress_signs_other_text', e.target.value)}
                        placeholder={t('aup.design.distressSignsOtherPlaceholder')}
                    />
                )}
            </div>

            <div className="h-px bg-border my-4" />

            {/* 4.1.6 緩解措施（複選） */}
            <div className="space-y-4">
                <Label>{t('aup.design.reliefMeasuresLabel')}</Label>
                <div className="space-y-2 pl-4">
                    {RELIEF_MEASURE_OPTIONS.map(({ value: measure, labelKey }) => (
                        <div key={measure} className="flex items-start space-x-3 py-1">
                            <Checkbox
                                id={`relief_${measure}`}
                                checked={pain.relief_measures.includes(measure)}
                                onCheckedChange={() => toggleReliefMeasure(measure)}
                                className="mt-0.5"
                            />
                            <Label htmlFor={`relief_${measure}`} className="font-normal leading-relaxed cursor-pointer">
                                {t(labelKey)}
                            </Label>
                        </div>
                    ))}
                </div>
                {pain.relief_measures.includes('anesthesia_analgesia') && (
                    <div className="pl-4 space-y-3">
                        <Label>{t('aup.design.reliefDrugNameLabel')} *</Label>
                        {RELIEF_DRUG_GROUPS.map(group => (
                            <div key={group.group} className="space-y-2">
                                <p className="text-sm font-medium text-muted-foreground">
                                    {t(`aup.design.reliefDrugGroups.${group.group}`)}
                                </p>
                                <div className="grid gap-2 md:grid-cols-2 pl-2">
                                    {group.drugs.map(drug => (
                                        <div key={drug} className="flex items-center space-x-2">
                                            <Checkbox
                                                id={`relief_drug_${drug}`}
                                                checked={checkedReliefDrugs.includes(drug)}
                                                onCheckedChange={() => toggleReliefDrug(drug)}
                                            />
                                            <Label htmlFor={`relief_drug_${drug}`} className="font-normal cursor-pointer">
                                                {drug}
                                            </Label>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}
                        <div className="space-y-2">
                            <Label htmlFor="relief_drug_other" className="text-sm font-medium text-muted-foreground">
                                {t('aup.design.reliefDrugOtherLabel')}
                            </Label>
                            <Input
                                id="relief_drug_other"
                                value={reliefDrugOther}
                                onChange={(e) => rebuildReliefDrugName(checkedReliefDrugs, e.target.value)}
                                placeholder={t('aup.design.reliefDrugOtherPlaceholder')}
                            />
                        </div>
                    </div>
                )}
                {pain.relief_measures.includes('no_relief_with_justification') && (
                    <div className="pl-4 space-y-2">
                        <Label>{t('aup.design.noReliefJustificationLabel')} *</Label>
                        <AutoGrowTextarea
                            value={pain.no_relief_justification}
                            onChange={(e) => updateWorkingContent('design', 'pain.no_relief_justification', e.target.value)}
                            placeholder={t('aup.design.noReliefJustificationPlaceholder')}
                            rows={3}
                        />
                    </div>
                )}
            </div>
            </>
            )}
        </>
    )
}
