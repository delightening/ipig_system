import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { animalBreedNames, animalGenderNames, AnimalSource, Animal } from '@/lib/api'

interface AnimalEditReadOnlyFieldsProps {
    animal: Animal
    sources?: AnimalSource[]
}

export function AnimalEditReadOnlyFields({ animal, sources }: AnimalEditReadOnlyFieldsProps) {
    return (
        <>
            {/* 耳號 */}
            <div className="space-y-2">
                <Label htmlFor="ear_tag" className="text-muted-foreground">耳號 *</Label>
                <Input id="ear_tag" value={animal.ear_tag || ''} disabled className="bg-muted" />
            </div>

            {/* 品種 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">品種 *</Label>
                <Input
                    value={animal.breed === 'other' ? (animal.breed_other || '其他') : animalBreedNames[animal.breed]}
                    disabled className="bg-muted"
                />
            </div>

            {/* 性別 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">性別 *</Label>
                <Input value={animalGenderNames[animal.gender]} disabled className="bg-muted" />
            </div>

            {/* 來源 */}
            <div className="space-y-2">
                <Label className="text-muted-foreground">來源</Label>
                <Input
                    value={animal.source_id ? sources?.find(s => s.id === animal.source_id)?.name || '' : '未指定'}
                    disabled className="bg-muted"
                />
            </div>

            {/* 出生日期 */}
            <div className="space-y-2">
                <Label htmlFor="birth_date" className="text-muted-foreground">出生日期</Label>
                <Input
                    id="birth_date" type="date"
                    value={animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 進場日期 */}
            <div className="space-y-2">
                <Label htmlFor="entry_date" className="text-muted-foreground">進場日期 *</Label>
                <Input
                    id="entry_date" type="date"
                    value={animal.entry_date ? new Date(animal.entry_date).toISOString().split('T')[0] : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 進場體重 */}
            <div className="space-y-2">
                <Label htmlFor="entry_weight" className="text-muted-foreground">進場體重 (kg)</Label>
                <Input
                    id="entry_weight" type="text"
                    value={animal.entry_weight !== undefined && animal.entry_weight !== null ? String(animal.entry_weight) : ''}
                    disabled className="bg-muted"
                />
            </div>

            {/* 實驗前代號 */}
            <div className="space-y-2">
                <Label htmlFor="pre_experiment_code" className="text-muted-foreground">實驗前代號</Label>
                <Input id="pre_experiment_code" value={animal.pre_experiment_code || ''} disabled className="bg-muted" />
            </div>
        </>
    )
}
