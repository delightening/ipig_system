// Section Purpose 元件
// 2.0 Abstract / 2.1 Significance / 2.2 Replacement / 2.3 Reduction (+2.3.1-2.3.3) / 2.4 Refinement

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import type { SectionProps } from './types'

const ALT_PLATFORMS = [
    { id: 'altbib', labelKey: 'altbibLabel', url: 'https://ntp.niehs.nih.gov/whatwestudy/niceatm/altbib' },
    { id: 'db_alm', labelKey: 'dbAlmLabel', url: 'https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/EURL-ECVAM/datasets/DBALM/LATEST/online/dbalm.html' },
    { id: 're_place', labelKey: 'rePlaceLabel', url: 'https://www.re-place.be/' },
    { id: 'johns_hopkins', labelKey: 'johnsHopkinsLabel', url: 'http://altweb.jhsph.edu/resources/searchalt/searchaltdata.html' },
    { id: 'taat', labelKey: 'taatLabel', url: 'https://taat.nhri.edu.tw/' },
    { id: 'nc3rs_eda', labelKey: 'nc3rsEdaLabel', url: 'https://nc3rs.org.uk/' },
    { id: 'nc3rs_refinement', labelKey: 'nc3rsRefinementLabel', url: 'https://refinementdatabase.org/' },
    { id: 'other', labelKey: 'otherPlatformLabel' },
]

const SINGLE_HOUSING_REASONS = [
    'b1_pregnant_female', 'b1_breeding_male', 'b1_post_wean',
    'b2_post_surgery', 'b2_single_in_group', 'b2_metabolic_cage',
    'b3_aggressive', 'b3_temporary', 'b4_other',
]

const ANIMAL_REUSE_OPTIONS = [
    'no_further_procedure', 'partial_procedure_euthanasia',
    'teaching_purpose', 'deferred', 'other',
]

const DEFAULT_REFINEMENT = `本計畫於動物實驗設計與執行過程中，已充分考量實驗動物福利與精緻化原則（Refinement），以降低動物之疼痛、緊迫及不適情形。實驗操作前將由具相關訓練經驗之人員執行動物處置與監測，並依實驗需求採適當之麻醉與止痛措施，以減輕動物於實驗過程中的不適感。

於飼養管理方面，動物飼養環境依據機構實驗動物照護標準辦理，提供適當空間、通風、溫濕度控制及定期健康觀察，以維持動物良好生理狀態。同時配合機構環境豐富化政策，於飼養欄位內提供玩具球及鍊條等環境豐富化物件，使豬隻可進行探索及互動行為，以降低心理緊迫並促進其自然行為表現，提升整體動物福利。

此外，實驗期間將持續觀察動物之行為與健康狀況，如發現異常或疼痛跡象，將立即通報獸醫師並依建議採取適當處置措施，以確保動物福利並符合精緻化原則之要求。`

