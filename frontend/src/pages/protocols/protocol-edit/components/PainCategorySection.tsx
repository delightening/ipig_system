// 4.1.3 疼痛等級評估 + 4.1.5 疼痛症狀 + 4.1.6 緩解措施
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
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

const RELIEF_MEASURES = [
    'alternative_painless_procedure',
    'anesthesia_analgesia',
    'humane_euthanasia',
    'no_relief_with_justification',
]

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function PainCategorySection({ formData, updateWorkingContent, t }: Props) {
    const { pain } = formData.working_content.design
    const selectedCategory = pain.category

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
                            {['B', 'C', 'D', 'E'].map(cat => (
                                <SelectItem key={cat} value={cat}>{t(`aup.design.painCategories.${cat}`)}</SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>

                {/* 依所選等級展開細項 checkbox */}
                {selectedCategory && CATEGORY_ITEMS[selectedCategory] && (
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
                    {RELIEF_MEASURES.map(measure => (
                        <div key={measure} className="flex items-start space-x-3 py-1">
                            <Checkbox
                                id={`relief_${measure}`}
                                checked={pain.relief_measures.includes(measure)}
                                onCheckedChange={() =>
                                    toggleArrayItem('design', 'pain.relief_measures', pain.relief_measures, measure)
                                }
                                className="mt-0.5"
                            />
                            <Label htmlFor={`relief_${measure}`} className="font-normal leading-relaxed cursor-pointer">
                                {t(`aup.design.reliefMeasures.${measure}`)}
                            </Label>
                        </div>
                    ))}
                </div>
                {pain.relief_measures.includes('anesthesia_analgesia') && (
                    <div className="pl-4 space-y-2">
                        <Label>{t('aup.design.reliefDrugNameLabel')} *</Label>
                        <Input
                            value={pain.relief_drug_name}
                            onChange={(e) => updateWorkingContent('design', 'pain.relief_drug_name', e.target.value)}
                            placeholder={t('aup.design.reliefDrugNamePlaceholder')}
                        />
                    </div>
                )}
                {pain.relief_measures.includes('no_relief_with_justification') && (
                    <div className="pl-4 space-y-2">
                        <Label>{t('aup.design.noReliefJustificationLabel')} *</Label>
                        <Textarea
                            value={pain.no_relief_justification}
                            onChange={(e) => updateWorkingContent('design', 'pain.no_relief_justification', e.target.value)}
                            placeholder={t('aup.design.noReliefJustificationPlaceholder')}
                            rows={3}
                        />
                    </div>
                )}
            </div>
        </>
    )
}
