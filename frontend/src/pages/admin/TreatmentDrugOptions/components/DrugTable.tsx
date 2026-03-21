import type { TreatmentDrugOption } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { Loader2, Package, Check, XCircle, Pencil, Trash2 } from 'lucide-react'
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
    if (isLoading) {
        return (
            <div className="bg-white rounded-lg border overflow-hidden">
                <div className="flex items-center justify-center py-12">
                    <Loader2 className="h-6 w-6 animate-spin text-blue-500" />
                    <span className="ml-2 text-slate-500">載入中...</span>
                </div>
            </div>
        )
    }

    if (drugs.length === 0) {
        return (
            <div className="bg-white rounded-lg border overflow-hidden">
                <div className="text-center py-12 text-slate-500">
                    <Package className="h-12 w-12 mx-auto mb-3 text-slate-300" />
                    <p>尚無藥物選項</p>
                    <p className="text-sm mt-1">點擊「新增藥物」或「從 ERP 匯入」開始建立</p>
                </div>
            </div>
        )
    }

    return (
        <div className="bg-white rounded-lg border overflow-hidden">
            <table className="w-full">
                <thead className="bg-slate-50 border-b">
                    <tr>
                        <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">藥物名稱</th>
                        <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">顯示名稱</th>
                        <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">分類</th>
                        <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">預設單位</th>
                        <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">排序</th>
                        <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">狀態</th>
                        <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">ERP</th>
                        <th className="text-right px-4 py-3 text-xs font-medium text-slate-500 uppercase">操作</th>
                    </tr>
                </thead>
                <tbody className="divide-y">
                    {drugs.map((drug) => (
                        <DrugRow
                            key={drug.id}
                            drug={drug}
                            onEdit={onEdit}
                            onToggleActive={onToggleActive}
                            onDelete={onDelete}
                        />
                    ))}
                </tbody>
            </table>
        </div>
    )
}

function DrugRow({
    drug,
    onEdit,
    onToggleActive,
    onDelete,
}: {
    drug: TreatmentDrugOption
    onEdit: (drug: TreatmentDrugOption) => void
    onToggleActive: (drug: TreatmentDrugOption) => void
    onDelete: (drug: TreatmentDrugOption) => void
}) {
    return (
        <tr className={cn('hover:bg-slate-50', !drug.is_active && 'opacity-50')}>
            <td className="px-4 py-3 text-sm font-medium text-slate-700">{drug.name}</td>
            <td className="px-4 py-3 text-sm text-slate-500">{drug.display_name || '\u2014'}</td>
            <td className="px-4 py-3 text-sm">
                {drug.category ? (
                    <span className="px-2 py-0.5 rounded-full text-xs bg-blue-50 text-blue-600">
                        {drug.category}
                    </span>
                ) : '\u2014'}
            </td>
            <td className="px-4 py-3 text-sm text-slate-500">{drug.default_dosage_unit || '\u2014'}</td>
            <td className="px-4 py-3 text-sm text-center text-slate-400">{drug.sort_order}</td>
            <td className="px-4 py-3 text-center">
                <button
                    onClick={() => onToggleActive(drug)}
                    className={cn(
                        'px-2 py-1 rounded text-xs font-medium transition-colors',
                        drug.is_active
                            ? 'bg-green-50 text-green-700 hover:bg-green-100'
                            : 'bg-red-50 text-red-600 hover:bg-red-100'
                    )}
                >
                    {drug.is_active ? '啟用' : '停用'}
                </button>
            </td>
            <td className="px-4 py-3 text-center">
                {drug.erp_product_id ? (
                    <Check className="h-4 w-4 text-green-500 mx-auto" />
                ) : (
                    <XCircle className="h-4 w-4 text-slate-300 mx-auto" />
                )}
            </td>
            <td className="px-4 py-3 text-right">
                <div className="flex justify-end gap-1">
                    <Button variant="ghost" size="sm" onClick={() => onEdit(drug)}>
                        <Pencil className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="sm" onClick={() => onDelete(drug)}>
                        <Trash2 className="h-4 w-4 text-red-500" />
                    </Button>
                </div>
            </td>
        </tr>
    )
}