export function SectionPurpose({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {
    const { purpose } = formData.working_content

    const togglePlatform = (platformId: string) => {
        const current = purpose.replacement.alt_search.platforms
        const updated = current.includes(platformId)
            ? current.filter(p => p !== platformId)
            : [...current, platformId]
        updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
    }

    const toggleHousingReason = (reason: string) => {
        const current = purpose.reduction.single_housing.reasons
        const updated = current.includes(reason)
            ? current.filter(r => r !== reason)
            : [...current, reason]
        updateWorkingContent('purpose', 'reduction.single_housing.reasons', updated)
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t('aup.section2')}</CardTitle>
                <CardDescription>{t('aup.purpose.subtitle')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">

                {/* ========== 2.0 Abstract ========== */}
                <div className="space-y-2">
                    <h3 className="font-semibold">{t('aup.purpose.abstractLabel')}</h3>
                    <p className="text-sm text-muted-foreground">{t('aup.purpose.abstractSubtitle')}</p>
                    <Textarea
                        value={purpose.abstract}
                        onChange={(e) => updateWorkingContent('purpose', 'abstract', e.target.value)}
                        placeholder={t('aup.purpose.abstractPlaceholder')}
                        rows={6}
                    />
                </div>

                <div className="h-px bg-border my-4" />

                {/* ========== 2.1 Purpose and Significance ========== */}
                <div className="space-y-2">
                    <h3 className="font-semibold">{t('aup.purpose.significance')}</h3>
                    <Textarea
                        value={purpose.significance}
                        onChange={(e) => updateWorkingContent('purpose', 'significance', e.target.value)}
                        placeholder={t('aup.purpose.significancePlaceholder')}
                        rows={5}
                    />
                </div>

                <div className="h-px bg-border my-4" />

                {/* ========== 2.2 Replacement Principle ========== */}
                <div className="space-y-4">
                    <h3 className="font-semibold">{t('aup.purpose.replacementPrinciple')}</h3>

                    {/* 2.2.1 Live Animal Necessity */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.liveAnimalNecessity')} *</Label>
                        <Textarea
                            value={purpose.replacement.rationale}
                            onChange={(e) => updateWorkingContent('purpose', 'replacement.rationale', e.target.value)}
                            placeholder={t('aup.purpose.liveAnimalNecessityPlaceholder')}
                            rows={4}
                        />
                    </div>

                    {/* 2.2.2 Alternative Methods Search */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.altSearchLabel')} *</Label>
                        <div className="space-y-3 pl-4">
                            {ALT_PLATFORMS.map(platform => (
                                <div key={platform.id} className="flex items-start space-x-3 py-1">
                                    <Checkbox
                                        id={`search_${platform.id}`}
                                        checked={purpose.replacement.alt_search.platforms.includes(platform.id)}
                                        onCheckedChange={() => togglePlatform(platform.id)}
                                        className="mt-1"
                                    />
                                    <Label htmlFor={`search_${platform.id}`} className="font-normal leading-relaxed flex-1">
                                        {t(`aup.purpose.${platform.labelKey}`)}
                                        {platform.url && (
                                            <>
                                                <br />
                                                <a
                                                    href={platform.url}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="text-primary hover:underline text-sm break-all"
                                                >
                                                    {platform.url}
                                                </a>
                                            </>
                                        )}
                                    </Label>
                                </div>
                            ))}
                        </div>
                        {purpose.replacement.alt_search.platforms.includes('other') && (
                            <Input
                                placeholder={t('aup.purpose.otherDbPlaceholder')}
                                value={purpose.replacement.alt_search.other_name || ''}
                                onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.other_name', e.target.value)}
                                className="mt-2 ml-4"
                            />
                        )}
                    </div>
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.searchKeywords')} *</Label>
                        <Input
                            value={purpose.replacement.alt_search.keywords}
                            onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.keywords', e.target.value)}
                            placeholder={t('aup.purpose.searchKeywordsPlaceholder')}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.searchConclusion')} *</Label>
                        <Textarea
                            value={purpose.replacement.alt_search.conclusion}
                            onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.conclusion', e.target.value)}
                            placeholder={t('aup.purpose.searchConclusionPlaceholder')}
                            rows={3}
                        />
                    </div>

                    {/* 2.2.3 Duplicate Experiment */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.duplicateExperiment')}</Label>
                        <Select
                            value={purpose.duplicate.status}
                            onValueChange={(value) => {
                                updateWorkingContent('purpose', 'duplicate.status', value)
                                updateWorkingContent('purpose', 'duplicate.regulation_basis', '')
                                updateWorkingContent('purpose', 'duplicate.previous_iacuc_no', '')
                                updateWorkingContent('purpose', 'duplicate.justification', '')
                            }}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder={t('common.pleaseSelect')} />
                            </SelectTrigger>
                            <SelectContent>
                                {['no', 'not_applicable', 'yes_continuation', 'yes_duplicate'].map(opt => (
                                    <SelectItem key={opt} value={opt}>
                                        {t(`aup.purpose.duplicateStatusOptions.${opt}`)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        {purpose.duplicate.status === 'not_applicable' && (
                            <div className="space-y-2 mt-2">
                                <Label>{t('aup.purpose.regulationBasis')} *</Label>
                                <Input
                                    value={purpose.duplicate.regulation_basis}
                                    onChange={(e) => updateWorkingContent('purpose', 'duplicate.regulation_basis', e.target.value)}
                                    placeholder={t('aup.purpose.regulationBasisPlaceholder')}
                                />
                            </div>
                        )}
                        {purpose.duplicate.status === 'yes_continuation' && (
                            <div className="space-y-2 mt-2">
                                <Label>{t('aup.purpose.previousIacucNo')} *</Label>
                                <Input
                                    value={purpose.duplicate.previous_iacuc_no}
                                    onChange={(e) => updateWorkingContent('purpose', 'duplicate.previous_iacuc_no', e.target.value)}
                                    placeholder={t('aup.purpose.previousIacucNoPlaceholder')}
                                />
                            </div>
                        )}
                        {purpose.duplicate.status === 'yes_duplicate' && (
                            <div className="space-y-2 mt-2">
                                <Label>{t('aup.purpose.duplicateJustification')} *</Label>
                                <Textarea
                                    value={purpose.duplicate.justification}
                                    onChange={(e) => updateWorkingContent('purpose', 'duplicate.justification', e.target.value)}
                                    placeholder={t('aup.purpose.duplicateJustificationPlaceholder')}
                                    rows={3}
                                />
                            </div>
                        )}
                    </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* ========== 2.3 Reduction Principle ========== */}
                <div className="space-y-4">
                    <h3 className="font-semibold">{t('aup.purpose.reductionPrinciple')}</h3>
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.reductionDesign')} *</Label>
                        <Textarea
                            value={purpose.reduction.design}
                            onChange={(e) => updateWorkingContent('purpose', 'reduction.design', e.target.value)}
                            placeholder={t('aup.purpose.reductionDesignPlaceholder')}
                            rows={6}
                        />
                    </div>

                    <div className="h-px bg-border my-2" />

                    {/* 2.3.1 Special Care */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.specialCareTitle')}</Label>
                        <Select
                            value={purpose.reduction.special_care.needed === null ? '' : purpose.reduction.special_care.needed ? 'yes' : 'no'}
                            onValueChange={(val) => {
                                updateWorkingContent('purpose', 'reduction.special_care.needed', val === 'yes')
                                if (val === 'no') updateWorkingContent('purpose', 'reduction.special_care.description', '')
                            }}
                        >
                            <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="no">{t('common.no')}</SelectItem>
                                <SelectItem value="yes">{t('common.yes')}</SelectItem>
                            </SelectContent>
                        </Select>
                        {purpose.reduction.special_care.needed && (
                            <Textarea
                                value={purpose.reduction.special_care.description}
                                onChange={(e) => updateWorkingContent('purpose', 'reduction.special_care.description', e.target.value)}
                                placeholder={t('aup.purpose.specialCareDescriptionPlaceholder')}
                                rows={3}
                            />
                        )}
                    </div>

                    <div className="h-px bg-border my-2" />

                    {/* 2.3.2 Single Housing */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.singleHousingTitle')}</Label>
                        <Select
                            value={purpose.reduction.single_housing.required === null ? '' : purpose.reduction.single_housing.required ? 'yes' : 'no'}
                            onValueChange={(val) => {
                                updateWorkingContent('purpose', 'reduction.single_housing.required', val === 'yes')
                                if (val === 'no') {
                                    updateWorkingContent('purpose', 'reduction.single_housing.reasons', [])
                                    updateWorkingContent('purpose', 'reduction.single_housing.monitoring_method', '')
                                    updateWorkingContent('purpose', 'reduction.single_housing.estimated_duration', '')
                                }
                            }}
                        >
                            <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="no">{t('common.no')}</SelectItem>
                                <SelectItem value="yes">{t('common.yes')}</SelectItem>
                            </SelectContent>
                        </Select>
                        {purpose.reduction.single_housing.required && (
                            <div className="space-y-3 pl-4">
                                <Label>{t('aup.purpose.singleHousingReasons')}</Label>
                                {SINGLE_HOUSING_REASONS.map(reason => (
                                    <div key={reason} className="flex items-center space-x-3">
                                        <Checkbox
                                            id={`housing_${reason}`}
                                            checked={purpose.reduction.single_housing.reasons.includes(reason)}
                                            onCheckedChange={() => toggleHousingReason(reason)}
                                        />
                                        <Label htmlFor={`housing_${reason}`} className="font-normal cursor-pointer">
                                            {t(`aup.purpose.singleHousingReasonOptions.${reason}`)}
                                        </Label>
                                    </div>
                                ))}
                                {purpose.reduction.single_housing.reasons.includes('b2_metabolic_cage') && (
                                    <div className="space-y-1">
                                        <Label>{t('aup.purpose.metabolicCageDuration')}</Label>
                                        <Input
                                            value={purpose.reduction.single_housing.metabolic_cage_duration}
                                            onChange={(e) => updateWorkingContent('purpose', 'reduction.single_housing.metabolic_cage_duration', e.target.value)}
                                            placeholder={t('aup.purpose.metabolicCageDurationPlaceholder')}
                                        />
                                    </div>
                                )}
                                <div className="space-y-1">
                                    <Label>{t('aup.purpose.singleHousingMonitoringMethod')}</Label>
                                    <Input
                                        value={purpose.reduction.single_housing.monitoring_method}
                                        onChange={(e) => updateWorkingContent('purpose', 'reduction.single_housing.monitoring_method', e.target.value)}
                                        placeholder={t('aup.purpose.singleHousingMonitoringMethodPlaceholder')}
                                    />
                                </div>
                                <div className="space-y-1">
                                    <Label>{t('aup.purpose.singleHousingEstimatedDuration')}</Label>
                                    <Input
                                        value={purpose.reduction.single_housing.estimated_duration}
                                        onChange={(e) => updateWorkingContent('purpose', 'reduction.single_housing.estimated_duration', e.target.value)}
                                        placeholder={t('aup.purpose.singleHousingEstimatedDurationPlaceholder')}
                                    />
                                </div>
                            </div>
                        )}
                    </div>

                    <div className="h-px bg-border my-2" />

                    {/* 2.3.3 Animal Reuse */}
                    <div className="space-y-2">
                        <Label>{t('aup.purpose.animalReuseTitle')}</Label>
                        <Select
                            value={purpose.reduction.animal_reuse.considered === null ? '' : purpose.reduction.animal_reuse.considered ? 'yes' : 'no'}
                            onValueChange={(val) => {
                                updateWorkingContent('purpose', 'reduction.animal_reuse.considered', val === 'yes')
                                if (val === 'no') {
                                    updateWorkingContent('purpose', 'reduction.animal_reuse.plan', '')
                                    updateWorkingContent('purpose', 'reduction.animal_reuse.plan_other', '')
                                }
                            }}
                        >
                            <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="no">{t('common.no')}</SelectItem>
                                <SelectItem value="yes">{t('common.yes')}</SelectItem>
                            </SelectContent>
                        </Select>
                        {purpose.reduction.animal_reuse.considered && (
                            <div className="space-y-2 pl-4">
                                <Label>{t('aup.purpose.animalReusePlan')}</Label>
                                <Select
                                    value={purpose.reduction.animal_reuse.plan}
                                    onValueChange={(val) => {
                                        updateWorkingContent('purpose', 'reduction.animal_reuse.plan', val)
                                        if (val !== 'other') updateWorkingContent('purpose', 'reduction.animal_reuse.plan_other', '')
                                    }}
                                >
                                    <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                                    <SelectContent>
                                        {ANIMAL_REUSE_OPTIONS.map(opt => (
                                            <SelectItem key={opt} value={opt}>
                                                {t(`aup.purpose.animalReusePlanOptions.${opt}`)}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                {purpose.reduction.animal_reuse.plan === 'other' && (
                                    <Textarea
                                        value={purpose.reduction.animal_reuse.plan_other}
                                        onChange={(e) => updateWorkingContent('purpose', 'reduction.animal_reuse.plan_other', e.target.value)}
                                        placeholder={t('aup.purpose.animalReusePlanOtherPlaceholder')}
                                        rows={2}
                                    />
                                )}
                            </div>
                        )}
                    </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* ========== 2.4 Refinement ========== */}
                <div className="space-y-2">
                    <div className="flex items-center justify-between">
                        <h3 className="font-semibold">{t('aup.purpose.refinementLabel')}</h3>
                        {!purpose.refinement_description.trim() && (
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={() => updateWorkingContent('purpose', 'refinement_description', DEFAULT_REFINEMENT)}
                            >
                                {t('aup.purpose.insertDefaultRefinement')}
                            </Button>
                        )}
                    </div>
                    <p className="text-sm text-muted-foreground">{t('aup.purpose.refinementSubtitle')}</p>
                    <Textarea
                        value={purpose.refinement_description}
                        onChange={(e) => updateWorkingContent('purpose', 'refinement_description', e.target.value)}
                        placeholder={t('aup.purpose.refinementPlaceholder')}
                        rows={8}
                    />
                </div>

            </CardContent>
        </Card>
    )
}
