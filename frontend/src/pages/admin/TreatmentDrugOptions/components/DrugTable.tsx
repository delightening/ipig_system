import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { TreatmentDrugOption } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Package, Check, XCircle, Pencil, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'

import { drugCategoryLabel } from '../constants'

interface DrugTableProps {
    drugs: TreatmentDrugOption[]
    isLoading: boolean
    onEdit: (drug: TreatmentDrugOption) => void
    onToggleActive: (drug: TreatmentDrugOption) => void
    onDelete: (drug: TreatmentDrugOption) => void
}

export function DrugTable({
    drugs,
    isLoading,
    onEdit,
    onToggleActive,
    onDelete,
}: DrugTableProps) {
    const { t } = useTranslation()
    const columns = useMemo<ColumnDef<TreatmentDrugOption>[]>(() => [
        { key: 'name', header: t('adminOps.treatmentDrugs.table.colName'), cell: (d) => <span className="font-medium">{d.name}</span> },
        { key: 'display', header: t('adminOps.treatmentDrugs.table.colDisplayName'), cell: (d) => d.display_name || '\u2014' },
        {
            key: 'category', header: t('adminOps.treatmentDrugs.table.colCategory'),
            cell: (d) => d.category ? (
                <span className="px-2 py-0.5 rounded-full text-xs bg-primary/10 text-primary">{drugCategoryLabel(d.category, t)}</span>
            ) : '\u2014',
        },
        { key: 'unit', header: t('adminOps.treatmentDrugs.table.colDefaultUnit'), cell: (d) => d.default_dosage_unit || '\u2014' },
        { key: 'sort', header: t('adminOps.treatmentDrugs.table.colSort'), className: 'text-center', cell: (d) => d.sort_order },
        {
            key: 'status', header: t('adminOps.treatmentDrugs.table.colStatus'), className: 'text-center',
            cell: (d) => (
                <button
                    onClick={() => onToggleActive(d)}
                    className={cn(
                        'px-2 py-1 rounded text-xs font-medium transition-colors',
                        d.is_active
                            ? 'bg-status-success-bg text-status-success-text hover:bg-status-success-bg/80'
                            : 'bg-status-error-bg text-destructive hover:bg-status-error-bg/80'
                    )}
                >
                    {d.is_active ? t('adminOps.treatmentDrugs.table.statusActive') : t('adminOps.treatmentDrugs.table.statusInactive')}
                </button>
            ),
        },
        {
            key: 'erp', header: 'ERP', className: 'text-center',
            cell: (d) => d.erp_product_id
                ? <Check className="h-4 w-4 text-status-success-text mx-auto" />
                : <XCircle className="h-4 w-4 text-muted-foreground/50 mx-auto" />,
        },
        {
            key: 'actions', header: t('common.actions'), className: 'text-right',
            cell: (d) => (
                <div className="flex justify-end gap-1">
                    <Button variant="ghost" size="sm" onClick={() => onEdit(d)}>
                        <Pencil className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="sm" onClick={() => onDelete(d)}>
                        <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                </div>
            ),
        },
    ], [onEdit, onToggleActive, onDelete, t])

    return (
        <DataTable
            columns={columns}
            data={drugs}
            isLoading={isLoading}
            emptyIcon={Package}
            emptyTitle={t('adminOps.treatmentDrugs.table.emptyTitle')}
            emptyDescription={t('adminOps.treatmentDrugs.table.emptyDescription')}
            rowKey={(d) => d.id}
            rowClassName={(d) => !d.is_active ? 'opacity-50' : ''}
        />
    )
}
