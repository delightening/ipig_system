import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { treatmentDrugApi } from '@/lib/api'
import type {
    TreatmentDrugOption,
    CreateTreatmentDrugRequest,
    UpdateTreatmentDrugRequest,
} from '@/types/treatment-drug'
import { useDialogSet } from '@/hooks/useDialogSet'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const INITIAL_FORM: CreateTreatmentDrugRequest = {
    name: '',
    display_name: '',
    default_dosage_unit: '',
    available_units: [],
    category: '',
    sort_order: 0,
}

export function useDrugOptions() {
    const queryClient = useQueryClient()

    // 篩選狀態
    const [keyword, setKeyword] = useState('')
    const [filterCategory, setFilterCategory] = useState<string>('all')
    const [filterActive, setFilterActive] = useState<string>('all')

    // Dialog 狀態
    const dialogs = useDialogSet(['create', 'edit', 'import'] as const)
    const [editingDrug, setEditingDrug] = useState<TreatmentDrugOption | null>(null)

    // 表單狀態
    const [form, setForm] = useState<CreateTreatmentDrugRequest>(INITIAL_FORM)

    const invalidateDrugQueries = () => {
        queryClient.invalidateQueries({ queryKey: ['admin-treatment-drugs'] })
        queryClient.invalidateQueries({ queryKey: ['treatment-drugs'] })
    }

    // 資料查詢
    const { data: drugs = [], isLoading } = useQuery({
        queryKey: ['admin-treatment-drugs', keyword, filterCategory, filterActive],
        queryFn: async () => {
            const params: Record<string, string | boolean | undefined> = {}
            if (keyword) params.keyword = keyword
            if (filterCategory !== 'all') params.category = filterCategory
            if (filterActive !== 'all') params.is_active = filterActive === 'active'
            const res = await treatmentDrugApi.adminList(params)
            return res.data
        },
    })

    // 建立
    const createMutation = useMutation({
        mutationFn: (data: CreateTreatmentDrugRequest) => treatmentDrugApi.create(data),
        onSuccess: () => {
            invalidateDrugQueries()
            dialogs.close('create')
            resetForm()
            toast({ title: '成功', description: '已新增藥物選項' })
        },
        onError: (err: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
        },
    })

    // 更新
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateTreatmentDrugRequest }) =>
            treatmentDrugApi.update(id, data),
        onSuccess: () => {
            invalidateDrugQueries()
            dialogs.close('edit')
            setEditingDrug(null)
            toast({ title: '成功', description: '已更新藥物選項' })
        },
        onError: (err: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
        },
    })

    // 刪除（軟刪除）
    const deleteMutation = useMutation({
        mutationFn: (id: string) => treatmentDrugApi.delete(id),
        onSuccess: () => {
            invalidateDrugQueries()
            toast({ title: '成功', description: '已停用藥物選項' })
        },
    })

    const resetForm = () => {
        setForm(INITIAL_FORM)
    }

    const openEditDialog = (drug: TreatmentDrugOption) => {
        setEditingDrug(drug)
        setForm({
            name: drug.name,
            display_name: drug.display_name || '',
            default_dosage_unit: drug.default_dosage_unit || '',
            available_units: drug.available_units || [],
            category: drug.category || '',
            sort_order: drug.sort_order,
        })
        dialogs.open('edit')
    }

    const handleCreate = () => {
        if (!form.name.trim()) {
            toast({ title: '錯誤', description: '藥品名稱為必填', variant: 'destructive' })
            return
        }
        createMutation.mutate(form)
    }

    const handleUpdate = () => {
        if (!editingDrug) return
        updateMutation.mutate({
            id: editingDrug.id,
            data: {
                name: form.name,
                display_name: form.display_name || undefined,
                default_dosage_unit: form.default_dosage_unit || undefined,
                available_units: form.available_units?.length ? form.available_units : undefined,
                category: form.category || undefined,
                sort_order: form.sort_order,
            },
        })
    }

    const handleToggleActive = (drug: TreatmentDrugOption) => {
        updateMutation.mutate({
            id: drug.id,
            data: { is_active: !drug.is_active },
        })
    }

    const handleDelete = (drug: TreatmentDrugOption) => {
        if (!confirm(`確定要${drug.is_active ? '停用' : '啟用'}「${drug.name}」嗎？`)) return
        if (drug.is_active) {
            deleteMutation.mutate(drug.id)
        } else {
            handleToggleActive(drug)
        }
    }

    const toggleUnit = (unit: string) => {
        setForm((prev) => ({
            ...prev,
            available_units: prev.available_units?.includes(unit)
                ? prev.available_units.filter((u) => u !== unit)
                : [...(prev.available_units || []), unit],
        }))
    }

    return {
        // 篩選
        keyword,
        setKeyword,
        filterCategory,
        setFilterCategory,
        filterActive,
        setFilterActive,
        // 資料
        drugs,
        isLoading,
        // Dialog
        dialogs,
        editingDrug,
        setEditingDrug,
        // 表單
        form,
        setForm,
        resetForm,
        toggleUnit,
        // 操作
        openEditDialog,
        handleCreate,
        handleUpdate,
        handleToggleActive,
        handleDelete,
        createMutation,
        updateMutation,
    }
}
