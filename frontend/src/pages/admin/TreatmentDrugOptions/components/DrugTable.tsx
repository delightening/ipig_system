import { useMemo } from 'react'
import type { TreatmentDrugOption } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Package, Check, XCircle, Pencil, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'

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
    const columns = useMemo<ColumnDef<TreatmentDrugOption>[]>(() => [
        { key: 'name', header: '藥物名稱', cell: (d) => <span className="font-medium">{d.name}</span> },
        { key: 'display', header: '顯示名稱', cell: (d) => d.display_name || '\u2014' },
        {
            key: 'category', header: '分類',
            cell: (d) => d.category ? (
                <span className="px-2 py-0.5 rounded-full text-xs bg-primary/10 text-primary">{d.category}</span>
            ) : '\u2014',
        },
        { key: 'unit', header: '預設單位', cell: (d) => d.default_dosage_unit || '\u2014' },
        { key: 'sort', header: '排序', className: 'text-center', cell: (d) => d.sort_order },
        {
            key: 'status', header: '狀態', className: 'text-center',
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
                    {d.is_active ? '啟用' : '停用'}
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
            key: 'actions', header: '操作', className: 'text-right',
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
    ], [onEdit, onToggleActive, onDelete])

    return (
        <DataTable
            columns={columns}
            data={drugs}
            isLoading={isLoading}
            emptyIcon={Package}
            emptyTitle="尚無藥物選項"
            emptyDescription="點擊「新增藥物」或「從 ERP 匯入」開始建立"
            rowKey={(d) => d.id}
            rowClassName={(d) => !d.is_active ? 'opacity-50' : ''}
        />
    )
}
