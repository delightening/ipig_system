import { useTranslation } from 'react-i18next'
import type { CreateTreatmentDrugRequest } from '@/types/treatment-drug'
import { DRUG_CATEGORIES, DOSAGE_UNITS } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

import { drugCategoryLabel } from '../constants'

interface DrugFormDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    title: string
    form: CreateTreatmentDrugRequest
    setForm: (fn: (prev: CreateTreatmentDrugRequest) => CreateTreatmentDrugRequest) => void
    onSubmit: () => void
    isLoading: boolean
    toggleUnit: (unit: string) => void
}

export function DrugFormDialog({
    open,
    onOpenChange,
    title,
    form,
    setForm,
    onSubmit,
    isLoading,
    toggleUnit,
}: DrugFormDialogProps) {
    const { t } = useTranslation()
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="sm">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogDescription>{t('adminOps.treatmentDrugs.form.description')}</DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-2">
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.nameLabel')}</Label>
                        <Input
                            value={form.name}
                            onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                            placeholder={t('adminOps.treatmentDrugs.form.namePlaceholder')}
                        />
                        <p className="text-xs text-muted-foreground mt-1">{t('adminOps.treatmentDrugs.form.nameHint')}</p>
                    </div>
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.displayNameLabel')}</Label>
                        <Input
                            value={form.display_name || ''}
                            onChange={(e) => setForm((prev) => ({ ...prev, display_name: e.target.value }))}
                            placeholder={t('adminOps.treatmentDrugs.form.displayNamePlaceholder')}
                        />
                    </div>
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.categoryLabel')}</Label>
                        <Select
                            value={form.category || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, category: v }))}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder={t('adminOps.treatmentDrugs.form.categoryPlaceholder')} />
                            </SelectTrigger>
                            <SelectContent>
                                {DRUG_CATEGORIES.map((cat) => (
                                    <SelectItem key={cat} value={cat}>{drugCategoryLabel(cat, t)}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.availableUnitsLabel')}</Label>
                        <div className="flex flex-wrap gap-2 mt-1">
                            {DOSAGE_UNITS.map((unit) => (
                                <button
                                    key={unit}
                                    type="button"
                                    onClick={() => toggleUnit(unit)}
                                    className={cn(
                                        'px-2 py-1 rounded text-xs border transition-colors',
                                        form.available_units?.includes(unit)
                                            ? 'bg-primary/10 border-primary/30 text-primary'
                                            : 'bg-background border-border text-muted-foreground hover:border-border'
                                    )}
                                >
                                    {unit}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.defaultUnitLabel')}</Label>
                        {/* 選項收斂自上方已勾選的可用單位，結構上不可能選出清單外的值 */}
                        <Select
                            value={form.default_dosage_unit || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, default_dosage_unit: v }))}
                            disabled={!form.available_units?.length}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder={form.available_units?.length ? t('adminOps.treatmentDrugs.form.unitPlaceholder') : t('adminOps.treatmentDrugs.form.unitPlaceholderDisabled')} />
                            </SelectTrigger>
                            <SelectContent>
                                {(form.available_units || []).map((unit) => (
                                    <SelectItem key={unit} value={unit}>{unit}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>{t('adminOps.treatmentDrugs.form.sortOrderLabel')}</Label>
                        <Input
                            type="number"
                            value={form.sort_order || 0}
                            onChange={(e) => setForm((prev) => ({ ...prev, sort_order: parseInt(e.target.value) || 0 }))}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.cancel')}
                    </Button>
                    <Button onClick={onSubmit} disabled={isLoading}>
                        {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {t('common.save')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
