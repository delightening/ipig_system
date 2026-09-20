import { useTranslation } from 'react-i18next'

import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { FilterBar } from '@/components/ui/filter-bar'
import { DRUG_CATEGORIES } from '@/types/treatment-drug'

import { drugCategoryLabel } from '../constants'

interface DrugFilterBarProps {
    keyword: string
    onKeywordChange: (value: string) => void
    filterCategory: string
    onCategoryChange: (value: string) => void
    filterActive: string
    onActiveChange: (value: string) => void
}

export function DrugFilterBar({
    keyword,
    onKeywordChange,
    filterCategory,
    onCategoryChange,
    filterActive,
    onActiveChange,
}: DrugFilterBarProps) {
    const { t } = useTranslation()
    return (
        <FilterBar
            search={keyword}
            onSearchChange={onKeywordChange}
            searchPlaceholder={t('adminOps.treatmentDrugs.filter.searchPlaceholder')}
        >
            <Select value={filterCategory} onValueChange={onCategoryChange}>
                <SelectTrigger className="w-[140px]">
                    <SelectValue placeholder={t('adminOps.treatmentDrugs.filter.categoryPlaceholder')} />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="all">{t('adminOps.treatmentDrugs.filter.allCategories')}</SelectItem>
                    {DRUG_CATEGORIES.map((cat) => (
                        <SelectItem key={cat} value={cat}>{drugCategoryLabel(cat, t)}</SelectItem>
                    ))}
                </SelectContent>
            </Select>
            <Select value={filterActive} onValueChange={onActiveChange}>
                <SelectTrigger className="w-[120px]">
                    <SelectValue placeholder={t('adminOps.treatmentDrugs.filter.statusPlaceholder')} />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="all">{t('adminOps.treatmentDrugs.filter.all')}</SelectItem>
                    <SelectItem value="active">{t('adminOps.treatmentDrugs.filter.active')}</SelectItem>
                    <SelectItem value="inactive">{t('adminOps.treatmentDrugs.filter.inactive')}</SelectItem>
                </SelectContent>
            </Select>
        </FilterBar>
    )
}
