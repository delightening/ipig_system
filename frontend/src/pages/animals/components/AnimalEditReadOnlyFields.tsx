import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { AnimalSource, Animal } from '@/lib/api'
import { animalSpeciesLabel } from '@/lib/animalSpecies'

interface AnimalEditReadOnlyFieldsProps {
    animal: Animal
    sources?: AnimalSource[]
}

export function AnimalEditReadOnlyFields({ animal, sources }: AnimalEditReadOnlyFieldsProps) {
    const { t } = useTranslation()
    return (
        <>
            {/* 耳號 */}
            <div className="space-y-2">
                <Label htmlFor="ear_tag" className="text-muted-foreground">{t('animals.earTag')} *</Label>
                <Input id="ear_tag" value={animal.ear_tag || ''} disabled className="bg-muted" />
            </div>

            {/* 品種 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">{t('animals.breed')} *</Label>
                <Input value={animalSpeciesLabel(animal, (b) => t(`animals.breedLabels.${b}`))} disabled className="bg-muted" />
            </div>

            {/* 性別 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">{t('animals.gender')} *</Label>
                <Input value={t(`animals.genderLabels.${animal.gender}`)} disabled className="bg-muted" />
            </div>

            {/* 來源 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">{t('animalPages.shared.source')}</Label>
                <Input
                    value={animal.source_id ? sources?.find(s => s.id === animal.source_id)?.name || '' : t('animalPages.editPage.sourceNotSpecified')}
                    disabled className="bg-muted"
                />
            </div>

            {/* 出生日期 */}
            <div className="space-y-2">
                <Label htmlFor="birth_date" className="text-muted-foreground">{t('animals.birthDate')}</Label>
                <Input
                    id="birth_date" type="date"
                    value={animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 進場日期 */}
            <div className="space-y-2">
                <Label htmlFor="entry_date" className="text-muted-foreground">{t('animals.entryDate')} *</Label>
                <Input
                    id="entry_date" type="date"
                    value={animal.entry_date ? new Date(animal.entry_date).toISOString().split('T')[0] : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 進場體重 */}
            <div className="space-y-2">
                <Label htmlFor="entry_weight" className="text-muted-foreground">{t('animalPages.shared.entryWeightKg')}</Label>
                <Input
                    id="entry_weight" type="text"
                    value={animal.entry_weight !== undefined && animal.entry_weight !== null ? String(animal.entry_weight) : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 實驗前代號 */}
            <div className="space-y-2">
                <Label htmlFor="pre_experiment_code" className="text-muted-foreground">{t('animalPages.shared.preExperimentCode')}</Label>
                <Input id="pre_experiment_code" value={animal.pre_experiment_code || ''} disabled className="bg-muted" />
            </div>
        </>
    )
}
