import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { treatmentDrugApi } from '@/lib/api'
import type {
    TreatmentDrugOption,
    CreateTreatmentDrugRequest,
    UpdateTreatmentDrugRequest,
} from '@/types/treatment-drug'
import { useDialogSet } from '@/hooks/useDialogSet'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

const INITIAL_FORM: CreateTreatmentDrugRequest = {
    name: '',
    display_name: '',
    default_dosage_unit: '',
    available_units: [],
    category: '',
    sort_order: 0,
}

export function useDrugOptions() {
    const { t } = useTranslation()
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
            toast({ title: t('common.success'), description: t('adminOps.treatmentDrugs.toast.created') })
        },
        onError: (err: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.createFailed')), variant: 'destructive' })
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
            toast({ title: t('common.success'), description: t('adminOps.treatmentDrugs.toast.updated') })
        },
        onError: (err: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.updateFailed')), variant: 'destructive' })
        },
    })

    // 刪除（軟刪除）
    const deleteMutation = useMutation({
        mutationFn: (id: string) => treatmentDrugApi.delete(id),
        onSuccess: () => {
            invalidateDrugQueries()
            toast({ title: t('common.success'), description: t('adminOps.treatmentDrugs.toast.deactivated') })
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
            toast({ title: t('common.error'), description: t('adminOps.treatmentDrugs.toast.nameRequired'), variant: 'destructive' })
            return
        }
        createMutation.mutate({
            ...form,
            display_name: form.display_name || undefined,
            // 未選預設單位時維持 undefined，不能送空字串——後端會當成
            // 「有設定一個空單位」而一律擋下建立（2026-08 Qodo 審查發現）
            default_dosage_unit: form.default_dosage_unit || undefined,
            available_units: form.available_units?.length ? form.available_units : undefined,
            category: form.category || undefined,
        })
    }

    const handleUpdate = () => {
        if (!editingDrug) return
        updateMutation.mutate({
            id: editingDrug.id,
            data: {
                name: form.name,
                display_name: form.display_name || undefined,
                // 不可比照其他欄位轉成 undefined：undefined＝後端「不動此欄」，
                // 但 toggleUnit 取消勾選目前預設單位時就是要清空成 NULL，若轉成
                // undefined，後端會誤以為沒改、沿用舊值去驗證新的可用單位清單，
                // 導致永遠改不掉（2026-08 Qodo 審查發現）。空字串原樣送出，
                // 由後端正規化成 NULL。
                default_dosage_unit: form.default_dosage_unit,
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
        const confirmMessage = drug.is_active
            ? t('adminOps.treatmentDrugs.confirm.deactivate', { name: drug.name })
            : t('adminOps.treatmentDrugs.confirm.activate', { name: drug.name })
        if (!confirm(confirmMessage)) return
        if (drug.is_active) {
            deleteMutation.mutate(drug.id)
        } else {
            handleToggleActive(drug)
        }
    }

    // 取消勾選目前正被設為「預設劑量單位」的品項時，一併清空預設值——
    // 避免預設值殘留成可用單位清單外的孤兒值（單位 select 會找不到對應 option 而顯示空白）
    const toggleUnit = (unit: string) => {
        setForm((prev) => {
            const nextUnits = prev.available_units?.includes(unit)
                ? prev.available_units.filter((u) => u !== unit)
                : [...(prev.available_units || []), unit]
            return {
                ...prev,
                available_units: nextUnits,
                default_dosage_unit:
                    prev.default_dosage_unit === unit && !nextUnits.includes(unit)
                        ? ''
                        : prev.default_dosage_unit,
            }
        })
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
